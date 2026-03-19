// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Deploy HTLC and Dividend contracts on Kaspa Testnet-12.
use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ISSUER_KEY: &str = "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d";

#[tokio::test]
async fn test_deploy_htlc() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();
    let wallet = Wallet::from_hex(ISSUER_KEY).unwrap();
    let addr = wallet.address_string();
    let contract = load_contract_json("../../contracts/silverscript/htlc.json").unwrap();
    println!(
        "[K-RWA] Deploying {} ({} bytes)",
        contract.contract_name,
        contract.redeem_script.len()
    );
    let tx_id = client
        .deploy_contract(&addr, &contract, 1_000_000, wallet.keypair())
        .await
        .unwrap();
    println!(
        "[K-RWA] HTLC DEPLOYED! TX: {} P2SH: {}",
        tx_id, contract.p2sh_address
    );
    client.disconnect().await.ok();
}

#[tokio::test]
async fn test_deploy_dividend() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();
    let wallet = Wallet::from_hex(ISSUER_KEY).unwrap();
    let addr = wallet.address_string();
    let contract = load_contract_json("../../contracts/silverscript/dividend.json").unwrap();
    println!(
        "[K-RWA] Deploying {} ({} bytes)",
        contract.contract_name,
        contract.redeem_script.len()
    );
    let tx_id = client
        .deploy_contract(&addr, &contract, 1_001_000, wallet.keypair())
        .await
        .unwrap();
    println!(
        "[K-RWA] Dividend DEPLOYED! TX: {} P2SH: {}",
        tx_id, contract.p2sh_address
    );
    client.disconnect().await.ok();
}
