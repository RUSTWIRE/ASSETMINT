// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Deploy individual SilverScript contracts from different wallets.
//! Each test deploys one contract to avoid UTXO conflicts.
//!
//! Run all: cargo test -p kaspa-adapter --test deploy_single -- --nocapture --test-threads=1
//! Run one: cargo test -p kaspa-adapter --test deploy_single test_deploy_rwa_core -- --nocapture

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

const RPC_URL: &str = "ws://127.0.0.1:17210";

// Bob's wallet — 7718 KAS available, large UTXOs
const BOB_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";

/// Deploy amount: 0.01 KAS
const DEPLOY_AMOUNT: u64 = 1_000_000;

async fn deploy_one(contract_path: &str, private_key: &str, amount_offset: u64) {
    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let wallet = Wallet::from_hex(private_key).expect("Failed to load wallet");
    let addr = wallet.address_string();

    let contract = load_contract_json(contract_path)
        .unwrap_or_else(|e| panic!("Failed to load {}: {}", contract_path, e));

    println!("[K-RWA] Deploying {} ({} bytes) from {}",
        contract.contract_name, contract.redeem_script.len(), addr);
    println!("[K-RWA] P2SH address: {}", contract.p2sh_address);

    // Use slightly different amounts to avoid duplicate TX IDs
    let amount = DEPLOY_AMOUNT + amount_offset;

    // Retry with delay — mempool-spent UTXOs clear after block confirmation
    let mut last_err = String::new();
    for attempt in 0..10 {
        let try_amount = amount + (attempt as u64 * 100);
        match client
            .deploy_contract(&addr, &contract, try_amount, wallet.keypair())
            .await
        {
            Ok(tx_id) => {
                println!("[K-RWA] ========================================");
                println!("[K-RWA]  {} DEPLOYED!", contract.contract_name);
                println!("[K-RWA]  TX: {}", tx_id);
                println!("[K-RWA]  P2SH: {}", contract.p2sh_address);
                println!("[K-RWA] ========================================");
                client.disconnect().await.expect("Failed to disconnect");
                return;
            }
            Err(e) => {
                last_err = format!("{}", e);
                if last_err.contains("already spent") || last_err.contains("mempool") {
                    println!("[K-RWA]  Attempt {}/10 — UTXO conflict, waiting 3s...", attempt + 1);
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                } else if last_err.contains("already in the mempool") {
                    println!("[K-RWA]  {} already deployed", contract.contract_name);
                    client.disconnect().await.expect("Failed to disconnect");
                    return;
                } else {
                    break;
                }
            }
        }
    }

    client.disconnect().await.ok();
    panic!("Deploy {} failed after retries: {}", contract.contract_name, last_err);
}

#[tokio::test]
async fn test_deploy_rwa_core() {
    deploy_one("../../contracts/silverscript/rwa-core.json", BOB_KEY, 0).await;
}

#[tokio::test]
async fn test_deploy_state_verity() {
    deploy_one("../../contracts/silverscript/state-verity.json", BOB_KEY, 1000).await;
}

#[tokio::test]
async fn test_deploy_zkkyc_verifier() {
    deploy_one("../../contracts/silverscript/zkkyc-verifier.json", BOB_KEY, 2000).await;
}

#[tokio::test]
async fn test_deploy_reserves() {
    deploy_one("../../contracts/silverscript/reserves.json", BOB_KEY, 3000).await;
}
