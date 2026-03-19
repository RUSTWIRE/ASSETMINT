// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! UTXO transaction builder for Kaspa Testnet-12.
//! Builds real `Transaction` objects using rusty-kaspa consensus types,
//! handles coin selection, fee estimation, change outputs, and OP_RETURN.

use kaspa_addresses::Address;
use kaspa_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use kaspa_consensus_core::tx::{
    ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint,
    TransactionOutput,
};
use kaspa_rpc_core::RpcUtxoEntry;
use kaspa_txscript::pay_to_address_script;
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum TxBuilderError {
    #[error("[K-RWA] Insufficient funds: need {needed}, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },
    #[error("[K-RWA] Invalid address: {0}")]
    InvalidAddress(String),
    #[error("[K-RWA] Build failed: {0}")]
    BuildFailed(String),
    #[error("[K-RWA] No inputs provided")]
    NoInputs,
    #[error("[K-RWA] No outputs provided")]
    NoOutputs,
}

/// A UTXO ready to be spent (includes the entry data needed for signing)
#[derive(Debug, Clone)]
pub struct SpendableUtxo {
    pub txid: TransactionId,
    pub index: u32,
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
}

/// Simple P2PK transfer parameters
#[derive(Debug)]
pub struct TransferParams {
    pub to_address: Address,
    pub amount: u64,
    pub change_address: Address,
    pub op_return_data: Option<Vec<u8>>,
}

/// Minimum fee per transaction (in sompis)
const MIN_RELAY_FEE: u64 = 1000;

/// Mass per transaction input (approximate)
const MASS_PER_INPUT: u64 = 1000;
/// Mass per transaction output (approximate)
const MASS_PER_OUTPUT: u64 = 1000;
/// Mass per signature operation
const MASS_PER_SIG_OP: u64 = 10000;

/// Maximum inputs per transaction to stay under Kaspa's 1,000,000 storage mass limit.
/// Actual mass per input is ~27,000 (includes script_public_key serialization, sig ops,
/// and output storage mass). Conservative limit: 25 inputs ≈ 675,000 mass.
const MAX_INPUTS: usize = 25;

/// Select UTXOs greedily (largest-first) to cover the target amount + fee.
/// Limits selection to MAX_INPUTS to avoid exceeding Kaspa's storage mass limit.
pub fn select_utxos(
    utxos: &[SpendableUtxo],
    target: u64,
) -> Result<(Vec<SpendableUtxo>, u64), TxBuilderError> {
    let mut sorted: Vec<SpendableUtxo> = utxos.to_vec();
    sorted.sort_by(|a, b| b.amount.cmp(&a.amount));

    let mut selected = Vec::new();
    let mut total: u64 = 0;

    for utxo in sorted {
        if selected.len() >= MAX_INPUTS {
            break;
        }
        selected.push(utxo.clone());
        total += utxo.amount;

        // Estimate fee based on current selection
        let fee = estimate_fee(selected.len(), 2); // 2 outputs: dest + change
        if total >= target + fee {
            info!(
                "{} Selected {} UTXOs, total={} sompis, target={}, fee={}",
                LOG_PREFIX,
                selected.len(),
                total,
                target,
                fee
            );
            return Ok((selected, fee));
        }
    }

    Err(TxBuilderError::InsufficientFunds {
        needed: target + estimate_fee(selected.len(), 2),
        available: total,
    })
}

/// Estimate the fee for a transaction
fn estimate_fee(num_inputs: usize, num_outputs: usize) -> u64 {
    let mass = (num_inputs as u64 * MASS_PER_INPUT)
        + (num_outputs as u64 * MASS_PER_OUTPUT)
        + (num_inputs as u64 * MASS_PER_SIG_OP);
    // fee = mass * minimum_fee_rate / 1000, minimum MIN_RELAY_FEE
    std::cmp::max(mass, MIN_RELAY_FEE)
}

/// Build a P2PK transfer transaction.
///
/// Returns (Transaction, Vec<(TransactionOutpoint, RpcUtxoEntry)>) — the
/// unsigned transaction and the UTXO entries needed for signing context.
pub fn build_transfer(
    utxos: &[SpendableUtxo],
    params: &TransferParams,
) -> Result<(Transaction, Vec<(TransactionOutpoint, RpcUtxoEntry)>), TxBuilderError> {
    if utxos.is_empty() {
        return Err(TxBuilderError::NoInputs);
    }

    // Select UTXOs
    let (selected, fee) = select_utxos(utxos, params.amount)?;

    let total_in: u64 = selected.iter().map(|u| u.amount).sum();
    let change = total_in - params.amount - fee;

    info!(
        "{} Building transfer: {} sompis to {}, fee={}, change={}",
        LOG_PREFIX, params.amount, params.to_address, fee, change
    );

    // Build inputs
    let mut inputs = Vec::new();
    let mut utxo_entries = Vec::new();

    for utxo in &selected {
        let outpoint = TransactionOutpoint::new(utxo.txid, utxo.index);

        inputs.push(TransactionInput::new(
            outpoint.clone(),
            vec![], // Empty sig script — filled by signing
            0,      // sequence
            1,      // sig_op_count (1 for P2PK)
        ));

        // Build the UTXO entry for signing context
        let entry = RpcUtxoEntry {
            amount: utxo.amount,
            script_public_key: utxo.script_public_key.clone(),
            block_daa_score: 0,
            is_coinbase: false,
            covenant_id: None,
        };
        utxo_entries.push((outpoint, entry));
    }

    // Build outputs
    let mut outputs = Vec::new();

    // Destination output
    let dest_script = pay_to_address_script(&params.to_address);
    outputs.push(TransactionOutput::new(params.amount, dest_script));

    // Change output (if any)
    if change > 0 {
        let change_script = pay_to_address_script(&params.change_address);
        outputs.push(TransactionOutput::new(change, change_script));
    }

    // OP_RETURN output (if any)
    if let Some(data) = &params.op_return_data {
        let mut script_bytes = vec![0x6a]; // OP_RETURN
        script_bytes.extend(data);
        let op_return_script = ScriptPublicKey::new(0, script_bytes.into());
        outputs.push(TransactionOutput::new(0, op_return_script));
    }

    // Build the transaction
    let tx = Transaction::new(
        0,                    // version
        inputs,               // inputs
        outputs,              // outputs
        0,                    // lock_time
        SUBNETWORK_ID_NATIVE, // subnetwork_id
        0,                    // gas
        vec![],               // payload
    );

    info!(
        "{} Transaction built: id={}, {} inputs, {} outputs",
        LOG_PREFIX,
        tx.id(),
        tx.inputs.len(),
        tx.outputs.len()
    );

    Ok((tx, utxo_entries))
}
