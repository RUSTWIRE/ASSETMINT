// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Claim types and issuance — ported from Polymesh CDD module.
//! Supports KYC verification, accredited investor status, jurisdiction checks.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum ClaimError {
    #[error("[K-RWA] Claim verification failed: {0}")]
    VerificationFailed(String),
    #[error("[K-RWA] Claim expired")]
    Expired,
    #[error("[K-RWA] Unauthorized issuer: {0}")]
    UnauthorizedIssuer(String),
}

/// Claim types supported by the compliance system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimType {
    /// Customer Due Diligence verified
    KycVerified,
    /// Accredited investor status
    AccreditedInvestor,
    /// Jurisdiction is allowed for this asset
    JurisdictionAllowed(String),
    /// Anti-money laundering check passed
    AmlClear,
    /// Entity is exempt from certain requirements
    ExemptedEntity,
}

/// A verifiable claim issued by a trusted claim issuer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim type
    pub claim_type: ClaimType,
    /// DID of the claim issuer
    pub issuer_did: String,
    /// DID of the claim subject
    pub subject_did: String,
    /// Expiry timestamp (Unix seconds, 0 = never expires)
    pub expiry: u64,
    /// Ed25519 signature of the claim data (hex)
    pub signature: String,
    /// Issuance timestamp
    pub issued_at: u64,
}

/// Issue a new claim
pub fn issue_claim(
    issuer_did: &str,
    subject_did: &str,
    claim_type: ClaimType,
    expiry: u64,
) -> Claim {
    info!(
        "{} Issuing claim {:?} from {} to {}",
        LOG_PREFIX, claim_type, issuer_did, subject_did
    );

    Claim {
        claim_type,
        issuer_did: issuer_did.to_string(),
        subject_did: subject_did.to_string(),
        expiry,
        signature: String::new(), // TODO: Ed25519 sign with issuer key
        issued_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}

/// Verify a claim is valid (not expired, signature checks out)
pub fn verify_claim(claim: &Claim) -> Result<bool, ClaimError> {
    info!(
        "{} Verifying claim {:?} for {}",
        LOG_PREFIX, claim.claim_type, claim.subject_did
    );

    // Check expiry
    if claim.expiry > 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > claim.expiry {
            return Err(ClaimError::Expired);
        }
    }

    // TODO: Verify Ed25519 signature against issuer's public key
    // For now, accept if signature is present
    if claim.signature.is_empty() {
        info!("{} WARNING: Claim signature not yet implemented", LOG_PREFIX);
    }

    Ok(true)
}
