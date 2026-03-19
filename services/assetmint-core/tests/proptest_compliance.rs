// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

use proptest::prelude::*;

use assetmint_core::claims::{Claim, ClaimType};
use assetmint_core::identity::Identity;
use assetmint_core::rules::{ComplianceEngine, ComplianceRule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_identity(did: &str, claims: Vec<Claim>) -> Identity {
    Identity {
        did: did.to_string(),
        primary_key: format!("0x{}", did),
        claims,
        revoked: false,
        created_at: 1000,
    }
}

fn kyc_claim(subject: &str) -> Claim {
    Claim {
        claim_type: ClaimType::KycVerified,
        issuer_did: "did:kaspa:issuer".into(),
        subject_did: subject.into(),
        expiry: 0, // non-expiring
        signature: "test".into(),
        issued_at: 1000,
    }
}

fn jurisdiction_claim(subject: &str, jurisdiction: &str) -> Claim {
    Claim {
        claim_type: ClaimType::JurisdictionAllowed(jurisdiction.to_string()),
        issuer_did: "did:kaspa:issuer".into(),
        subject_did: subject.into(),
        expiry: 0,
        signature: "test".into(),
        issued_at: 1000,
    }
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

proptest! {
    /// MaxTransferAmount allows iff amount <= max.
    #[test]
    fn prop_max_amount_always_enforced(max in 1u64..=u64::MAX, amount: u64) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::MaxTransferAmount(max));

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity("did:kaspa:receiver", vec![]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);

        if amount <= max {
            prop_assert!(result.allowed, "amount {} should be allowed with max {}", amount, max);
        } else {
            prop_assert!(!result.allowed, "amount {} should be denied with max {}", amount, max);
        }
    }

    /// An empty engine (no rules) always allows any transfer for any random amount.
    #[test]
    fn prop_empty_engine_always_allows(
        amount: u64,
        sender_did in "[a-z]{1,10}",
        receiver_did in "[a-z]{1,10}",
    ) {
        let engine = ComplianceEngine::empty();
        let sender = make_identity(&sender_did, vec![]);
        let receiver = make_identity(&receiver_did, vec![]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);
        prop_assert!(result.allowed, "empty engine must always allow");
        prop_assert!(result.violations.is_empty());
        prop_assert_eq!(result.rules_evaluated, 0);
    }

    /// With KYC rules, identities with empty claims are always denied regardless of amount.
    #[test]
    fn prop_kyc_required_denies_without_claim(amount: u64) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified));
        engine.add_rule(ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified));

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity("did:kaspa:receiver", vec![]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);
        prop_assert!(!result.allowed, "no-claim identities must be denied");
        prop_assert!(!result.violations.is_empty());
    }

    /// With KYC rules, identities with KYC claims (expiry=0 means non-expiring) are always
    /// allowed regardless of amount.
    #[test]
    fn prop_kyc_required_allows_with_claim(amount: u64) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified));
        engine.add_rule(ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified));

        let sender = make_identity("did:kaspa:sender", vec![kyc_claim("did:kaspa:sender")]);
        let receiver = make_identity("did:kaspa:receiver", vec![kyc_claim("did:kaspa:receiver")]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);
        prop_assert!(result.allowed, "KYC-holding identities must be allowed");
        prop_assert!(result.violations.is_empty());
    }

    /// For any hold_period and mint_timestamp, once a transfer is allowed at time T
    /// (i.e. now >= mint_ts + period), it stays allowed for all T' > T.
    ///
    /// We test this by controlling mint_timestamp relative to the current wall-clock time.
    /// If `now >= mint_ts + period`, the transfer is allowed. Any later point in time
    /// (simulated by using an even earlier mint_ts) must also be allowed.
    #[test]
    fn prop_hold_period_monotonic(
        hold_period in 1u64..1_000_000u64,
        extra_elapsed in 0u64..1_000_000u64,
    ) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::HoldPeriod(hold_period));

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity("did:kaspa:receiver", vec![]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // mint_timestamp far enough in the past that hold period is satisfied
        let mint_ts = now.saturating_sub(hold_period).saturating_sub(1);
        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", 100, mint_ts);
        prop_assert!(result.allowed, "should be allowed when hold period has passed");

        // Even further in the past — must still be allowed (monotonicity)
        let earlier_mint_ts = mint_ts.saturating_sub(extra_elapsed);
        let result2 = engine.evaluate_transfer(&sender, &receiver, "ASSET", 100, earlier_mint_ts);
        prop_assert!(result2.allowed, "must remain allowed for earlier mint timestamps");
    }

    /// When allowed is false, violations is non-empty. When allowed is true, violations is empty.
    #[test]
    fn prop_violations_count_matches_denied_rules(
        max in 1u64..=u64::MAX,
        amount: u64,
    ) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::MaxTransferAmount(max));

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity("did:kaspa:receiver", vec![]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);

        if result.allowed {
            prop_assert!(result.violations.is_empty(),
                "allowed=true must have no violations, got {:?}", result.violations);
        } else {
            prop_assert!(!result.violations.is_empty(),
                "allowed=false must have at least one violation");
        }
    }

    /// If receiver has JurisdictionAllowed("X") claim and rule blocks "X", transfer is
    /// always denied regardless of amount.
    #[test]
    fn prop_jurisdiction_block_always_blocks(
        amount: u64,
        jurisdiction in "[a-z]{1,10}",
    ) {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::ReceiverJurisdictionNotIn(vec![jurisdiction.clone()]));

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity(
            "did:kaspa:receiver",
            vec![jurisdiction_claim("did:kaspa:receiver", &jurisdiction)],
        );

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);
        prop_assert!(!result.allowed,
            "blocked jurisdiction {} must always deny transfer", jurisdiction);
        prop_assert!(result.violations.iter().any(|v| v.contains("blocked")),
            "violation message should mention 'blocked'");
    }

    /// The rules_evaluated count always equals the total number of individual rules added.
    #[test]
    fn prop_rules_evaluated_equals_rule_count(
        num_max_rules in 0usize..5,
        amount: u64,
    ) {
        let mut engine = ComplianceEngine::empty();

        for i in 0..num_max_rules {
            engine.add_rule(ComplianceRule::MaxTransferAmount((i as u64 + 1) * 1000));
        }

        let sender = make_identity("did:kaspa:sender", vec![]);
        let receiver = make_identity("did:kaspa:receiver", vec![]);

        let result = engine.evaluate_transfer(&sender, &receiver, "ASSET", amount, 0);
        prop_assert_eq!(result.rules_evaluated, num_max_rules,
            "rules_evaluated ({}) must equal rules added ({})",
            result.rules_evaluated, num_max_rules);
    }
}
