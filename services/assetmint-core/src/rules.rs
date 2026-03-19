// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! AssetMint Compliance Engine — implements Polymesh Compliance.Requirements patterns.
//! Multi-jurisdiction rule composition (US Reg D/S, EU MiCA, Singapore MAS).
//! Ported from Polymesh SDK `ComplianceManager` into composable AND/OR rule groups.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::claims::ClaimType;
use crate::identity::Identity;
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("[K-RWA] Transfer denied: {violations:?}")]
    TransferDenied { violations: Vec<String> },
}

/// A single compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceRule {
    /// Sender must have a specific claim type
    SenderMustHaveClaim(ClaimType),
    /// Receiver must have a specific claim type
    ReceiverMustHaveClaim(ClaimType),
    /// Receiver jurisdiction must NOT be in blocked list
    ReceiverJurisdictionNotIn(Vec<String>),
    /// Maximum single transfer amount (in sompis)
    MaxTransferAmount(u64),
    /// Minimum hold period before transfer (seconds since mint)
    HoldPeriod(u64),
    /// US SEC Reg D: Only accredited investors may participate
    RegDAccreditedOnly,
    /// US SEC Reg S: Non-US persons only (offshore transactions)
    RegSNonUS,
    /// SEC Rule 144: Restricted securities hold period
    Rule144HoldPeriod { months: u64 },
    /// EU MiCA: Crypto-Asset Service Provider prospectus requirement
    MiCAProspectusRequired,
    /// Singapore MAS: Accredited investor requirement (SG definition)
    MASAccreditedInvestor,
}

/// Result of a compliance evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub allowed: bool,
    pub violations: Vec<String>,
    pub rules_evaluated: usize,
}

/// A requirement group — either all rules must pass (AND) or at least one (OR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementGroup {
    /// All rules must pass
    All(Vec<ComplianceRule>),
    /// At least one rule must pass
    Any(Vec<ComplianceRule>),
}

/// A jurisdiction-specific compliance profile combining multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionProfile {
    pub name: String,
    pub jurisdiction_code: String,
    pub description: String,
    pub rules: Vec<ComplianceRule>,
}

impl JurisdictionProfile {
    /// US Reg D: Accredited investors only + KYC + AML + 12-month hold
    pub fn us_reg_d() -> Self {
        Self {
            name: "US SEC Regulation D".to_string(),
            jurisdiction_code: "US-REG-D".to_string(),
            description: "Accredited investors only — private placement exemption".to_string(),
            rules: vec![
                ComplianceRule::RegDAccreditedOnly,
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::SenderMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::Rule144HoldPeriod { months: 12 },
            ],
        }
    }

    /// US Reg S: Non-US persons + KYC + AML
    pub fn us_reg_s() -> Self {
        Self {
            name: "US SEC Regulation S".to_string(),
            jurisdiction_code: "US-REG-S".to_string(),
            description: "Offshore transactions — non-US persons only".to_string(),
            rules: vec![
                ComplianceRule::RegSNonUS,
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::SenderMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::AmlClear),
            ],
        }
    }

    /// EU MiCA: KYC + AML + prospectus requirement + EU jurisdiction
    pub fn eu_mica() -> Self {
        Self {
            name: "EU Markets in Crypto-Assets".to_string(),
            jurisdiction_code: "EU-MICA".to_string(),
            description: "MiCA-compliant crypto-asset issuance with prospectus".to_string(),
            rules: vec![
                ComplianceRule::MiCAProspectusRequired,
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::SenderMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::AmlClear),
            ],
        }
    }

    /// Singapore MAS: Accredited investors (SG) + KYC + AML
    pub fn sg_mas() -> Self {
        Self {
            name: "Singapore MAS Accredited Investor".to_string(),
            jurisdiction_code: "SG-MAS".to_string(),
            description: "MAS-regulated accredited investor requirement".to_string(),
            rules: vec![
                ComplianceRule::MASAccreditedInvestor,
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::SenderMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::AmlClear),
            ],
        }
    }

    /// Global default: KYC + AML only (minimal)
    pub fn global_default() -> Self {
        Self {
            name: "Global Default".to_string(),
            jurisdiction_code: "GLOBAL".to_string(),
            description: "Minimal compliance — KYC and AML only".to_string(),
            rules: vec![
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::SenderMustHaveClaim(ClaimType::AmlClear),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::AmlClear),
            ],
        }
    }
}

/// Compliance engine — evaluates transfer rules
pub struct ComplianceEngine {
    requirements: Vec<RequirementGroup>,
}

impl ComplianceEngine {
    /// Create a new compliance engine with default RWA rules
    pub fn new() -> Self {
        info!("{} Initializing compliance engine", LOG_PREFIX);
        Self {
            requirements: vec![RequirementGroup::All(vec![
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverJurisdictionNotIn(vec!["OFAC_BLOCKED".to_string()]),
            ])],
        }
    }

    /// Create an empty engine (for custom configuration)
    pub fn empty() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    /// Add a requirement group
    pub fn add_requirement(&mut self, group: RequirementGroup) {
        info!("{} Adding requirement group", LOG_PREFIX);
        self.requirements.push(group);
    }

    /// Apply a jurisdiction-specific compliance profile
    pub fn apply_jurisdiction_profile(&mut self, profile: &JurisdictionProfile) {
        info!(
            "{} Applying jurisdiction profile: {} ({})",
            LOG_PREFIX, profile.name, profile.jurisdiction_code
        );
        self.add_requirement(RequirementGroup::All(profile.rules.clone()));
    }

    /// Add a single rule as an AND requirement
    pub fn add_rule(&mut self, rule: ComplianceRule) {
        info!("{} Adding compliance rule: {:?}", LOG_PREFIX, rule);
        self.requirements.push(RequirementGroup::All(vec![rule]));
    }

    /// Evaluate whether a transfer is allowed.
    /// `mint_timestamp` is when the asset was minted (for HoldPeriod checks).
    pub fn evaluate_transfer(
        &self,
        sender: &Identity,
        receiver: &Identity,
        _asset_id: &str,
        amount: u64,
        mint_timestamp: u64,
    ) -> ComplianceResult {
        info!(
            "{} Evaluating transfer: {} -> {} (amount={})",
            LOG_PREFIX, sender.did, receiver.did, amount
        );

        let mut violations = Vec::new();
        let mut rules_count = 0;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for group in &self.requirements {
            match group {
                RequirementGroup::All(rules) => {
                    for rule in rules {
                        rules_count += 1;
                        if let Some(v) =
                            evaluate_rule(rule, sender, receiver, amount, mint_timestamp, now)
                        {
                            violations.push(v);
                        }
                    }
                }
                RequirementGroup::Any(rules) => {
                    rules_count += rules.len();
                    let any_pass = rules.iter().all(|rule| {
                        evaluate_rule(rule, sender, receiver, amount, mint_timestamp, now).is_some()
                    });
                    if any_pass && !rules.is_empty() {
                        violations.push("No rule in OR group was satisfied".to_string());
                    }
                }
            }
        }

        let allowed = violations.is_empty();
        info!(
            "{} Compliance evaluation: allowed={}, violations={}, rules={}",
            LOG_PREFIX,
            allowed,
            violations.len(),
            rules_count
        );

        ComplianceResult {
            allowed,
            violations,
            rules_evaluated: rules_count,
        }
    }
}

/// Evaluate a single rule, return Some(violation_message) if the rule fails
fn evaluate_rule(
    rule: &ComplianceRule,
    sender: &Identity,
    receiver: &Identity,
    amount: u64,
    mint_timestamp: u64,
    now: u64,
) -> Option<String> {
    match rule {
        ComplianceRule::SenderMustHaveClaim(claim_type) => {
            let has = sender
                .claims
                .iter()
                .any(|c| c.claim_type == *claim_type && (c.expiry == 0 || c.expiry > now));
            if !has {
                Some(format!(
                    "Sender {} missing required claim: {:?}",
                    sender.did, claim_type
                ))
            } else {
                None
            }
        }
        ComplianceRule::ReceiverMustHaveClaim(claim_type) => {
            let has = receiver
                .claims
                .iter()
                .any(|c| c.claim_type == *claim_type && (c.expiry == 0 || c.expiry > now));
            if !has {
                Some(format!(
                    "Receiver {} missing required claim: {:?}",
                    receiver.did, claim_type
                ))
            } else {
                None
            }
        }
        ComplianceRule::MaxTransferAmount(max) => {
            if amount > *max {
                Some(format!(
                    "Transfer amount {} exceeds maximum {}",
                    amount, max
                ))
            } else {
                None
            }
        }
        ComplianceRule::ReceiverJurisdictionNotIn(blocked) => {
            for claim in &receiver.claims {
                if let ClaimType::JurisdictionAllowed(jurisdiction) = &claim.claim_type {
                    if blocked.contains(jurisdiction) {
                        return Some(format!("Receiver jurisdiction {} is blocked", jurisdiction));
                    }
                }
            }
            None
        }
        ComplianceRule::HoldPeriod(period) => {
            if mint_timestamp > 0 && now < mint_timestamp + period {
                let remaining = (mint_timestamp + period) - now;
                Some(format!(
                    "Hold period not met: {} seconds remaining",
                    remaining
                ))
            } else {
                None
            }
        }
        ComplianceRule::RegDAccreditedOnly => {
            let sender_accredited = sender.claims.iter().any(|c| {
                c.claim_type == ClaimType::AccreditedInvestor && (c.expiry == 0 || c.expiry > now)
            });
            let receiver_accredited = receiver.claims.iter().any(|c| {
                c.claim_type == ClaimType::AccreditedInvestor && (c.expiry == 0 || c.expiry > now)
            });
            if !sender_accredited || !receiver_accredited {
                Some(format!(
                    "Reg D: Both parties must be accredited investors (sender={}, receiver={})",
                    sender_accredited, receiver_accredited
                ))
            } else {
                None
            }
        }
        ComplianceRule::RegSNonUS => {
            let receiver_is_us = receiver.claims.iter().any(|c| {
                matches!(&c.claim_type, ClaimType::JurisdictionAllowed(j) if j == "US")
                    && (c.expiry == 0 || c.expiry > now)
            });
            if receiver_is_us {
                Some("Reg S: Receiver must not be a US person".to_string())
            } else {
                None
            }
        }
        ComplianceRule::Rule144HoldPeriod { months } => {
            let hold_seconds = months * 30 * 86400;
            if mint_timestamp > 0 && now < mint_timestamp + hold_seconds {
                let remaining = (mint_timestamp + hold_seconds) - now;
                Some(format!(
                    "Rule 144: Hold period of {} months not met ({} seconds remaining)",
                    months, remaining
                ))
            } else {
                None
            }
        }
        ComplianceRule::MiCAProspectusRequired => {
            let sender_exempted = sender.claims.iter().any(|c| {
                c.claim_type == ClaimType::ExemptedEntity && (c.expiry == 0 || c.expiry > now)
            });
            if !sender_exempted {
                Some("MiCA: Sender must be an exempted entity (prospectus filed)".to_string())
            } else {
                None
            }
        }
        ComplianceRule::MASAccreditedInvestor => {
            let receiver_accredited = receiver.claims.iter().any(|c| {
                c.claim_type == ClaimType::AccreditedInvestor && (c.expiry == 0 || c.expiry > now)
            });
            let receiver_sg = receiver.claims.iter().any(|c| {
                matches!(&c.claim_type, ClaimType::JurisdictionAllowed(j) if j == "SG")
                    && (c.expiry == 0 || c.expiry > now)
            });
            if !receiver_accredited || !receiver_sg {
                Some(format!(
                    "MAS: Receiver must be SG accredited investor (accredited={}, sg={})",
                    receiver_accredited, receiver_sg
                ))
            } else {
                None
            }
        }
    }
}

impl Default for ComplianceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::Claim;

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
            expiry: 0,
            signature: "test".into(),
            issued_at: 1000,
            key_version: 1,
        }
    }

    #[test]
    fn test_both_kyc_passes() {
        let engine = ComplianceEngine::new();
        let sender = make_identity("did:kaspa:alice", vec![kyc_claim("did:kaspa:alice")]);
        let receiver = make_identity("did:kaspa:bob", vec![kyc_claim("did:kaspa:bob")]);

        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(result.allowed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_missing_sender_kyc_fails() {
        let engine = ComplianceEngine::new();
        let sender = make_identity("did:kaspa:alice", vec![]); // no KYC
        let receiver = make_identity("did:kaspa:bob", vec![kyc_claim("did:kaspa:bob")]);

        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("Sender"));
    }

    #[test]
    fn test_max_transfer_amount() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::MaxTransferAmount(5000));

        let sender = make_identity("did:kaspa:alice", vec![]);
        let receiver = make_identity("did:kaspa:bob", vec![]);

        let ok = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 3000, 0);
        assert!(ok.allowed);

        let denied = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 6000, 0);
        assert!(!denied.allowed);
    }

    #[test]
    fn test_blocked_jurisdiction() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::ReceiverJurisdictionNotIn(vec![
            "BLOCKED_COUNTRY".into(),
        ]));

        let sender = make_identity("did:kaspa:alice", vec![]);
        let receiver = make_identity(
            "did:kaspa:bob",
            vec![Claim {
                claim_type: ClaimType::JurisdictionAllowed("BLOCKED_COUNTRY".into()),
                issuer_did: "did:kaspa:issuer".into(),
                subject_did: "did:kaspa:bob".into(),
                expiry: 0,
                signature: "test".into(),
                issued_at: 1000,
                key_version: 1,
            }],
        );

        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("blocked"));
    }

    #[test]
    fn test_hold_period() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::HoldPeriod(86400)); // 1 day

        let sender = make_identity("did:kaspa:alice", vec![]);
        let receiver = make_identity("did:kaspa:bob", vec![]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Minted just now — hold period not met
        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, now);
        assert!(!result.allowed);

        // Minted 2 days ago — hold period met
        let result =
            engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, now - 172800);
        assert!(result.allowed);
    }

    // ── Multi-Jurisdiction Compliance Tests ──────────────────────────────

    fn accredited_claim(subject: &str) -> Claim {
        Claim {
            claim_type: ClaimType::AccreditedInvestor,
            issuer_did: "did:kaspa:issuer".into(),
            subject_did: subject.into(),
            expiry: 0,
            signature: "test".into(),
            issued_at: 1000,
            key_version: 1,
        }
    }

    fn aml_claim(subject: &str) -> Claim {
        Claim {
            claim_type: ClaimType::AmlClear,
            issuer_did: "did:kaspa:issuer".into(),
            subject_did: subject.into(),
            expiry: 0,
            signature: "test".into(),
            issued_at: 1000,
            key_version: 1,
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
            key_version: 1,
        }
    }

    fn exempted_claim(subject: &str) -> Claim {
        Claim {
            claim_type: ClaimType::ExemptedEntity,
            issuer_did: "did:kaspa:issuer".into(),
            subject_did: subject.into(),
            expiry: 0,
            signature: "test".into(),
            issued_at: 1000,
            key_version: 1,
        }
    }

    #[test]
    fn test_reg_d_accredited_only() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::RegDAccreditedOnly);

        // Both accredited → pass
        let sender = make_identity("did:kaspa:alice", vec![accredited_claim("did:kaspa:alice")]);
        let receiver = make_identity("did:kaspa:bob", vec![accredited_claim("did:kaspa:bob")]);
        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(result.allowed);

        // Sender not accredited → fail
        let sender_no = make_identity("did:kaspa:alice", vec![]);
        let result = engine.evaluate_transfer(&sender_no, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("Reg D"));

        // Receiver not accredited → fail
        let receiver_no = make_identity("did:kaspa:bob", vec![]);
        let result = engine.evaluate_transfer(&sender, &receiver_no, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
    }

    #[test]
    fn test_reg_s_non_us() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::RegSNonUS);

        let sender = make_identity("did:kaspa:alice", vec![]);

        // Non-US receiver → pass
        let receiver_eu = make_identity(
            "did:kaspa:bob",
            vec![jurisdiction_claim("did:kaspa:bob", "EU")],
        );
        let result = engine.evaluate_transfer(&sender, &receiver_eu, "KPROP-NYC-TEST", 1000, 0);
        assert!(result.allowed);

        // US receiver → fail
        let receiver_us = make_identity(
            "did:kaspa:bob",
            vec![jurisdiction_claim("did:kaspa:bob", "US")],
        );
        let result = engine.evaluate_transfer(&sender, &receiver_us, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("Reg S"));
    }

    #[test]
    fn test_rule_144_hold_period() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::Rule144HoldPeriod { months: 6 });

        let sender = make_identity("did:kaspa:alice", vec![]);
        let receiver = make_identity("did:kaspa:bob", vec![]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Minted 7 months ago (> 6 months) → pass
        let seven_months_ago = now - (7 * 30 * 86400);
        let result =
            engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, seven_months_ago);
        assert!(result.allowed);

        // Minted 2 months ago (< 6 months) → fail
        let two_months_ago = now - (2 * 30 * 86400);
        let result =
            engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, two_months_ago);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("Rule 144"));
    }

    #[test]
    fn test_mica_prospectus() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::MiCAProspectusRequired);

        let receiver = make_identity("did:kaspa:bob", vec![]);

        // Exempted entity sender → pass
        let sender_ok = make_identity("did:kaspa:issuer", vec![exempted_claim("did:kaspa:issuer")]);
        let result = engine.evaluate_transfer(&sender_ok, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(result.allowed);

        // Non-exempted sender → fail
        let sender_no = make_identity("did:kaspa:alice", vec![]);
        let result = engine.evaluate_transfer(&sender_no, &receiver, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("MiCA"));
    }

    #[test]
    fn test_mas_accredited_sg() {
        let mut engine = ComplianceEngine::empty();
        engine.add_rule(ComplianceRule::MASAccreditedInvestor);

        let sender = make_identity("did:kaspa:alice", vec![]);

        // SG accredited investor → pass
        let receiver_ok = make_identity(
            "did:kaspa:bob",
            vec![
                accredited_claim("did:kaspa:bob"),
                jurisdiction_claim("did:kaspa:bob", "SG"),
            ],
        );
        let result = engine.evaluate_transfer(&sender, &receiver_ok, "KPROP-NYC-TEST", 1000, 0);
        assert!(result.allowed);

        // Accredited but not SG → fail
        let receiver_no_sg =
            make_identity("did:kaspa:bob", vec![accredited_claim("did:kaspa:bob")]);
        let result = engine.evaluate_transfer(&sender, &receiver_no_sg, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);

        // SG but not accredited → fail
        let receiver_no_acc = make_identity(
            "did:kaspa:bob",
            vec![jurisdiction_claim("did:kaspa:bob", "SG")],
        );
        let result = engine.evaluate_transfer(&sender, &receiver_no_acc, "KPROP-NYC-TEST", 1000, 0);
        assert!(!result.allowed);
        assert!(result.violations[0].contains("MAS"));
    }

    #[test]
    fn test_jurisdiction_profile_composition() {
        let mut engine = ComplianceEngine::empty();
        let profile = JurisdictionProfile::us_reg_d();
        engine.apply_jurisdiction_profile(&profile);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Fully compliant: accredited + KYC + AML + minted > 12 months ago
        let sender = make_identity(
            "did:kaspa:alice",
            vec![
                accredited_claim("did:kaspa:alice"),
                kyc_claim("did:kaspa:alice"),
                aml_claim("did:kaspa:alice"),
            ],
        );
        let receiver = make_identity(
            "did:kaspa:bob",
            vec![
                accredited_claim("did:kaspa:bob"),
                kyc_claim("did:kaspa:bob"),
                aml_claim("did:kaspa:bob"),
            ],
        );

        let thirteen_months_ago = now - (13 * 30 * 86400);
        let result = engine.evaluate_transfer(
            &sender,
            &receiver,
            "KPROP-NYC-TEST",
            1000,
            thirteen_months_ago,
        );
        assert!(result.allowed, "violations: {:?}", result.violations);

        // Missing AML → fail
        let receiver_no_aml = make_identity(
            "did:kaspa:bob",
            vec![
                accredited_claim("did:kaspa:bob"),
                kyc_claim("did:kaspa:bob"),
            ],
        );
        let result = engine.evaluate_transfer(
            &sender,
            &receiver_no_aml,
            "KPROP-NYC-TEST",
            1000,
            thirteen_months_ago,
        );
        assert!(!result.allowed);

        // Minted too recently → fail (Rule 144 12-month hold)
        let two_months_ago = now - (2 * 30 * 86400);
        let result =
            engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", 1000, two_months_ago);
        assert!(!result.allowed);
    }
}
