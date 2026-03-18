// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! UTXO transaction builder for Kaspa Testnet-12.
//! Handles coin selection, covenant-spending tx construction,
//! witness data attachment, and fee estimation.

use thiserror::Error;
use tracing::info;

use crate::client::Utxo;
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum TxBuilderError {
    #[error("[K-RWA] Insufficient funds: need {needed}, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },
    #[error("[K-RWA] Invalid script: {0}")]
    InvalidScript(String),
    #[error("[K-RWA] Build failed: {0}")]
    BuildFailed(String),
}

/// Transaction output specification
#[derive(Debug, Clone)]
pub struct TxOutput {
    /// Destination address or P2SH script hash
    pub address: String,
    /// Amount in sompis (1 KAS = 100_000_000 sompis)
    pub amount: u64,
    /// Optional OP_RETURN data (DKG UAL, schema metadata)
    pub op_return_data: Option<Vec<u8>>,
}

/// Witness data for covenant spending
#[derive(Debug, Clone)]
pub struct WitnessData {
    /// Signature(s)
    pub signatures: Vec<Vec<u8>>,
    /// ZK proof bytes (Groth16)
    pub zk_proof: Option<Vec<u8>>,
    /// Merkle proof path
    pub merkle_proof: Option<Vec<Vec<u8>>>,
    /// Oracle attestation
    pub oracle_attestation: Option<Vec<u8>>,
}

/// Builds transactions for Kaspa Testnet-12
pub struct TransactionBuilder {
    inputs: Vec<Utxo>,
    outputs: Vec<TxOutput>,
    witness: Option<WitnessData>,
    fee_rate: u64,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        info!("{} Creating new transaction builder", LOG_PREFIX);
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            witness: None,
            fee_rate: 1, // 1 sompi/byte default
        }
    }

    /// Add UTXOs as inputs (coin selection: largest-first)
    pub fn add_inputs(&mut self, utxos: Vec<Utxo>) -> &mut Self {
        let mut sorted = utxos;
        sorted.sort_by(|a, b| b.amount.cmp(&a.amount));
        self.inputs.extend(sorted);
        self
    }

    /// Add an output
    pub fn add_output(&mut self, output: TxOutput) -> &mut Self {
        self.outputs.push(output);
        self
    }

    /// Attach witness data (for covenant spending)
    pub fn set_witness(&mut self, witness: WitnessData) -> &mut Self {
        self.witness = Some(witness);
        self
    }

    /// Estimate transaction fee
    pub fn estimate_fee(&self) -> u64 {
        // TODO: Calculate based on tx size
        // Target: ≤0.001 KAS per transfer
        let estimated_size = 250 + (self.inputs.len() * 148) + (self.outputs.len() * 34);
        (estimated_size as u64) * self.fee_rate
    }

    /// Build and serialize the transaction
    pub fn build(&self) -> Result<Vec<u8>, TxBuilderError> {
        info!("{} Building transaction with {} inputs and {} outputs",
            LOG_PREFIX, self.inputs.len(), self.outputs.len());

        let total_in: u64 = self.inputs.iter().map(|u| u.amount).sum();
        let total_out: u64 = self.outputs.iter().map(|o| o.amount).sum();
        let fee = self.estimate_fee();

        if total_in < total_out + fee {
            return Err(TxBuilderError::InsufficientFunds {
                needed: total_out + fee,
                available: total_in,
            });
        }

        // TODO: Serialize using kaspa-consensus-core Transaction type
        // Include witness data for covenant spending paths
        info!("{} Transaction built: {} inputs, {} outputs, fee={} sompis",
            LOG_PREFIX, self.inputs.len(), self.outputs.len(), fee);

        Ok(vec![])
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
