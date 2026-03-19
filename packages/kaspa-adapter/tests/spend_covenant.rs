// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! P2SH covenant spending tests — invoke SilverScript contract entrypoints
//! by spending their P2SH UTXOs on Kaspa Testnet-12.
//!
//! These tests require a live TN12 connection and funded P2SH UTXOs.
//!
//! Run: cargo test -p kaspa-adapter --test spend_covenant -- --nocapture --test-threads=1

use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::script::load_contract_json;
use kaspa_adapter::wallet::Wallet;
use kaspa_consensus_core::tx::TransactionOutput;
use kaspa_txscript::pay_to_address_script;

const RPC_URL: &str = "ws://127.0.0.1:17210";

// Owner wallet — this must be the key whose public key was baked into the
// Clawback contract's `owner` parameter at compile time.
// Bob's key (used for deployment)
const OWNER_KEY: &str = "37df3703a12b02b3d0a16efa38ca53cda2ee5e9eaa3b8861dc8e04383fb3fecc";

/// Miner fee in sompis (must match the contract's MINER_FEE constant)
const MINER_FEE: u64 = 1000;

/// Invoke the Clawback contract's `ownerSpend` entrypoint.
///
/// The ownerSpend path requires:
///   - ownerSig: Schnorr signature from the owner (verified by checkSig)
///   - recipientPk: 32-byte x-only public key of the recipient
///
/// The covenant enforces:
///   - checkSig(ownerSig, owner)
///   - tx.outputs[0].scriptPubKey == P2PK(recipientPk)
///   - tx.outputs[0].value >= inputValue - MINER_FEE
#[tokio::test]
async fn test_clawback_owner_spend() {
    println!("[K-RWA] === Clawback ownerSpend covenant invocation ===");

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    // Load the Clawback contract
    let contract = load_contract_json("../../contracts/silverscript/clawback.json")
        .expect("Failed to load clawback.json");

    println!(
        "[K-RWA] Contract: {} ({} bytes), P2SH: {}",
        contract.contract_name,
        contract.redeem_script.len(),
        contract.p2sh_address
    );

    // Check that the P2SH address has UTXOs
    let p2sh_addr = contract.p2sh_address.to_string();
    let utxos = client.get_utxos(&p2sh_addr).await.expect("Failed to query UTXOs");
    if utxos.is_empty() {
        println!("[K-RWA] No UTXOs at P2SH address — contract not deployed or already spent");
        println!("[K-RWA] Deploy first with: cargo test -p kaspa-adapter --test deploy_single test_deploy_clawback");
        client.disconnect().await.ok();
        return;
    }

    let utxo = &utxos[0];
    let input_amount = utxo.amount;
    println!(
        "[K-RWA] Found P2SH UTXO: {}:{} = {} sompis",
        utxo.txid, utxo.index, input_amount
    );

    // Owner wallet
    let owner_wallet = Wallet::from_hex(OWNER_KEY).expect("Failed to load owner wallet");
    let owner_addr = owner_wallet.address_string();
    println!("[K-RWA] Owner address: {}", owner_addr);

    // Recipient = owner (send back to ourselves for testing)
    let recipient_addr: kaspa_addresses::Address = owner_addr.as_str().try_into()
        .expect("Failed to parse recipient address");
    let (recipient_xonly, _) = owner_wallet.keypair().x_only_public_key();
    let recipient_pk_bytes = recipient_xonly.serialize().to_vec();

    println!("[K-RWA] Recipient pubkey: {}", hex::encode(&recipient_pk_bytes));

    // Build the output that satisfies the covenant:
    //   tx.outputs[0].scriptPubKey == P2PK(recipientPk)
    //   tx.outputs[0].value >= inputValue - MINER_FEE
    let output_amount = input_amount - MINER_FEE;
    let output_script = pay_to_address_script(&recipient_addr);
    let outputs = vec![TransactionOutput::new(output_amount, output_script)];

    // Build witness params for ownerSpend(sig ownerSig, pubkey recipientPk):
    //   - ownerSig: empty vec = placeholder, will be replaced with computed Schnorr sig
    //   - recipientPk: 32-byte x-only public key
    let witness_params = vec![
        vec![],               // Placeholder for ownerSig (computed by spend_p2sh)
        recipient_pk_bytes,   // recipientPk
    ];

    // Parse the UTXO txid
    let txid: kaspa_consensus_core::tx::TransactionId = utxo.txid.parse()
        .expect("Failed to parse UTXO txid");

    // Invoke the covenant!
    // sig_op_count = 1 (one checkSig in ownerSpend)
    match client.spend_p2sh(
        txid,
        utxo.index,
        utxo.amount,
        contract.script_public_key.clone(),
        &contract.redeem_script,
        witness_params,
        outputs,
        1, // sig_op_count
        owner_wallet.keypair(),
    ).await {
        Ok(tx_id) => {
            println!("[K-RWA] ========================================");
            println!("[K-RWA]  COVENANT SPEND SUCCEEDED!");
            println!("[K-RWA]  TX: {}", tx_id);
            println!("[K-RWA]  Contract: {}", contract.contract_name);
            println!("[K-RWA]  Entrypoint: ownerSpend");
            println!("[K-RWA]  Input: {} sompis", input_amount);
            println!("[K-RWA]  Output: {} sompis to {}", output_amount, owner_addr);
            println!("[K-RWA] ========================================");
        }
        Err(e) => {
            println!("[K-RWA] Covenant spend failed: {}", e);
            println!("[K-RWA] This may indicate:");
            println!("[K-RWA]   - Owner pubkey mismatch (contract was compiled with different owner)");
            println!("[K-RWA]   - UTXO already spent");
            println!("[K-RWA]   - Script execution error (check witness format)");
            client.disconnect().await.ok();
            panic!("Covenant spend failed: {}", e);
        }
    }

    client.disconnect().await.expect("Failed to disconnect");
}

/// Test using the higher-level spend_contract() helper
#[tokio::test]
async fn test_clawback_owner_spend_helper() {
    println!("[K-RWA] === Clawback ownerSpend via spend_contract() ===");

    let client = KaspaClient::new(RPC_URL).expect("Failed to create client");
    client.connect().await.expect("Failed to connect");

    let contract = load_contract_json("../../contracts/silverscript/clawback.json")
        .expect("Failed to load clawback.json");

    let p2sh_addr = contract.p2sh_address.to_string();
    let utxos = client.get_utxos(&p2sh_addr).await.expect("Failed to query UTXOs");
    if utxos.is_empty() {
        println!("[K-RWA] No UTXOs — skipping (deploy first)");
        client.disconnect().await.ok();
        return;
    }

    let owner_wallet = Wallet::from_hex(OWNER_KEY).expect("Failed to load owner wallet");
    let owner_addr = owner_wallet.address_string();

    let (recipient_xonly, _) = owner_wallet.keypair().x_only_public_key();
    let recipient_pk_bytes = recipient_xonly.serialize().to_vec();

    let output_amount = utxos[0].amount - MINER_FEE;

    // witness_params: [sig_placeholder, recipientPk]
    let witness_params = vec![
        vec![],               // ownerSig placeholder
        recipient_pk_bytes,   // recipientPk
    ];

    match client.spend_contract(
        &contract,
        witness_params,
        &owner_addr,
        output_amount,
        1,
        owner_wallet.keypair(),
    ).await {
        Ok(tx_id) => {
            println!("[K-RWA] spend_contract() succeeded: TX {}", tx_id);
        }
        Err(e) => {
            println!("[K-RWA] spend_contract() failed: {}", e);
            client.disconnect().await.ok();
            panic!("spend_contract() failed: {}", e);
        }
    }

    client.disconnect().await.expect("Failed to disconnect");
}
