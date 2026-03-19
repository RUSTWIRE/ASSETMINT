// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Multisig attestation generation.
//! 2-of-3 testnet oracle keys sign price data for on-chain consumption.
//! Uses Ed25519 signatures for oracle key signing.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::oracle::AggregatedPrice;
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("[K-RWA] Attestation signing failed: {0}")]
    SigningFailed(String),
    #[error("[K-RWA] Insufficient signatures: need {needed}, have {have}")]
    InsufficientSignatures { needed: usize, have: usize },
    #[error("[K-RWA] Invalid signature from signer {index}: {reason}")]
    InvalidSignature { index: usize, reason: String },
}

/// Required threshold: 2-of-3
pub const ATTESTATION_THRESHOLD: usize = 2;
/// Total oracle signers
pub const TOTAL_SIGNERS: usize = 3;

/// A signed oracle attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// The attested price data
    pub price: AggregatedPrice,
    /// Ed25519 signatures from oracle signers (hex-encoded)
    pub signatures: Vec<String>,
    /// Verifying keys of the signers (hex-encoded)
    pub signer_pubkeys: Vec<String>,
    /// Minimum required signatures
    pub threshold: usize,
    /// SHA-256 hash of the attested data (for on-chain reference)
    pub data_hash: String,
}

/// An oracle signer (holds Ed25519 key pair)
pub struct OracleSigner {
    pub name: String,
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl OracleSigner {
    /// Create an oracle signer from a 32-byte seed
    /// REPLACE_WITH_TESTNET_WALLET — use env-loaded keys in production
    pub fn new(name: &str, seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        info!(
            "{} Oracle signer created: {} (vk={})",
            LOG_PREFIX,
            name,
            hex::encode(verifying_key.as_bytes())
        );
        Self {
            name: name.to_string(),
            signing_key,
            verifying_key,
        }
    }

    /// Sign attestation data
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let sig = self.signing_key.sign(data);
        sig.to_bytes().to_vec()
    }
}

/// Build canonical attestation data for signing
pub fn build_attestation_data(price: &AggregatedPrice) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(price.asset_id.as_bytes());
    hasher.update(price.price_usd.to_le_bytes());
    hasher.update(price.timestamp.to_le_bytes());
    hasher.update(price.sources_used.to_le_bytes());
    hasher.finalize().to_vec()
}

/// Create a 2-of-3 multisig attestation
pub fn create_attestation(
    price: AggregatedPrice,
    signers: &[&OracleSigner],
) -> Result<Attestation, AttestationError> {
    if signers.len() < ATTESTATION_THRESHOLD {
        return Err(AttestationError::InsufficientSignatures {
            needed: ATTESTATION_THRESHOLD,
            have: signers.len(),
        });
    }

    info!(
        "{} Creating {}-of-{} multisig attestation for {} @ ${:.2}",
        LOG_PREFIX,
        ATTESTATION_THRESHOLD,
        TOTAL_SIGNERS,
        price.asset_id,
        price.price_usd
    );

    let data = build_attestation_data(&price);
    let data_hash = hex::encode(&data);

    let mut signatures = Vec::new();
    let mut signer_pubkeys = Vec::new();

    for signer in signers {
        let sig = signer.sign(&data);
        signatures.push(hex::encode(&sig));
        signer_pubkeys.push(hex::encode(signer.verifying_key.as_bytes()));
    }

    info!(
        "{} Attestation created with {} signatures, hash={}",
        LOG_PREFIX,
        signatures.len(),
        &data_hash[..16]
    );

    Ok(Attestation {
        price,
        signatures,
        signer_pubkeys,
        threshold: ATTESTATION_THRESHOLD,
        data_hash,
    })
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

    let data = build_attestation_data(&attestation.price);
    let mut valid_count = 0;

    for (i, (sig_hex, pk_hex)) in attestation
        .signatures
        .iter()
        .zip(attestation.signer_pubkeys.iter())
        .enumerate()
    {
        let sig_bytes = hex::decode(sig_hex).map_err(|e| AttestationError::InvalidSignature {
            index: i,
            reason: format!("hex decode sig: {}", e),
        })?;

        let pk_bytes = hex::decode(pk_hex).map_err(|e| AttestationError::InvalidSignature {
            index: i,
            reason: format!("hex decode pk: {}", e),
        })?;

        let pk_array: [u8; 32] =
            pk_bytes
                .try_into()
                .map_err(|_| AttestationError::InvalidSignature {
                    index: i,
                    reason: "invalid pubkey length".into(),
                })?;

        let vk = VerifyingKey::from_bytes(&pk_array).map_err(|e| {
            AttestationError::InvalidSignature {
                index: i,
                reason: format!("invalid pubkey: {}", e),
            }
        })?;

        let sig_array: [u8; 64] =
            sig_bytes
                .try_into()
                .map_err(|_| AttestationError::InvalidSignature {
                    index: i,
                    reason: "invalid signature length".into(),
                })?;

        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        match vk.verify(&data, &signature) {
            Ok(()) => valid_count += 1,
            Err(e) => {
                info!(
                    "{} Signature {} invalid: {}",
                    LOG_PREFIX, i, e
                );
            }
        }
    }

    info!(
        "{} Attestation verification: {}/{} valid, threshold={}",
        LOG_PREFIX, valid_count, attestation.signatures.len(), attestation.threshold
    );

    Ok(valid_count >= attestation.threshold)
}

/// Create default testnet oracle signers (deterministic seeds for reproducibility)
/// REPLACE_WITH_TESTNET_WALLET — use env-loaded keys in production
pub fn create_testnet_signers() -> Vec<OracleSigner> {
    vec![
        OracleSigner::new("oracle-alpha", &[1u8; 32]),
        OracleSigner::new("oracle-beta", &[2u8; 32]),
        OracleSigner::new("oracle-gamma", &[3u8; 32]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_price() -> AggregatedPrice {
        AggregatedPrice {
            price_usd: 250_000.0,
            sources_used: 3,
            sources_rejected: 0,
            timestamp: 1700000000,
            asset_id: "KPROP-NYC-TEST".into(),
        }
    }

    #[test]
    fn test_create_and_verify_attestation() {
        let signers = create_testnet_signers();
        let signer_refs: Vec<&OracleSigner> = signers.iter().take(2).collect();

        let attestation = create_attestation(test_price(), &signer_refs).unwrap();
        assert_eq!(attestation.signatures.len(), 2);
        assert_eq!(attestation.threshold, 2);

        let valid = verify_attestation(&attestation).unwrap();
        assert!(valid, "2-of-3 attestation should verify");
    }

    #[test]
    fn test_all_three_signers() {
        let signers = create_testnet_signers();
        let signer_refs: Vec<&OracleSigner> = signers.iter().collect();

        let attestation = create_attestation(test_price(), &signer_refs).unwrap();
        assert_eq!(attestation.signatures.len(), 3);

        let valid = verify_attestation(&attestation).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_insufficient_signers() {
        let signers = create_testnet_signers();
        let signer_refs: Vec<&OracleSigner> = signers.iter().take(1).collect();
        assert!(create_attestation(test_price(), &signer_refs).is_err());
    }

    #[test]
    fn test_tampered_attestation_fails() {
        let signers = create_testnet_signers();
        let signer_refs: Vec<&OracleSigner> = signers.iter().take(2).collect();

        let mut attestation = create_attestation(test_price(), &signer_refs).unwrap();
        // Tamper with the price
        attestation.price.price_usd = 999_999.0;

        let valid = verify_attestation(&attestation).unwrap();
        assert!(!valid, "Tampered attestation should fail verification");
    }

    #[test]
    fn test_data_hash_deterministic() {
        let price = test_price();
        let d1 = build_attestation_data(&price);
        let d2 = build_attestation_data(&price);
        assert_eq!(d1, d2);
    }
}
