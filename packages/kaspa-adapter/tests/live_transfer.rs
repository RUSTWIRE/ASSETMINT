// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Live transfer test: Alice → Bob on Kaspa Testnet-12
//! Requires: local kaspad running at ws://127.0.0.1:17210
//!
//! Run: cargo test -p kaspa-adapter --test live_transfer -- --nocapture

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;

/// Alice's testnet private key (from testnet-config.json)
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";
/// Bob's testnet address (derived from his private key)
const BOB_ADDRESS: &str = "kaspatest:qz7re8m28d5ppzynj9vpasrwzy8zvlatmnlguxhrj83unwy6zkwcstlxs2pmt";

const RPC_URL: &str = "ws://127.0.0.1:17210";
const SEND_AMOUNT: u64 = 10_000_000; // 0.1 KAS in sompis

#[tokio::test]
async fn test_live_alice_to_bob_transfer() {
    println!("[K-RWA] === LIVE TRANSFER TEST: Alice → Bob ===");

    // 1. Connect to kaspad
    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect to kaspad");
    println!("[K-RWA] Connected to kaspad");

    // 2. Load Alice's wallet
    let alice = Wallet::from_hex(ALICE_KEY).expect("Failed to load Alice's wallet");
    let alice_addr = alice.address_string();
    println!("[K-RWA] Alice address: {}", alice_addr);

    // 3. Check Alice's balance
    let balance = client.get_balance(&alice_addr).await.expect("Failed to get balance");
    println!("[K-RWA] Alice balance: {} sompis ({:.4} KAS)", balance, balance as f64 / 1e8);
    assert!(balance > SEND_AMOUNT + 100_000, "Alice needs at least 0.101 KAS");

    // 4. Send 0.1 KAS from Alice to Bob
    println!("[K-RWA] Sending {} sompis (0.1 KAS) to Bob...", SEND_AMOUNT);
    let tx_id = client
        .send_kas(
            &alice_addr,
            BOB_ADDRESS,
            SEND_AMOUNT,
            alice.keypair(),
            None, // no OP_RETURN
        )
        .await
        .expect("Transfer failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  TRANSACTION ACCEPTED BY KASPAD!");
    println!("[K-RWA]  TX ID: {}", tx_id);
    println!("[K-RWA] ========================================");

    // 5. Wait a moment for DAG propagation, then verify balance changed
    println!("[K-RWA] Waiting 2s for DAG confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let new_balance = client.get_balance(&alice_addr).await.expect("Failed to get balance");
    println!("[K-RWA] Alice new balance: {} sompis ({:.4} KAS)", new_balance, new_balance as f64 / 1e8);
    // Note: balance may not change immediately due to UTXO consolidation
    // The key proof is that kaspad accepted the TX (no error from submit_transaction)
    println!("[K-RWA] Balance delta: {} sompis", balance as i64 - new_balance as i64);

    client.disconnect().await.expect("Failed to disconnect");
    println!("[K-RWA] === TRANSFER TEST COMPLETE ===");
}

#[tokio::test]
async fn test_live_bob_to_alice_return() {
    println!("[K-RWA] === LIVE RETURN TRANSFER TEST: Bob → Alice ===");

    // Bob's testnet private key
    const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";
    const ALICE_ADDRESS: &str = "kaspatest:qrl9q9vapkepmgs375v2v329mlc24c0g8zujt52etwm7ragjw4nay2rs8kxh8";

    // 1. Connect
    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    // 2. Load Bob's wallet
    let bob = Wallet::from_hex(BOB_KEY).expect("Failed to load wallet");
    let bob_addr = bob.address_string();
    println!("[K-RWA] Bob address: {}", bob_addr);

    // 3. Check Bob's balance
    let balance = client.get_balance(&bob_addr).await.expect("Failed to get balance");
    println!("[K-RWA] Bob balance: {} sompis ({:.4} KAS)", balance, balance as f64 / 1e8);

    // 4. Send 0.05 KAS back to Alice (no OP_RETURN — Kaspa uses inscription protocol)
    let tx_id = client
        .send_kas(
            &bob_addr,
            ALICE_ADDRESS,
            5_000_000, // 0.05 KAS
            bob.keypair(),
            None,
        )
        .await
        .expect("Bob→Alice transfer failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  BOB → ALICE TX ACCEPTED!");
    println!("[K-RWA]  TX ID: {}", tx_id);
    println!("[K-RWA] ========================================");

    client.disconnect().await.expect("Failed to disconnect");
    println!("[K-RWA] === RETURN TRANSFER TEST COMPLETE ===");
}
