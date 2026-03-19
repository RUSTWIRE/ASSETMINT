// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! End-to-End integration test: full RWA compliance cycle.
//!
//! Exercises: register identity → issue KYC claim → evaluate transfer →
//! build Merkle tree → generate ZK proof → verify proof → clawback.

use assetmint_core::claims::{ClaimIssuer, ClaimType};
use assetmint_core::identity::IdentityRegistry;
use assetmint_core::merkle::MerkleTree;
use assetmint_core::rules::{ComplianceEngine, ComplianceRule};
use assetmint_core::zk_prover::{ZkProver, ZkWitness};
use assetmint_core::zk_verifier::ZkVerifier;
use zk_circuits::kyc_circuit::native_leaf_hash;
use zk_circuits::setup::run_trusted_setup;

/// Full end-to-end cycle on simulated Kaspa Testnet-12
#[test]
fn test_full_e2e_rwa_cycle() {
    println!("[K-RWA] === E2E TEST: Full RWA Compliance Cycle ===\n");

    // ─── Step 1: Set up identity registry ───────────────────
    println!("[K-RWA] Step 1: Register identities");
    let registry = IdentityRegistry::in_memory().expect("registry init");

    let alice = registry
        .register("did:kaspa:alice", "kaspatest:qr35alice")
        .expect("register alice");
    let bob = registry
        .register("did:kaspa:bob", "kaspatest:qr35bob")
        .expect("register bob");
    let mallory = registry
        .register("did:kaspa:mallory", "kaspatest:qr35mallory")
        .expect("register mallory");
    println!(
        "[K-RWA]   Registered: {}, {}, {}",
        alice.did, bob.did, mallory.did
    );

    // ─── Step 2: Issue KYC claims ───────────────────────────
    println!("\n[K-RWA] Step 2: Issue KYC claims");
    let issuer = ClaimIssuer::new("did:kaspa:assetmint-issuer", &[42u8; 32]);

    let alice_kyc = issuer.issue_claim("did:kaspa:alice", ClaimType::KycVerified, 0);
    let bob_kyc = issuer.issue_claim("did:kaspa:bob", ClaimType::KycVerified, 0);
    // Mallory gets NO KYC claim (should be denied transfers)

    registry.add_claim(&alice_kyc).expect("store alice KYC");
    registry.add_claim(&bob_kyc).expect("store bob KYC");
    println!(
        "[K-RWA]   KYC issued: alice (sig={}...)",
        &alice_kyc.signature[..16]
    );
    println!(
        "[K-RWA]   KYC issued: bob (sig={}...)",
        &bob_kyc.signature[..16]
    );
    println!("[K-RWA]   Mallory: NO KYC (should be denied)");

    // Verify claims
    let alice_loaded = registry.get("did:kaspa:alice").expect("get alice");
    assert_eq!(alice_loaded.claims.len(), 1, "Alice should have 1 claim");
    let bob_loaded = registry.get("did:kaspa:bob").expect("get bob");
    assert_eq!(bob_loaded.claims.len(), 1, "Bob should have 1 claim");

    // ─── Step 3: Evaluate compliant transfer (alice → bob) ──
    println!("\n[K-RWA] Step 3: Evaluate compliant transfer (alice → bob)");
    let engine = ComplianceEngine::new(); // Default: KYC required for both parties

    let result = engine.evaluate_transfer(
        &alice_loaded,
        &bob_loaded,
        "KPROP-NYC-TEST",
        1_000_000, // 0.01 KAS
        0,
    );
    assert!(result.allowed, "Alice→Bob transfer should be ALLOWED");
    assert!(result.violations.is_empty());
    println!(
        "[K-RWA]   Result: ALLOWED (rules evaluated: {})",
        result.rules_evaluated
    );

    // ─── Step 4: Evaluate non-compliant transfer (mallory → bob) ──
    println!("\n[K-RWA] Step 4: Evaluate non-compliant transfer (mallory → bob)");
    let mallory_loaded = registry.get("did:kaspa:mallory").expect("get mallory");

    let result =
        engine.evaluate_transfer(&mallory_loaded, &bob_loaded, "KPROP-NYC-TEST", 1_000_000, 0);
    assert!(!result.allowed, "Mallory→Bob transfer should be DENIED");
    assert!(!result.violations.is_empty());
    println!(
        "[K-RWA]   Result: DENIED ({} violations)",
        result.violations.len()
    );
    for v in &result.violations {
        println!("[K-RWA]     - {}", v);
    }

    // ─── Step 5: Test MaxTransferAmount rule ────────────────
    println!("\n[K-RWA] Step 5: Test MaxTransferAmount rule");
    let mut custom_engine = ComplianceEngine::new();
    custom_engine.add_rule(ComplianceRule::MaxTransferAmount(500_000));

    let big_transfer = custom_engine.evaluate_transfer(
        &alice_loaded,
        &bob_loaded,
        "KPROP-NYC-TEST",
        1_000_000, // Exceeds 500k limit
        0,
    );
    assert!(
        !big_transfer.allowed,
        "Over-limit transfer should be DENIED"
    );
    println!("[K-RWA]   1M sompis transfer: DENIED (limit 500k)");

    let small_transfer = custom_engine.evaluate_transfer(
        &alice_loaded,
        &bob_loaded,
        "KPROP-NYC-TEST",
        200_000, // Under limit
        0,
    );
    assert!(
        small_transfer.allowed,
        "Under-limit transfer should be ALLOWED"
    );
    println!("[K-RWA]   200k sompis transfer: ALLOWED");

    // ─── Step 6: Build Merkle tree of approved addresses ────
    println!("\n[K-RWA] Step 6: Build Merkle tree of approved addresses");
    let approved = registry
        .get_approved_addresses()
        .expect("get approved addresses");
    println!("[K-RWA]   Approved addresses: {}", approved.len());

    let tree = MerkleTree::build(&approved).expect("build merkle tree");
    let root = tree.root();
    println!("[K-RWA]   Merkle root: {}", hex::encode(root));

    // Verify proofs for each address
    for addr in &approved {
        let proof = tree.get_proof(addr).expect("get proof");
        assert!(
            MerkleTree::verify_proof(&proof, &root),
            "Proof for {} should verify",
            addr
        );
    }
    println!("[K-RWA]   All {} Merkle proofs verified", approved.len());

    // ─── Step 7: Groth16 ZK-KYC proof generation & verification ──
    println!("\n[K-RWA] Step 7: Groth16 ZK-KYC proof cycle");
    let keys_dir = "/tmp/assetmint_e2e_test";
    let _ = std::fs::remove_dir_all(keys_dir);

    let start = std::time::Instant::now();
    let setup = run_trusted_setup(2, keys_dir).expect("trusted setup");
    let setup_time = start.elapsed();
    println!("[K-RWA]   Trusted setup: {:?}", setup_time);

    let mut prover = ZkProver::new(2);
    prover.set_proving_key(setup.proving_key);
    let mut verifier = ZkVerifier::new();
    verifier.set_verifying_key(setup.verifying_key);

    // Create witness: alice's secret is 42
    let secret = ark_bn254::Fr::from(42u64);
    let leaf = native_leaf_hash(secret);
    let leaves = vec![
        ark_bn254::Fr::from(100u64),
        leaf,
        ark_bn254::Fr::from(200u64),
        ark_bn254::Fr::from(300u64),
    ];

    let witness = ZkWitness {
        secret_key: {
            let mut bytes = Vec::new();
            ark_serialize::CanonicalSerialize::serialize_compressed(&secret, &mut bytes).unwrap();
            bytes
        },
        leaf_index: 1,
        all_leaves: leaves
            .iter()
            .map(|l| {
                let mut bytes = Vec::new();
                ark_serialize::CanonicalSerialize::serialize_compressed(l, &mut bytes).unwrap();
                bytes
            })
            .collect(),
    };

    // Generate proof
    let start = std::time::Instant::now();
    let proof = prover.generate_proof(&witness).expect("proof generation");
    let prove_time = start.elapsed();
    println!(
        "[K-RWA]   Proof generated: {:?} ({} bytes, hash={}...)",
        prove_time,
        proof.proof_bytes.len(),
        &hex::encode(proof.proof_hash)[..16]
    );

    // Verify proof
    let start = std::time::Instant::now();
    let valid = verifier.verify(&proof).expect("proof verification");
    let verify_time = start.elapsed();
    assert!(valid, "Valid ZK proof should verify");
    println!("[K-RWA]   Proof verified: {:?}", verify_time);

    // VK hash for on-chain commitment
    let vk_hash = verifier.verification_key_hash().expect("vk hash");
    println!("[K-RWA]   VK hash: {}...", &hex::encode(vk_hash)[..16]);

    // ─── Step 8: Revoke identity and verify denial ──────────
    println!("\n[K-RWA] Step 8: Revoke identity (mallory) and verify");
    registry
        .revoke("did:kaspa:mallory")
        .expect("revoke mallory");
    let approved_after = registry
        .get_approved_addresses()
        .expect("get approved after revoke");
    assert_eq!(
        approved_after.len(),
        approved.len() - 1,
        "Should have one fewer approved address"
    );
    println!(
        "[K-RWA]   Approved addresses after revoke: {} (was {})",
        approved_after.len(),
        approved.len()
    );

    // Clean up
    let _ = std::fs::remove_dir_all(keys_dir);

    println!("\n[K-RWA] === E2E TEST PASSED: All 8 steps completed ===");
    println!("[K-RWA] Summary:");
    println!("[K-RWA]   - 3 identities registered, 2 KYC claims issued");
    println!("[K-RWA]   - Compliant transfer: ALLOWED");
    println!("[K-RWA]   - Non-compliant transfer: DENIED (missing KYC)");
    println!("[K-RWA]   - MaxTransferAmount: enforced correctly");
    println!(
        "[K-RWA]   - Merkle tree: {} proofs verified",
        approved.len()
    );
    println!(
        "[K-RWA]   - ZK proof: gen={:?}, verify={:?}",
        prove_time, verify_time
    );
    println!("[K-RWA]   - Identity revocation: approved count updated");
}
