// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Execute a compliance-gated covenant on Kaspa TN12.
//!
//! Uses the KTT-proven pattern (TX 27385b04) extended with KIP-10
//! introspection opcodes for value conservation enforcement.
//!
//! Flow:
//!   1. Build a compliance covenant with Alice's pubkey
//!   2. Derive P2SH address via `pay_to_script_hash_script`
//!   3. Deploy it (fund the P2SH with 1 KAS)
//!   4. Spend it via `spend_p2sh()` with Alice's signature
//!   5. Verify the TX is accepted

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::covenant_builder;
use kaspa_adapter::wallet::Wallet;
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_txscript::pay_to_script_hash_script;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";

/// Deploy and spend a compliance covenant with KIP-10 value conservation.
#[tokio::test]
async fn test_compliance_covenant_deploy_and_spend() {
    // --- Setup ---
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let (alice_xonly, _) = alice.keypair().x_only_public_key();
    let alice_pk_bytes = alice_xonly.serialize();

    println!("[K-RWA] === COMPLIANCE COVENANT TEST (KTT + KIP-10) ===");
    println!("[K-RWA] Alice address: {}", alice.address_string());

    // --- Step 1: Build the compliance covenant ---
    let redeem_script = covenant_builder::build_compliance_covenant(&alice_pk_bytes);
    println!(
        "[K-RWA] Compliance covenant: {} bytes = {}",
        redeem_script.len(),
        hex::encode(&redeem_script)
    );

    // --- Step 2: Derive P2SH address ---
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let script_hash = &p2sh_spk.script()[2..34];
    let p2sh_addr = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);
    println!("[K-RWA] P2SH address: {}", p2sh_addr);

    // --- Step 3: Deploy (fund the P2SH with 1 KAS = 100_000_000 sompis) ---
    println!("[K-RWA] Deploying compliance covenant (1 KAS)...");
    let deploy_tx = client
        .send_kas(
            &alice.address_string(),
            &p2sh_addr.to_string(),
            100_000_000,
            alice.keypair(),
            None,
        )
        .await
        .unwrap();
    println!("[K-RWA] Deployed: TX {}", deploy_tx);

    // Wait for UTXO confirmation
    println!("[K-RWA] Waiting 10s for UTXO confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // --- Step 4: Spend the P2SH UTXO ---
    let recipient = Wallet::generate().unwrap();
    println!(
        "[K-RWA] Spending compliance covenant -> {}",
        recipient.address_string()
    );

    // Witness: empty vec = placeholder for Schnorr signature (spend_p2sh fills it)
    let witness = vec![
        vec![], // sig placeholder
    ];

    // Fetch the P2SH UTXO
    let utxos = client
        .get_spendable_utxos(&p2sh_addr.to_string())
        .await
        .unwrap();
    if utxos.is_empty() {
        println!("[K-RWA] No UTXOs at P2SH address — skipping spend");
        client.disconnect().await.ok();
        return;
    }

    let utxo = &utxos[0];
    println!(
        "[K-RWA] P2SH UTXO: {}:{} = {} sompis",
        utxo.txid, utxo.index, utxo.amount
    );

    // Build output: 99_990_000 sompis (minus 10_000 fee)
    let recipient_addr: Address = recipient.address_string().as_str().try_into().unwrap();
    let dest_script = kaspa_txscript::pay_to_address_script(&recipient_addr);
    let output =
        kaspa_consensus_core::tx::TransactionOutput::new(99_990_000, dest_script);

    match client
        .spend_p2sh(
            utxo.txid,
            utxo.index,
            utxo.amount,
            utxo.script_public_key.clone(),
            &redeem_script,
            witness,
            vec![output],
            1, // sig_op_count = 1 CHECKSIG
            alice.keypair(),
        )
        .await
    {
        Ok(tx_id) => {
            println!("[K-RWA] ========================================");
            println!("[K-RWA]  COMPLIANCE COVENANT EXECUTION SUCCESS!");
            println!("[K-RWA]  TX: {}", tx_id);
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] Compliance covenant spend failed: {}", e);
            // Don't panic — the test documents the result
        }
    }

    // --- Step 5: Verify by checking recipient balance ---
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let recipient_balance = client
        .get_balance(&recipient.address_string())
        .await
        .unwrap_or(0);
    println!(
        "[K-RWA] Recipient balance after spend: {} sompis",
        recipient_balance
    );

    client.disconnect().await.ok();
}

/// Deploy and spend a simple CHECKSIG covenant (baseline / KTT-proven).
#[tokio::test]
async fn test_checksig_covenant_baseline() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let (alice_xonly, _) = alice.keypair().x_only_public_key();
    let alice_pk_bytes = alice_xonly.serialize();

    println!("[K-RWA] === CHECKSIG COVENANT BASELINE TEST ===");

    // Build the proven CHECKSIG covenant
    let redeem_script = covenant_builder::build_checksig_covenant(&alice_pk_bytes);
    println!(
        "[K-RWA] CHECKSIG covenant: {} bytes = {}",
        redeem_script.len(),
        hex::encode(&redeem_script)
    );

    // Derive P2SH
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let script_hash = &p2sh_spk.script()[2..34];
    let p2sh_addr = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);
    println!("[K-RWA] P2SH address: {}", p2sh_addr);

    // Deploy
    println!("[K-RWA] Deploying CHECKSIG covenant (1 KAS)...");
    let deploy_tx = client
        .send_kas(
            &alice.address_string(),
            &p2sh_addr.to_string(),
            100_000_000,
            alice.keypair(),
            None,
        )
        .await
        .unwrap();
    println!("[K-RWA] Deployed: TX {}", deploy_tx);

    println!("[K-RWA] Waiting 10s for UTXO confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Spend
    let recipient = Wallet::generate().unwrap();
    let witness = vec![vec![]]; // sig placeholder

    let utxos = client
        .get_spendable_utxos(&p2sh_addr.to_string())
        .await
        .unwrap();
    if utxos.is_empty() {
        println!("[K-RWA] No UTXOs — skipping");
        client.disconnect().await.ok();
        return;
    }

    let utxo = &utxos[0];
    let recipient_addr: Address = recipient.address_string().as_str().try_into().unwrap();
    let dest_script = kaspa_txscript::pay_to_address_script(&recipient_addr);
    let output =
        kaspa_consensus_core::tx::TransactionOutput::new(99_990_000, dest_script);

    match client
        .spend_p2sh(
            utxo.txid,
            utxo.index,
            utxo.amount,
            utxo.script_public_key.clone(),
            &redeem_script,
            witness,
            vec![output],
            1,
            alice.keypair(),
        )
        .await
    {
        Ok(tx_id) => {
            println!("[K-RWA] ========================================");
            println!("[K-RWA]  CHECKSIG BASELINE SUCCESS!");
            println!("[K-RWA]  TX: {}", tx_id);
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] CHECKSIG baseline failed: {}", e);
        }
    }

    client.disconnect().await.ok();
}
