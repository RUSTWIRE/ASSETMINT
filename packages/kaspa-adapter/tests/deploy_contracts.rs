// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Deploy SilverScript contracts on Kaspa Testnet-12.
//! Compiles contracts with real issuer keys, funds P2SH addresses on-chain.
//!
//! Run: cargo test -p kaspa-adapter --test deploy_contracts -- --nocapture

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;

/// Issuer's testnet private key (from testnet-config.json)
const ISSUER_KEY: &str = "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d";
const RPC_URL: &str = "ws://127.0.0.1:17210";

/// Funding amount per contract: 0.01 KAS (1_000_000 sompis)
const DEPLOY_AMOUNT: u64 = 1_000_000;

#[tokio::test]
async fn test_deploy_clawback_contract() {
    println!("[K-RWA] === DEPLOYING Clawback CONTRACT ON TN12 ===");

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let issuer = Wallet::from_hex(ISSUER_KEY).expect("Failed to load issuer wallet");
    let issuer_addr = issuer.address_string();
    println!("[K-RWA] Issuer address: {}", issuer_addr);

    // Load compiled contract
    let contract = load_contract_json("../../contracts/silverscript/clawback.json")
        .expect("Failed to load clawback contract");
    println!("[K-RWA] Contract: {}", contract.contract_name);
    println!(
        "[K-RWA] Redeem script: {} bytes",
        contract.redeem_script.len()
    );
    println!("[K-RWA] P2SH address: {}", contract.p2sh_address);
    println!("[K-RWA] Entrypoints:");
    for f in &contract.abi {
        let params: Vec<String> = f
            .inputs
            .iter()
            .map(|i| format!("{}: {}", i.name, i.type_name))
            .collect();
        println!("[K-RWA]   - {}({})", f.name, params.join(", "));
    }

    // Deploy: fund the P2SH address
    let tx_id = client
        .deploy_contract(&issuer_addr, &contract, DEPLOY_AMOUNT, issuer.keypair())
        .await
        .expect("Deployment failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  CLAWBACK CONTRACT DEPLOYED!");
    println!("[K-RWA]  TX ID:   {}", tx_id);
    println!("[K-RWA]  P2SH:    {}", contract.p2sh_address);
    println!(
        "[K-RWA]  Funding: {} sompis ({:.4} KAS)",
        DEPLOY_AMOUNT,
        DEPLOY_AMOUNT as f64 / 1e8
    );
    println!("[K-RWA] ========================================");

    client.disconnect().await.expect("Failed to disconnect");
}

#[tokio::test]
async fn test_deploy_rwa_core_contract() {
    println!("[K-RWA] === DEPLOYING RwaCore CONTRACT ON TN12 ===");

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let issuer = Wallet::from_hex(ISSUER_KEY).expect("Failed to load issuer wallet");
    let issuer_addr = issuer.address_string();

    let contract = load_contract_json("../../contracts/silverscript/rwa-core.json")
        .expect("Failed to load rwa-core contract");
    println!(
        "[K-RWA] Contract: {} ({} bytes, {} entrypoints)",
        contract.contract_name,
        contract.redeem_script.len(),
        contract.abi.len()
    );
    println!("[K-RWA] P2SH: {}", contract.p2sh_address);

    let tx_id = client
        .deploy_contract(&issuer_addr, &contract, DEPLOY_AMOUNT, issuer.keypair())
        .await
        .expect("Deployment failed");

    println!("[K-RWA] ========================================");
    println!("[K-RWA]  RWA-CORE CONTRACT DEPLOYED!");
    println!("[K-RWA]  TX ID: {}", tx_id);
    println!("[K-RWA]  P2SH:  {}", contract.p2sh_address);
    println!("[K-RWA] ========================================");

    client.disconnect().await.expect("Failed to disconnect");
}

#[tokio::test]
async fn test_deploy_all_contracts() {
    println!("[K-RWA] === DEPLOYING ALL 5 CONTRACTS ON TN12 ===");

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let issuer = Wallet::from_hex(ISSUER_KEY).expect("Failed to load issuer wallet");
    let issuer_addr = issuer.address_string();

    let balance = client
        .get_balance(&issuer_addr)
        .await
        .expect("Balance query failed");
    println!("[K-RWA] Issuer balance: {:.4} KAS", balance as f64 / 1e8);
    assert!(
        balance > 5 * DEPLOY_AMOUNT + 500_000,
        "Need at least 0.055 KAS for 5 deployments"
    );

    let contracts = [
        "../../contracts/silverscript/clawback.json",
        "../../contracts/silverscript/rwa-core.json",
        "../../contracts/silverscript/state-verity.json",
        "../../contracts/silverscript/zkkyc-verifier.json",
        "../../contracts/silverscript/reserves.json",
    ];

    println!("[K-RWA] ========================================");
    for (i, path) in contracts.iter().enumerate() {
        let contract =
            load_contract_json(path).unwrap_or_else(|e| panic!("Failed to load {}: {}", path, e));

        // Use slightly different amounts to avoid duplicate TX IDs
        let amount = DEPLOY_AMOUNT + (i as u64 * 1000);

        // Retry loop: UTXO set may need time to update after mempool changes
        let mut tx_id = String::new();
        for attempt in 0..5 {
            let result = client
                .deploy_contract(&issuer_addr, &contract, amount, issuer.keypair())
                .await;

            match result {
                Ok(id) => {
                    tx_id = id.to_string();
                    break;
                }
                Err(e) => {
                    let err_str = format!("{}", e);
                    if err_str.contains("already in the mempool") {
                        println!(
                            "[K-RWA]  {} — already in mempool (previously deployed)",
                            contract.contract_name
                        );
                        tx_id = "(already deployed)".to_string();
                        break;
                    } else if err_str.contains("already spent") && attempt < 4 {
                        println!(
                            "[K-RWA]  {} — UTXO conflict, waiting 5s (attempt {}/5)...",
                            contract.contract_name,
                            attempt + 1
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    } else {
                        panic!(
                            "Deploy {} failed after {} attempts: {}",
                            contract.contract_name,
                            attempt + 1,
                            e
                        );
                    }
                }
            }
        }

        println!(
            "[K-RWA]  {} | {} bytes | P2SH {} | TX {}",
            contract.contract_name,
            contract.redeem_script.len(),
            contract.p2sh_address,
            tx_id
        );

        // Wait for change UTXO to become available
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    println!("[K-RWA] ========================================");
    println!("[K-RWA]  ALL 5 CONTRACTS DEPLOYED ON TN12!");
    println!("[K-RWA] ========================================");

    client.disconnect().await.expect("Failed to disconnect");
}
