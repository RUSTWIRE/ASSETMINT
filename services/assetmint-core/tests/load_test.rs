// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Load test: 10k+ compliance evaluations with throughput measurement.
//! Simulates concurrent RWA transfer compliance checks.

use assetmint_core::claims::{ClaimIssuer, ClaimType};
use assetmint_core::identity::IdentityRegistry;
use assetmint_core::merkle::MerkleTree;
use assetmint_core::rules::{ComplianceEngine, ComplianceRule, RequirementGroup};
use std::time::Instant;

const NUM_IDENTITIES: usize = 200;
const NUM_EVALUATIONS: usize = 10_000;

#[test]
fn test_load_10k_compliance_evaluations() {
    println!(
        "[K-RWA] === LOAD TEST: {} compliance evaluations ===",
        NUM_EVALUATIONS
    );

    // 1. Setup: register identities with KYC claims
    let registry = IdentityRegistry::in_memory().unwrap();
    let issuer = ClaimIssuer::new("did:kaspa:issuer", &[42u8; 32]);

    for i in 0..NUM_IDENTITIES {
        let did = format!("did:kaspa:user-{}", i);
        registry.register(&did, &format!("0x{:064x}", i)).unwrap();
        let claim = issuer.issue_claim(&did, ClaimType::KycVerified, 0);
        registry.add_claim(&claim).unwrap();

        // Half get accredited investor status
        if i % 2 == 0 {
            let claim = issuer.issue_claim(&did, ClaimType::AccreditedInvestor, 0);
            registry.add_claim(&claim).unwrap();
        }
    }

    println!(
        "[K-RWA] Registered {} identities with KYC claims",
        NUM_IDENTITIES
    );

    // 2. Configure compliance engine with realistic rules
    let mut engine = ComplianceEngine::new();
    engine.add_requirement(RequirementGroup::All(vec![
        ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
        ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
        ComplianceRule::MaxTransferAmount(1_000_000_000_000), // 10,000 KAS
        ComplianceRule::ReceiverJurisdictionNotIn(vec![
            "OFAC-sanctioned".into(),
            "high-risk".into(),
        ]),
    ]));

    // 3. Run load test
    let start = Instant::now();
    let mut allowed = 0u64;
    let mut denied = 0u64;

    for i in 0..NUM_EVALUATIONS {
        let sender_idx = i % NUM_IDENTITIES;
        let receiver_idx = (i + 1) % NUM_IDENTITIES;
        let sender_did = format!("did:kaspa:user-{}", sender_idx);
        let receiver_did = format!("did:kaspa:user-{}", receiver_idx);

        let sender = registry.get(&sender_did).unwrap();
        let receiver = registry.get(&receiver_did).unwrap();

        // Vary amounts — some exceed max
        let amount = if i % 100 == 0 {
            2_000_000_000_000 // Over limit → denied
        } else {
            (i as u64 + 1) * 100_000 // Normal amount
        };

        let result = engine.evaluate_transfer(&sender, &receiver, "KPROP-NYC-TEST", amount, 0);

        if result.allowed {
            allowed += 1;
        } else {
            denied += 1;
        }
    }

    let elapsed = start.elapsed();
    let throughput = NUM_EVALUATIONS as f64 / elapsed.as_secs_f64();

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  LOAD TEST RESULTS");
    println!("[K-RWA]  Evaluations: {}", NUM_EVALUATIONS);
    println!("[K-RWA]  Allowed:     {}", allowed);
    println!("[K-RWA]  Denied:      {}", denied);
    println!(
        "[K-RWA]  Duration:    {:.3}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    println!("[K-RWA]  Throughput:  {:.0} evals/sec", throughput);
    println!(
        "[K-RWA]  Avg latency: {:.0}ns per eval",
        elapsed.as_nanos() as f64 / NUM_EVALUATIONS as f64
    );
    println!("[K-RWA] ========================================");

    // Assertions
    // Note: In debug mode, SQLite claim lookups limit throughput to ~23k/sec.
    // In release mode with in-memory cache, this exceeds 1M/sec (see benchmarks).
    assert!(
        throughput > 10_000.0,
        "Throughput must exceed 10k evals/sec (debug mode), got {:.0}",
        throughput
    );
    assert!(allowed > 0, "Some transfers should be allowed");
    assert!(denied > 0, "Some transfers should be denied (max amount)");
}

#[test]
fn test_load_merkle_tree_10k_leaves() {
    println!("[K-RWA] === LOAD TEST: Merkle tree with 10k leaves ===");

    let addresses: Vec<String> = (0..10_000)
        .map(|i| format!("kaspatest:qr{:064x}", i))
        .collect();

    let start = Instant::now();
    let tree = MerkleTree::build(&addresses).unwrap();
    let root = tree.root();
    let build_time = start.elapsed();

    println!("[K-RWA] Tree built: root={}", hex::encode(root));
    println!(
        "[K-RWA] Build time: {:.3}ms for {} leaves",
        build_time.as_secs_f64() * 1000.0,
        addresses.len()
    );

    // Verify 1000 proofs
    let start = Instant::now();
    let mut verified = 0;
    for i in 0..1000 {
        let proof = tree.get_proof(&addresses[i]).unwrap();
        if MerkleTree::verify_proof(&proof, &root) {
            verified += 1;
        }
    }
    let verify_time = start.elapsed();

    println!(
        "[K-RWA] Verified {} proofs in {:.3}ms ({:.0} proofs/sec)",
        verified,
        verify_time.as_secs_f64() * 1000.0,
        1000.0 / verify_time.as_secs_f64()
    );

    assert_eq!(verified, 1000);
    assert!(build_time.as_millis() < 5000, "Tree build should be <5s");
}
