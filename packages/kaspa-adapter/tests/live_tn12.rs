// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Live integration test against local kaspad on Testnet-12.
//! Requires kaspad running on 127.0.0.1:17210 with --utxoindex.
//!
//! Run with: cargo test --test live_tn12 -- --nocapture

use kaspa_adapter::client::KaspaClient;

/// Connect to local kaspad and query server info
#[tokio::test]
async fn test_live_server_info() {
    println!("[K-RWA] === LIVE TN12 TEST: Server Info ===\n");

    let client = KaspaClient::new("ws://127.0.0.1:17210")
        .expect("create client");

    client.connect().await.expect("connect to kaspad");
    assert!(client.is_connected(), "Should be connected");

    let info = client.get_server_info().await.expect("get server info");
    println!("[K-RWA] Server version: {}", info.server_version);
    println!("[K-RWA] Synced: {}", info.is_synced);
    println!("[K-RWA] DAA score: {}", info.virtual_daa_score);
    println!("[K-RWA] Network: {}", info.network_id);

    assert!(info.server_version.contains("1.1.0") || !info.server_version.is_empty());
    assert!(info.is_synced, "Node should be synced");

    // Query DAG info
    let (block_count, daa_score, difficulty) = client
        .get_block_dag_info()
        .await
        .expect("get dag info");
    println!("[K-RWA] Block count: {}", block_count);
    println!("[K-RWA] DAA score: {}", daa_score);
    println!("[K-RWA] Difficulty: {:.2}", difficulty);

    assert!(block_count > 0, "Should have blocks");

    client.disconnect().await.expect("disconnect");
    println!("\n[K-RWA] === LIVE TN12 TEST PASSED ===");
}

/// Generate a wallet and query its balance
#[tokio::test]
async fn test_live_wallet_balance() {
    use kaspa_adapter::wallet::Wallet;

    println!("[K-RWA] === LIVE TN12 TEST: Wallet Balance ===\n");

    // Generate a fresh testnet wallet
    let wallet = Wallet::generate().expect("generate wallet");
    let addr = wallet.address_string();
    println!("[K-RWA] Generated wallet: {}", addr);
    assert!(addr.starts_with("kaspatest:"), "Should be testnet address");

    // Connect and query balance (will be 0 for fresh wallet)
    let client = KaspaClient::new("ws://127.0.0.1:17210")
        .expect("create client");
    client.connect().await.expect("connect");

    let balance = client.get_balance(&addr).await.expect("get balance");
    println!("[K-RWA] Balance: {} sompis ({:.4} KAS)", balance, balance as f64 / 1e8);

    let utxos = client.get_utxos(&addr).await.expect("get utxos");
    println!("[K-RWA] UTXOs: {}", utxos.len());

    client.disconnect().await.expect("disconnect");
    println!("\n[K-RWA] === LIVE WALLET TEST PASSED ===");
}
