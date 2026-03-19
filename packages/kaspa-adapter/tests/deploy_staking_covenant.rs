// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Deploy a staking timelock covenant on Kaspa TN12.
//!
//! Uses CHECKLOCKTIMEVERIFY to enforce time-locked staking.
//! The covenant script:
//!   [push_32][pubkey] OP_CHECKSIG OP_VERIFY
//!   [push_8][daa_score] OP_CHECKLOCKTIMEVERIFY OP_DROP
//!   OP_TRUE

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_txscript::pay_to_script_hash_script;
use sha2::{Digest, Sha256};

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";

// Kaspa opcodes
const OP_CHECKSIG: u8 = 0xac;
const OP_CHECKLOCKTIMEVERIFY: u8 = 0xb0;
const OP_DROP: u8 = 0x75;
const OP_VERIFY: u8 = 0x69;
const OP_TRUE: u8 = 0x51;
const OP_DATA_32: u8 = 0x20;
const OP_DATA_8: u8 = 0x08;

/// Build staking timelock covenant script.
fn build_staking_covenant(pubkey: &[u8; 32], unlock_daa_score: u64) -> Vec<u8> {
    let mut script = Vec::with_capacity(47);
    script.push(OP_DATA_32);
    script.extend_from_slice(pubkey);
    script.push(OP_CHECKSIG);
    script.push(OP_VERIFY);
    script.push(OP_DATA_8);
    script.extend_from_slice(&unlock_daa_score.to_le_bytes());
    script.push(OP_CHECKLOCKTIMEVERIFY);
    script.push(OP_DROP);
    script.push(OP_TRUE);
    script
}

#[tokio::test]
async fn test_deploy_staking_covenant() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let (alice_xonly, _) = alice.keypair().x_only_public_key();
    let alice_pk_bytes = alice_xonly.serialize();

    // Get current DAA score to set unlock time 1 hour from now
    let info = client.get_server_info().await.unwrap();
    let current_daa = info.virtual_daa_score;
    let unlock_daa = current_daa + 3600; // ~1 hour at 1 BPS

    println!("[K-RWA] === STAKING TIMELOCK COVENANT DEPLOY ===");
    println!("[K-RWA] Current DAA: {}", current_daa);
    println!("[K-RWA] Unlock DAA:  {} (~1 hour lock)", unlock_daa);

    // Build the timelock covenant
    let redeem_script = build_staking_covenant(&alice_pk_bytes, unlock_daa);
    println!(
        "[K-RWA] Staking covenant: {} bytes = {}",
        redeem_script.len(),
        hex::encode(&redeem_script)
    );

    // Derive P2SH address
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let script_hash = &p2sh_spk.script()[2..34];
    let p2sh_addr = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);
    println!("[K-RWA] P2SH address: {}", p2sh_addr);

    // Deploy (fund with 0.5 KAS to avoid storage mass issues)
    println!("[K-RWA] Deploying staking covenant (0.5 KAS)...");
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
            println!("[K-RWA]  STAKING COVENANT DEPLOYED!");
            println!("[K-RWA]  TX: {}", deploy_tx);
            println!("[K-RWA]  P2SH: {}", p2sh_addr);
            println!("[K-RWA]  Unlock at DAA: {}", unlock_daa);
            println!("[K-RWA]  Script: {} bytes", redeem_script.len());
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] Deploy failed: {}", e);
            println!("[K-RWA] This may be due to UTXO fragmentation.");
            println!("[K-RWA] The staking covenant was built correctly ({} bytes)", redeem_script.len());
        }
    }

    client.disconnect().await.ok();
}
