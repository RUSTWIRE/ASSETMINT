// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM KRC-20 inscription token.
//! Deploy, mint, and transfer operations via Kaspa inscriptions.
//! Inscription data is embedded in transaction witness/OP_RETURN.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("[K-RWA] Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("[K-RWA] Amount exceeds mint limit: {amount} > {limit}")]
    ExceedsMintLimit { amount: u64, limit: u64 },
    #[error("[K-RWA] Serialization error: {0}")]
    SerializationError(String),
}

/// ASTM token configuration
pub const ASTM_TICKER: &str = "ASTM";
pub const ASTM_MAX_SUPPLY: u64 = 1_000_000_000;
pub const ASTM_MINT_LIMIT: u64 = 1_000;
pub const ASTM_DECIMALS: u8 = 8;

/// KRC-20 inscription operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Krc20Inscription {
    /// Protocol identifier
    pub p: String,
    /// Operation type
    pub op: String,
    /// Token ticker
    pub tick: String,
    /// Max supply (deploy only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
    /// Per-mint limit (deploy only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lim: Option<String>,
    /// Amount (mint/transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amt: Option<String>,
    /// Recipient address (transfer only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Decimals (deploy only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dec: Option<String>,
}

/// A raw inscription-bearing transaction ready for broadcast
#[derive(Debug, Clone)]
pub struct InscriptionTx {
    /// The inscription data as JSON bytes
    pub inscription_data: Vec<u8>,
    /// SHA-256 commitment hash of the inscription
    pub commitment_hash: [u8; 32],
    /// The parsed inscription
    pub inscription: Krc20Inscription,
}

/// Build an inscription transaction envelope
fn build_inscription_tx(inscription: Krc20Inscription) -> InscriptionTx {
    let data = serde_json::to_vec(&inscription).expect("inscription serialization");
    let commitment_hash: [u8; 32] = Sha256::digest(&data).into();
    info!(
        "{} Inscription built: op={}, commitment={}",
        LOG_PREFIX,
        inscription.op,
        hex::encode(commitment_hash)
    );
    InscriptionTx {
        inscription_data: data,
        commitment_hash,
        inscription,
    }
}

/// Create the ASTM token deploy inscription
pub fn deploy_inscription() -> InscriptionTx {
    info!("{} Creating ASTM KRC-20 deploy inscription", LOG_PREFIX);
    build_inscription_tx(Krc20Inscription {
        p: "krc-20".into(),
        op: "deploy".into(),
        tick: ASTM_TICKER.into(),
        max: Some(ASTM_MAX_SUPPLY.to_string()),
        lim: Some(ASTM_MINT_LIMIT.to_string()),
        amt: None,
        to: None,
        dec: Some(ASTM_DECIMALS.to_string()),
    })
}

/// Create a mint inscription (enforces per-mint limit)
pub fn mint_inscription(amount: u64) -> Result<InscriptionTx, TokenError> {
    if amount > ASTM_MINT_LIMIT {
        return Err(TokenError::ExceedsMintLimit {
            amount,
            limit: ASTM_MINT_LIMIT,
        });
    }
    info!(
        "{} Creating ASTM mint inscription: amount={}",
        LOG_PREFIX, amount
    );
    Ok(build_inscription_tx(Krc20Inscription {
        p: "krc-20".into(),
        op: "mint".into(),
        tick: ASTM_TICKER.into(),
        max: None,
        lim: None,
        amt: Some(amount.to_string()),
        to: None,
        dec: None,
    }))
}

/// Create a transfer inscription
pub fn transfer_inscription(to: &str, amount: u64) -> InscriptionTx {
    info!(
        "{} Creating ASTM transfer inscription: to={}, amount={}",
        LOG_PREFIX, to, amount
    );
    build_inscription_tx(Krc20Inscription {
        p: "krc-20".into(),
        op: "transfer".into(),
        tick: ASTM_TICKER.into(),
        max: None,
        lim: None,
        amt: Some(amount.to_string()),
        to: Some(to.to_string()),
        dec: None,
    })
}

/// Validate an inscription JSON blob
pub fn validate_inscription(json_bytes: &[u8]) -> Result<Krc20Inscription, TokenError> {
    let inscription: Krc20Inscription = serde_json::from_slice(json_bytes)
        .map_err(|e| TokenError::SerializationError(e.to_string()))?;

    if inscription.p != "krc-20" {
        return Err(TokenError::InvalidOperation(format!(
            "unknown protocol: {}",
            inscription.p
        )));
    }

    match inscription.op.as_str() {
        "deploy" | "mint" | "transfer" => Ok(inscription),
        other => Err(TokenError::InvalidOperation(format!(
            "unknown op: {}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_inscription() {
        let tx = deploy_inscription();
        assert_eq!(tx.inscription.op, "deploy");
        assert_eq!(tx.inscription.tick, "ASTM");
        assert_eq!(tx.inscription.max, Some("1000000000".to_string()));
        assert_eq!(tx.inscription.lim, Some("1000".to_string()));
        assert_eq!(tx.inscription.dec, Some("8".to_string()));
        assert_ne!(tx.commitment_hash, [0u8; 32]);
    }

    #[test]
    fn test_mint_inscription() {
        let tx = mint_inscription(500).unwrap();
        assert_eq!(tx.inscription.op, "mint");
        assert_eq!(tx.inscription.amt, Some("500".to_string()));
    }

    #[test]
    fn test_mint_exceeds_limit() {
        assert!(mint_inscription(2000).is_err());
    }

    #[test]
    fn test_transfer_inscription() {
        let tx = transfer_inscription("kaspatest:addr123", 100);
        assert_eq!(tx.inscription.op, "transfer");
        assert_eq!(tx.inscription.to, Some("kaspatest:addr123".to_string()));
        assert_eq!(tx.inscription.amt, Some("100".to_string()));
    }

    #[test]
    fn test_validate_roundtrip() {
        let tx = deploy_inscription();
        let parsed = validate_inscription(&tx.inscription_data).unwrap();
        assert_eq!(parsed, tx.inscription);
    }

    #[test]
    fn test_validate_invalid_protocol() {
        let json = br#"{"p":"brc-20","op":"deploy","tick":"X"}"#;
        assert!(validate_inscription(json).is_err());
    }

    #[test]
    fn test_validate_invalid_op() {
        let json = br#"{"p":"krc-20","op":"burn","tick":"X"}"#;
        assert!(validate_inscription(json).is_err());
    }
}
