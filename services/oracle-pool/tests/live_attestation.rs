// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Integration test: fetch a live (or simulated-fallback) KAS price,
//! create a 2-of-3 Ed25519 multisig attestation, verify it, and print
//! the attestation hash that would be committed on-chain.

use oracle_pool::attestation::{create_attestation, create_testnet_signers, verify_attestation};
use oracle_pool::oracle::{get_aggregated_price, get_simulated_price};

/// Synchronous path: use simulated price sources, create & verify attestation.
/// Works offline — no CoinGecko or Kaspa connection required.
#[test]
fn test_attestation_from_simulated_price() {
    let price = get_aggregated_price("KAS").expect("simulated aggregation");
    assert!(price.price_usd > 0.0, "price must be positive");

    let signers = create_testnet_signers();
    // Sign with 2-of-3 (threshold)
    let signer_refs: Vec<&_> = signers.iter().take(2).collect();
    let attestation = create_attestation(price, &signer_refs).expect("create attestation");

    assert_eq!(attestation.signatures.len(), 2);
    assert_eq!(attestation.threshold, 2);
    assert!(!attestation.data_hash.is_empty());

    let valid = verify_attestation(&attestation).expect("verify attestation");
    assert!(valid, "2-of-3 attestation must verify");

    println!("--- Simulated Attestation ---");
    println!("Asset:      {}", attestation.price.asset_id);
    println!("Price:      ${:.6}", attestation.price.price_usd);
    println!("Sources:    {}", attestation.price.sources_used);
    println!("Timestamp:  {}", attestation.price.timestamp);
    println!("Data hash:  {}", attestation.data_hash);
    println!("Signatures: {}", attestation.signatures.len());
    println!("Signers:    {:?}", attestation.signer_pubkeys);
}

/// Synchronous path: full 3-of-3 signing round.
#[test]
fn test_attestation_all_three_signers() {
    let price = get_aggregated_price("KPROP-NYC-TEST").expect("property price");
    let signers = create_testnet_signers();
    let signer_refs: Vec<&_> = signers.iter().collect();

    let attestation = create_attestation(price, &signer_refs).expect("create attestation");
    assert_eq!(attestation.signatures.len(), 3);

    let valid = verify_attestation(&attestation).expect("verify");
    assert!(valid);

    println!("--- Property Token Attestation ---");
    println!("Asset:     {}", attestation.price.asset_id);
    println!("Price:     ${:.2}", attestation.price.price_usd);
    println!("Data hash: {}", attestation.data_hash);
}

/// Async path: fetch live price from CoinGecko (falls back to simulated),
/// then create and verify a multisig attestation.
#[tokio::test]
async fn test_live_price_attestation() {
    use oracle_pool::oracle::get_live_aggregated_price;

    let price = get_live_aggregated_price("KAS")
        .await
        .expect("live aggregated price");

    assert!(price.price_usd > 0.0);
    assert!(price.sources_used >= 2);

    let signers = create_testnet_signers();
    let signer_refs: Vec<&_> = signers.iter().take(2).collect();
    let attestation = create_attestation(price, &signer_refs).expect("create attestation");

    let valid = verify_attestation(&attestation).expect("verify");
    assert!(valid, "live attestation must verify");

    println!("--- Live Price Attestation ---");
    println!("Asset:      {}", attestation.price.asset_id);
    println!("Price:      ${:.6}", attestation.price.price_usd);
    println!("Sources:    {}", attestation.price.sources_used);
    println!("Timestamp:  {}", attestation.price.timestamp);
    println!("Data hash:  {}", attestation.data_hash);
    println!(
        "On-chain:   send minimal KAS to address derived from hash to timestamp this attestation"
    );
}

/// Verify that a single simulated price point can be wrapped in an
/// attestation (useful for quick oracle health checks).
#[test]
fn test_single_source_attestation_hash() {
    let point = get_simulated_price("KAS");
    assert!(point.price_usd > 0.0);

    // Build an AggregatedPrice from the single point for hashing
    let agg = oracle_pool::oracle::AggregatedPrice {
        price_usd: point.price_usd,
        sources_used: 1,
        sources_rejected: 0,
        timestamp: point.timestamp,
        asset_id: "KAS".to_string(),
    };

    let data = oracle_pool::attestation::build_attestation_data(&agg);
    let hash = hex::encode(&data);
    assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
    println!("Single-source hash: {}", hash);
}
