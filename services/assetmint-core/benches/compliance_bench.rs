// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

use assetmint_core::claims::{Claim, ClaimType};
use assetmint_core::identity::Identity;
use assetmint_core::merkle::MerkleTree;
use assetmint_core::rules::ComplianceEngine;
use criterion::{criterion_group, criterion_main, Criterion};

/// Helper: build an Identity with a single KYC claim
fn kyc_identity(did: &str) -> Identity {
    Identity {
        did: did.to_string(),
        primary_key: format!("0x{}", did),
        claims: vec![Claim {
            claim_type: ClaimType::KycVerified,
            issuer_did: "did:kaspa:issuer".into(),
            subject_did: did.into(),
            expiry: 0,
            signature: "bench-sig".into(),
            issued_at: 1_000,
        }],
        revoked: false,
        created_at: 1_000,
    }
}

/// Benchmark: evaluate a compliant transfer through the default KYC rule-set
fn compliance_evaluation(c: &mut Criterion) {
    let engine = ComplianceEngine::new();
    let sender = kyc_identity("did:kaspa:alice");
    let receiver = kyc_identity("did:kaspa:bob");

    c.bench_function("evaluate_transfer (default KYC rules)", |b| {
        b.iter(|| {
            let result = engine.evaluate_transfer(
                &sender,
                &receiver,
                "KPROP-NYC-BENCH",
                10_000,
                0,
            );
            assert!(result.allowed);
        });
    });
}

/// Benchmark: build a Merkle tree from 100 approved addresses
fn merkle_tree_build(c: &mut Criterion) {
    let addresses: Vec<String> = (0..100)
        .map(|i| format!("kaspatest:qz9bench{:04}", i))
        .collect();

    c.bench_function("merkle_tree_build (100 leaves)", |b| {
        b.iter(|| {
            let tree = MerkleTree::build(&addresses).expect("build must succeed");
            criterion::black_box(tree.root());
        });
    });
}

/// Benchmark: verify a single Merkle inclusion proof
fn merkle_proof_verify(c: &mut Criterion) {
    let addresses: Vec<String> = (0..100)
        .map(|i| format!("kaspatest:qz9bench{:04}", i))
        .collect();
    let tree = MerkleTree::build(&addresses).expect("build must succeed");
    let root = tree.root();
    let proof = tree
        .get_proof("kaspatest:qz9bench0042")
        .expect("proof must exist");

    c.bench_function("merkle_proof_verify (single proof)", |b| {
        b.iter(|| {
            assert!(MerkleTree::verify_proof(&proof, &root));
        });
    });
}

criterion_group!(
    benches,
    compliance_evaluation,
    merkle_tree_build,
    merkle_proof_verify,
);
criterion_main!(benches);
