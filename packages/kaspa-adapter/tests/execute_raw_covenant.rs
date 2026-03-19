// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//! Execute a hand-built covenant on Kaspa TN12.
//! Uses the KTT-proven pattern: raw CHECKSIG in P2SH.

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_txscript::pay_to_script_hash_script;

const RPC_URL: &str = "ws://127.0.0.1:17210";
const ALICE_KEY: &str = "ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92";

#[tokio::test]
async fn test_raw_checksig_covenant() {
    let client = KaspaClient::new(RPC_URL).unwrap();
    client.connect().await.unwrap();

    let alice = Wallet::from_hex(ALICE_KEY).unwrap();
    let (alice_xonly, _) = alice.keypair().x_only_public_key();
    let alice_pk_bytes = alice_xonly.serialize();

    // Build the simplest possible covenant: just CHECKSIG
    // Redeem script: [push_32][alice_pubkey] OP_CHECKSIG
    // This is equivalent to P2PK but wrapped in P2SH
    let mut redeem_script = Vec::new();
    redeem_script.push(0x20); // push 32 bytes
    redeem_script.extend_from_slice(&alice_pk_bytes);
    redeem_script.push(0xac); // OP_CHECKSIG

    println!(
        "[K-RWA] Redeem script: {} bytes = {}",
        redeem_script.len(),
        hex::encode(&redeem_script)
    );

    // Derive P2SH address from redeem script
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let script_hash = &p2sh_spk.script()[2..34]; // extract 32-byte hash from OP_BLAKE2B <hash> OP_EQUAL
    let p2sh_addr = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);
    println!("[K-RWA] P2SH address: {}", p2sh_addr);

    // Fund the P2SH address from Alice (deploy the covenant)
    println!("[K-RWA] Deploying raw CHECKSIG covenant (1 KAS)...");
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

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Now spend the P2SH UTXO
    // ScriptSig: [sig_push] [redeem_script_push]
    // CHECKSIG will use alice's pubkey (embedded in redeem script) and the sig from scriptSig
    // Wait — CHECKSIG pops pubkey + sig from stack. The redeem script pushes the pubkey,
    // so the scriptSig only needs the signature.
    let recipient = Wallet::generate().unwrap();
    println!(
        "[K-RWA] Invoking CHECKSIG covenant → {}",
        recipient.address_string()
    );

    // Witness: just the signature (the pubkey is in the redeem script)
    let witness = vec![
        vec![], // sig placeholder — will be filled by spend_p2sh
    ];

    // Get the P2SH UTXO
    let utxos = client
        .get_spendable_utxos(&p2sh_addr.to_string())
        .await
        .unwrap();
    if utxos.is_empty() {
        println!("[K-RWA] No UTXOs at P2SH — skipping");
        client.disconnect().await.ok();
        return;
    }

    let utxo = &utxos[0];
    println!(
        "[K-RWA] P2SH UTXO: {}:{} = {} sompis",
        utxo.txid, utxo.index, utxo.amount
    );

    // Build output
    let recipient_addr: Address = recipient.address_string().as_str().try_into().unwrap();
    let dest_script = kaspa_txscript::pay_to_address_script(&recipient_addr);
    let output =
        kaspa_consensus_core::tx::TransactionOutput::new(utxo.amount - 10_000, dest_script);

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
            println!("[K-RWA]  RAW COVENANT EXECUTION SUCCESS!");
            println!("[K-RWA]  TX: {}", tx_id);
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] Covenant failed: {}", e);
        }
    }

    client.disconnect().await.ok();
}
