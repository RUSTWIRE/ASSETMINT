// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Kaspa Testnet-12 WebSocket RPC client.
//! Connects to ws://tn12-node.kaspa.com:17210 for UTXO queries and tx broadcast.

use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("[K-RWA] Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("[K-RWA] RPC error: {0}")]
    RpcError(String),
}

/// Kaspa Testnet-12 RPC client
pub struct KaspaClient {
    endpoint: String,
}

impl KaspaClient {
    /// Create a new client targeting Testnet-12
    ///
    /// # Arguments
    /// * `endpoint` - WebSocket RPC URL (must be TN12)
    pub fn new(endpoint: &str) -> Self {
        info!("{} Initializing Kaspa client: {}", LOG_PREFIX, endpoint);
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    /// Connect to Testnet-12 and verify connectivity
    pub async fn connect(&self) -> Result<(), ClientError> {
        info!("{} Connecting to {}", LOG_PREFIX, self.endpoint);
        // TODO: Implement WebSocket connection via rusty-kaspa RPC
        // Uses kaspa-rpc-core::client::RpcClient
        Ok(())
    }

    /// Query current block count (DAG blue score)
    pub async fn get_block_count(&self) -> Result<u64, ClientError> {
        info!("{} Querying block count", LOG_PREFIX);
        // TODO: Implement via getBlockCount RPC
        Ok(0)
    }

    /// Get UTXOs for an address
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<Utxo>, ClientError> {
        info!("{} Querying UTXOs for {}", LOG_PREFIX, address);
        // TODO: Implement via getUtxosByAddresses RPC
        Ok(vec![])
    }

    /// Broadcast a signed transaction
    pub async fn broadcast_tx(&self, tx_hex: &str) -> Result<String, ClientError> {
        info!("{} Broadcasting transaction", LOG_PREFIX);
        // TODO: Implement via submitTransaction RPC
        Ok(String::new())
    }
}

/// Unspent transaction output
#[derive(Debug, Clone)]
pub struct Utxo {
    pub txid: String,
    pub index: u32,
    pub amount: u64,
    pub script_pubkey: String,
}
