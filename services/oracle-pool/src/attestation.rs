// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Multisig attestation generation.
//! 2-of-3 testnet oracle keys sign price data for on-chain consumption.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;
use crate::oracle::AggregatedPrice;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("[K-RWA] Attestation signing failed: {0}")]
    SigningFailed(String),
    #[error("[K-RWA] Insufficient signatures: need {needed}, have {have}")]
    InsufficientSignatures { needed: usize, have: usize },
}

/// A signed oracle attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// The attested price data
    pub price: AggregatedPrice,
    /// Ed25519 signatures from oracle signers (hex)
    pub signatures: Vec<String>,
    /// Public keys of the signers (hex)
    pub signer_pubkeys: Vec<String>,
    /// Minimum required signatures (2 of 3)
    pub threshold: usize,
}

/// Generate a multisig attestation for a price
pub fn create_attestation(
    price: AggregatedPrice,
    _signer_keys: &[Vec<u8>],
) -> Result<Attestation, AttestationError> {
    info!(
        "{} Creating 2-of-3 multisig attestation for {} @ ${:.2}",
        LOG_PREFIX, price.asset_id, price.price_usd
    );

    // TODO: Sign price data with 2+ of the 3 oracle keys (Ed25519)
    // REPLACE_WITH_TESTNET_WALLET — oracle signer keys from .env
    let attestation = Attestation {
        price,
        signatures: vec![
            "sig1_placeholder".to_string(),
            "sig2_placeholder".to_string(),
        ],
        signer_pubkeys: vec![
            "pubkey1_placeholder".to_string(),
            "pubkey2_placeholder".to_string(),
        ],
        threshold: 2,
    };

    info!("{} Attestation created with {} signatures", LOG_PREFIX, attestation.signatures.len());
    Ok(attestation)
}

/// Verify an attestation has sufficient valid signatures
pub fn verify_attestation(attestation: &Attestation) -> Result<bool, AttestationError> {
    info!("{} Verifying oracle attestation", LOG_PREFIX);

    if attestation.signatures.len() < attestation.threshold {
        return Err(AttestationError::InsufficientSignatures {
            needed: attestation.threshold,
            have: attestation.signatures.len(),
        });
    }

    // TODO: Verify each Ed25519 signature against the corresponding pubkey
    Ok(true)
}
