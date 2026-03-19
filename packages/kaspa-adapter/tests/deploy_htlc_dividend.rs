// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Deploy HTLC + Dividend using fresh wallet funded from Bob.
//! Avoids mempool conflicts by using a wallet with no TX history.

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";

#[tokio::test]
async fn test_deploy_htlc_fresh_wallet() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    // Generate a fresh wallet with no UTXO history
    let fresh = Wallet::generate().unwrap();
    let fresh_addr = fresh.address_string();
    println!("[K-RWA] Fresh wallet: {}", fresh_addr);

    // Fund it from Bob with a larger UTXO (2 KAS) to avoid storage mass issues
    // Kaspa storage mass = C / output_value, so small outputs have high mass
    let bob = Wallet::from_hex(BOB_KEY).unwrap();
    let bob_addr = bob.address_string();
    println!("[K-RWA] Funding from Bob (2 KAS)...");
    let fund_tx = client
        .send_kas(&bob_addr, &fresh_addr, 200_000_000, bob.keypair(), None)
        .await
        .unwrap();
    println!("[K-RWA] Funded fresh wallet: TX {}", fund_tx);

    // Wait for DAG confirmation (TN12 needs ~5-10s for UTXO to appear)
    println!("[K-RWA] Waiting 10s for UTXO confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Deploy HTLC from fresh wallet (1 clean UTXO, no mass issues)
    let htlc = load_contract_json("../../contracts/silverscript/htlc.json").unwrap();
    println!(
        "[K-RWA] Deploying {} ({} bytes)",
        htlc.contract_name,
        htlc.redeem_script.len()
    );
    let htlc_tx = client
        .deploy_contract(&fresh_addr, &htlc, 50_000_000, fresh.keypair())
        .await
        .unwrap();
    println!("[K-RWA] ========================================");
    println!("[K-RWA]  HTLC DEPLOYED!");
    println!("[K-RWA]  TX: {}", htlc_tx);
    println!("[K-RWA]  P2SH: {}", htlc.p2sh_address);
    println!("[K-RWA] ========================================");

    // Wait then deploy Dividend from the change
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let div = load_contract_json("../../contracts/silverscript/dividend.json").unwrap();
    println!(
        "[K-RWA] Deploying {} ({} bytes)",
        div.contract_name,
        div.redeem_script.len()
    );
    let div_tx = client
        .deploy_contract(&fresh_addr, &div, 50_000_000, fresh.keypair())
        .await
        .unwrap();
    println!("[K-RWA] ========================================");
    println!("[K-RWA]  Dividend DEPLOYED!");
    println!("[K-RWA]  TX: {}", div_tx);
    println!("[K-RWA]  P2SH: {}", div.p2sh_address);
    println!("[K-RWA] ========================================");

    client.disconnect().await.ok();
}
