// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM KRC-20 inscription deployment on Kaspa Testnet-12.
//! Broadcasts the deploy and mint inscriptions as OP_RETURN data.
//!
//! Run: cargo test -p kaspa-adapter --test deploy_astm -- --nocapture

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";

#[tokio::test]
async fn test_deploy_astm_inscription() {
    let inscription = tokenomics::token::deploy_inscription();
    println!(
        "[K-RWA] ASTM deploy inscription: {} bytes",
        inscription.inscription_data.len()
    );
    println!(
        "[K-RWA] Data: {}",
        String::from_utf8_lossy(&inscription.inscription_data)
    );

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let wallet = Wallet::from_hex(BOB_KEY).expect("Failed to load wallet");
    let addr = wallet.address_string();

    // Send inscription as OP_RETURN data in a self-send transaction
    let tx_id = client
        .send_kas(
            &addr,
            &addr,
            1_000_000,
            wallet.keypair(),
            Some(inscription.inscription_data),
        )
        .await
        .expect("ASTM deploy inscription broadcast failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  ASTM KRC-20 DEPLOY INSCRIPTION BROADCAST!");
    println!("[K-RWA]  TX: {}", tx_id);
    println!("[K-RWA] ========================================");

    client.disconnect().await.ok();
}

#[tokio::test]
async fn test_mint_astm_inscription() {
    let inscription =
        tokenomics::token::mint_inscription(500).expect("Failed to build mint inscription");
    println!(
        "[K-RWA] ASTM mint inscription: {} bytes",
        inscription.inscription_data.len()
    );

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let wallet = Wallet::from_hex(BOB_KEY).expect("Failed to load wallet");
    let addr = wallet.address_string();

    // Wait for deploy TX to clear mempool
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let tx_id = client
        .send_kas(
            &addr,
            &addr,
            1_000_000,
            wallet.keypair(),
            Some(inscription.inscription_data),
        )
        .await
        .expect("ASTM mint inscription broadcast failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  ASTM KRC-20 MINT INSCRIPTION BROADCAST!");
    println!("[K-RWA]  TX: {}", tx_id);
    println!("[K-RWA] ========================================");

    client.disconnect().await.ok();
}
