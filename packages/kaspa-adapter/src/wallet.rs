// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Dynamic testnet wallet generation.
//! NEVER hardcode keys — generate dynamically for Testnet-12.
//! REPLACE_WITH_TESTNET_WALLET

use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("[K-RWA] Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("[K-RWA] Signing failed: {0}")]
    SigningFailed(String),
}

/// Testnet wallet — generates ephemeral keypairs for TN12
pub struct Wallet {
    // REPLACE_WITH_TESTNET_WALLET — never hardcode
    private_key: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
    address: Option<String>,
}

impl Wallet {
    /// Generate a new random testnet wallet
    pub fn generate() -> Result<Self, WalletError> {
        info!("{} Generating new testnet wallet", LOG_PREFIX);
        // TODO: Use kaspa-wasm PrivateKey::generate() or ed25519-dalek
        Ok(Self {
            private_key: None,
            public_key: None,
            address: None,
        })
    }

    /// Import from environment variable (KASPA_PRIVATE_KEY)
    pub fn from_env() -> Result<Self, WalletError> {
        info!("{} Loading wallet from environment", LOG_PREFIX);
        let key = std::env::var("KASPA_PRIVATE_KEY")
            .map_err(|e| WalletError::KeyGenFailed(format!("KASPA_PRIVATE_KEY not set: {}", e)))?;
        if key.is_empty() {
            return Err(WalletError::KeyGenFailed(
                "KASPA_PRIVATE_KEY is empty — REPLACE_WITH_TESTNET_WALLET".into(),
            ));
        }
        // TODO: Parse hex key and derive public key + address
        Ok(Self {
            private_key: Some(hex::decode(&key).map_err(|e| WalletError::KeyGenFailed(e.to_string()))?),
            public_key: None,
            address: None,
        })
    }

    /// Get the Kaspa address (testnet format)
    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    /// Sign a transaction hash
    pub fn sign(&self, _tx_hash: &[u8]) -> Result<Vec<u8>, WalletError> {
        info!("{} Signing transaction", LOG_PREFIX);
        // TODO: Implement schnorr signing via kaspa-wasm
        Err(WalletError::SigningFailed("Not yet implemented".into()))
    }
}
