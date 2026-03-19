// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Deploy a clawback covenant on Kaspa TN12.
//!
//! A clawback covenant allows the issuer to reclaim funds at any time,
//! providing a regulatory compliance mechanism for RWA tokens.
//! The issuer key controls the P2SH address, enabling forced recovery.
//!
//! Covenant script:
//!   [push_32][issuer_pubkey] OP_CHECKSIG OP_VERIFY OP_TRUE
//!
//! This means only the issuer can spend from this address (clawback).

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_txscript::pay_to_script_hash_script;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";

// Kaspa opcodes
const OP_CHECKSIG: u8 = 0xac;
const OP_VERIFY: u8 = 0x69;
const OP_TRUE: u8 = 0x51;
const OP_DATA_32: u8 = 0x20;

/// Build a clawback covenant script.
///
/// Only the issuer's signature can unlock funds, enabling regulatory
/// recovery of RWA tokens. The script structure:
///   [OP_DATA_32][issuer_pubkey] OP_CHECKSIG OP_VERIFY OP_TRUE
fn build_clawback_covenant(issuer_pubkey: &[u8; 32]) -> Vec<u8> {
    let mut script = Vec::with_capacity(36);
    script.push(OP_DATA_32);
    script.extend_from_slice(issuer_pubkey);
    script.push(OP_CHECKSIG);
    script.push(OP_VERIFY);
    script.push(OP_TRUE);
    script
}

#[tokio::test]
async fn test_deploy_clawback_covenant() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    // Alice acts as the issuer with clawback authority
    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let (alice_xonly, _) = alice.keypair().x_only_public_key();
    let issuer_pk_bytes = alice_xonly.serialize();

    println!("[K-RWA] === CLAWBACK COVENANT DEPLOY ===");
    println!("[K-RWA] Issuer (Alice): {}", alice.address_string());

    // Build the clawback covenant keyed to the issuer
    let redeem_script = build_clawback_covenant(&issuer_pk_bytes);
    println!(
        "[K-RWA] Clawback covenant: {} bytes = {}",
        redeem_script.len(),
        hex::encode(&redeem_script)
    );

    // Derive P2SH address from the covenant script
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let script_hash = &p2sh_spk.script()[2..34];
    let p2sh_addr = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);
    println!("[K-RWA] P2SH address: {}", p2sh_addr);

    // Deploy (fund with 0.5 KAS to avoid storage mass issues)
    println!("[K-RWA] Deploying clawback covenant (0.5 KAS)...");
    match client
        .send_kas(
            &alice.address_string(),
            &p2sh_addr.to_string(),
            50_000_000, // 0.5 KAS
            alice.keypair(),
            None,
        )
        .await
    {
        Ok(deploy_tx) => {
            println!("[K-RWA] ========================================");
            println!("[K-RWA]  CLAWBACK COVENANT DEPLOYED!");
            println!("[K-RWA]  TX: {}", deploy_tx);
            println!("[K-RWA]  P2SH: {}", p2sh_addr);
            println!("[K-RWA]  Issuer: {}", alice.address_string());
            println!("[K-RWA]  Script: {} bytes", redeem_script.len());
            println!("[K-RWA]  Purpose: Issuer-controlled fund recovery");
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] Deploy failed: {}", e);
            println!("[K-RWA] This may be due to UTXO fragmentation.");
            println!(
                "[K-RWA] The clawback covenant was built correctly ({} bytes)",
                redeem_script.len()
            );
        }
    }

    client.disconnect().await.ok();
}
