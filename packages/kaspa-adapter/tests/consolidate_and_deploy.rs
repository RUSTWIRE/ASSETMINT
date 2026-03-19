// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Consolidate fragmented UTXOs then deploy HTLC + Dividend contracts.

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ISSUER_KEY: &str = "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d";

#[tokio::test]
async fn test_consolidate_then_deploy() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();
    let wallet = Wallet::from_hex(ISSUER_KEY).unwrap();
    let addr = wallet.address_string();

    // Step 1: Check current UTXO count
    let utxos = client.get_spendable_utxos(&addr).await.unwrap();
    println!("[K-RWA] Issuer has {} UTXOs before consolidation", utxos.len());

    // Step 2: Consolidate if needed
    if utxos.len() > 5 {
        let batches = client.consolidate_utxos(&addr, wallet.keypair()).await.unwrap();
        println!("[K-RWA] Consolidated in {} batches", batches);
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    // Step 3: Deploy HTLC
    let htlc = load_contract_json("../../contracts/silverscript/htlc.json").unwrap();
    println!("[K-RWA] Deploying {} ({} bytes)", htlc.contract_name, htlc.redeem_script.len());
    let htlc_tx = client.deploy_contract(&addr, &htlc, 1_000_000, wallet.keypair()).await.unwrap();
    println!("[K-RWA] HTLC DEPLOYED! TX: {} P2SH: {}", htlc_tx, htlc.p2sh_address);

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Step 4: Deploy Dividend
    let div = load_contract_json("../../contracts/silverscript/dividend.json").unwrap();
    println!("[K-RWA] Deploying {} ({} bytes)", div.contract_name, div.redeem_script.len());
    let div_tx = client.deploy_contract(&addr, &div, 1_001_000, wallet.keypair()).await.unwrap();
    println!("[K-RWA] Dividend DEPLOYED! TX: {} P2SH: {}", div_tx, div.p2sh_address);

    client.disconnect().await.ok();
}
