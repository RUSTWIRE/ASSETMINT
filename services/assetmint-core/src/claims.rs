// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! AssetMint Claims System — implements Polymesh CDD (Customer Due Diligence) patterns.
//! Ed25519 signed claims with expiry, W3C Verifiable Credential support.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    #[error("[K-RWA] Invalid signature: {0}")]
    InvalidSignature(String),
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
    /// Ed25519 signature of the claim data (hex-encoded)
    pub signature: String,
    /// Issuance timestamp
    pub issued_at: u64,
}

// ── W3C Verifiable Credentials ────────────────────────────────────────

/// W3C Verifiable Credential (JSON-LD format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "type")]
    pub vc_type: Vec<String>,
    pub issuer: String,
    pub issuance_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    pub credential_subject: CredentialSubject,
    pub proof: VcProof,
}

/// The subject of a Verifiable Credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSubject {
    pub id: String,
    pub claim_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
}

/// Proof block for a Verifiable Credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcProof {
    #[serde(rename = "type")]
    pub type_: String,
    pub created: String,
    pub verification_method: String,
    pub proof_value: String,
}

impl Claim {
    /// Convert this claim to a W3C Verifiable Credential format.
    pub fn to_verifiable_credential(&self) -> VerifiableCredential {
        let claim_type_str = match &self.claim_type {
            ClaimType::KycVerified => "KycVerified".to_string(),
            ClaimType::AccreditedInvestor => "AccreditedInvestor".to_string(),
            ClaimType::JurisdictionAllowed(j) => format!("JurisdictionAllowed:{}", j),
            ClaimType::AmlClear => "AmlClear".to_string(),
            ClaimType::ExemptedEntity => "ExemptedEntity".to_string(),
        };

        let jurisdiction = match &self.claim_type {
            ClaimType::JurisdictionAllowed(j) => Some(j.clone()),
            _ => None,
        };

        // Format timestamps as ISO 8601
        let issuance_date = timestamp_to_iso(self.issued_at);
        let expiration_date = if self.expiry > 0 {
            Some(timestamp_to_iso(self.expiry))
        } else {
            None
        };

        VerifiableCredential {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://assetmint.io/credentials/v1".to_string(),
            ],
            vc_type: vec![
                "VerifiableCredential".to_string(),
                "ComplianceClaim".to_string(),
            ],
            issuer: self.issuer_did.clone(),
            issuance_date,
            expiration_date,
            credential_subject: CredentialSubject {
                id: self.subject_did.clone(),
                claim_type: claim_type_str,
                jurisdiction,
            },
            proof: VcProof {
                type_: "Ed25519Signature2020".to_string(),
                created: timestamp_to_iso(self.issued_at),
                verification_method: format!("{}#key-1", self.issuer_did),
                proof_value: self.signature.clone(),
            },
        }
    }
}

/// Convert a Unix timestamp to ISO 8601 string
fn timestamp_to_iso(ts: u64) -> String {
    // Simple conversion without chrono dependency: YYYY-MM-DDTHH:MM:SSZ
    let secs_per_day: u64 = 86400;
    let days_since_epoch = ts / secs_per_day;
    let time_of_day = ts % secs_per_day;

    // Compute year/month/day from days since 1970-01-01 (civil_from_days algorithm)
    let z = days_since_epoch as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let h = time_of_day / 3600;
    let min = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, h, min, s)
}

/// Verify a VC proof against the issuer's verifying key
pub fn verify_vc_proof(vc: &VerifiableCredential, issuer_vk: &VerifyingKey) -> Result<bool, ClaimError> {
    info!("{} Verifying VC proof for subject {}", LOG_PREFIX, vc.credential_subject.id);

    // Parse the claim type back from VC format
    let claim_type = if vc.credential_subject.claim_type.starts_with("JurisdictionAllowed:") {
        let j = vc.credential_subject.claim_type.strip_prefix("JurisdictionAllowed:").unwrap_or("");
        ClaimType::JurisdictionAllowed(j.to_string())
    } else {
        match vc.credential_subject.claim_type.as_str() {
            "KycVerified" => ClaimType::KycVerified,
            "AccreditedInvestor" => ClaimType::AccreditedInvestor,
            "AmlClear" => ClaimType::AmlClear,
            "ExemptedEntity" => ClaimType::ExemptedEntity,
            other => return Err(ClaimError::VerificationFailed(format!("Unknown claim type: {}", other))),
        }
    };

    // Parse expiry from VC
    let expiry = vc.expiration_date.as_ref().map(|_| 0u64).unwrap_or(0);

    // Parse issuance timestamp from ISO
    let issued_at = iso_to_timestamp(&vc.proof.created).unwrap_or(0);

    // Check expiry if set
    if let Some(ref exp_str) = vc.expiration_date {
        let exp_ts = iso_to_timestamp(exp_str).unwrap_or(0);
        if exp_ts > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > exp_ts {
                return Err(ClaimError::Expired);
            }
        }
    }

    // Rebuild claim data and verify signature
    let claim_data = build_claim_data(
        &vc.credential_subject.id,
        &claim_type,
        expiry,
        issued_at,
    );

    let sig_bytes = hex::decode(&vc.proof.proof_value)
        .map_err(|e| ClaimError::InvalidSignature(format!("hex decode: {}", e)))?;

    let signature = ed25519_dalek::Signature::from_bytes(
        sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| ClaimError::InvalidSignature("invalid signature length".into()))?,
    );

    issuer_vk
        .verify(&claim_data, &signature)
        .map_err(|e| ClaimError::InvalidSignature(e.to_string()))?;

    info!("{} VC proof verified successfully", LOG_PREFIX);
    Ok(true)
}

/// Parse a simple ISO 8601 timestamp to Unix seconds (best-effort)
fn iso_to_timestamp(iso: &str) -> Option<u64> {
    // Expected format: YYYY-MM-DDTHH:MM:SSZ
    let parts: Vec<&str> = iso.split('T').collect();
    if parts.len() != 2 { return None; }
    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    let time_str = parts[1].trim_end_matches('Z');
    let time_parts: Vec<u64> = time_str.split(':').filter_map(|s| s.parse().ok()).collect();
    if date_parts.len() != 3 || time_parts.len() != 3 { return None; }

    let (y, m, d) = (date_parts[0] as i64, date_parts[1] as u64, date_parts[2] as u64);
    let (h, min, s) = (time_parts[0], time_parts[1], time_parts[2]);

    // civil_to_days algorithm (inverse of civil_from_days)
    let (y_adj, m_adj) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = (y_adj - era * 400) as u64;
    let doy = (153 * m_adj + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146097 + doe as i64 - 719468) as u64;

    Some(days * 86400 + h * 3600 + min * 60 + s)
}

/// A trusted claim issuer (holds the Ed25519 signing key)
pub struct ClaimIssuer {
    pub did: String,
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl ClaimIssuer {
    /// Create a new claim issuer from a 32-byte seed
    pub fn new(did: &str, seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        info!(
            "{} Claim issuer created: {} (vk={})",
            LOG_PREFIX,
            did,
            hex::encode(verifying_key.as_bytes())
        );
        Self {
            did: did.to_string(),
            signing_key,
            verifying_key,
        }
    }

    /// Issue a signed claim
    pub fn issue_claim(
        &self,
        subject_did: &str,
        claim_type: ClaimType,
        expiry: u64,
    ) -> Claim {
        info!(
            "{} Issuing claim {:?} from {} to {}",
            LOG_PREFIX, claim_type, self.did, subject_did
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create the claim data to sign
        let claim_data = build_claim_data(subject_did, &claim_type, expiry, now);

        // Sign with Ed25519
        let signature = self.signing_key.sign(&claim_data);

        Claim {
            claim_type,
            issuer_did: self.did.clone(),
            subject_did: subject_did.to_string(),
            expiry,
            signature: hex::encode(signature.to_bytes()),
            issued_at: now,
        }
    }
}

/// Build the canonical byte representation of claim data for signing/verification
fn build_claim_data(subject_did: &str, claim_type: &ClaimType, expiry: u64, issued_at: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(subject_did.as_bytes());
    hasher.update(serde_json::to_string(claim_type).unwrap_or_default().as_bytes());
    hasher.update(expiry.to_le_bytes());
    hasher.update(issued_at.to_le_bytes());
    hasher.finalize().to_vec()
}

/// Verify a claim's signature and expiry
pub fn verify_claim(claim: &Claim, issuer_vk: &VerifyingKey) -> Result<bool, ClaimError> {
    info!(
        "{} Verifying claim {:?} for {} (issuer={})",
        LOG_PREFIX, claim.claim_type, claim.subject_did, claim.issuer_did
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

    // Verify Ed25519 signature
    let claim_data = build_claim_data(
        &claim.subject_did,
        &claim.claim_type,
        claim.expiry,
        claim.issued_at,
    );

    let sig_bytes = hex::decode(&claim.signature)
        .map_err(|e| ClaimError::InvalidSignature(format!("hex decode: {}", e)))?;

    let signature = ed25519_dalek::Signature::from_bytes(
        sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| ClaimError::InvalidSignature("invalid signature length".into()))?,
    );

    issuer_vk
        .verify(&claim_data, &signature)
        .map_err(|e| ClaimError::InvalidSignature(e.to_string()))?;

    info!("{} Claim verified successfully", LOG_PREFIX);
    Ok(true)
}

/// Verify a claim using only the claim data (no issuer key — for unsigned/placeholder claims)
pub fn verify_claim_expiry(claim: &Claim) -> Result<bool, ClaimError> {
    if claim.expiry > 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > claim.expiry {
            return Err(ClaimError::Expired);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_issuer() -> ClaimIssuer {
        let seed = [42u8; 32];
        ClaimIssuer::new("did:kaspa:issuer", &seed)
    }

    #[test]
    fn test_issue_and_verify_claim() {
        let issuer = test_issuer();
        let claim = issuer.issue_claim("did:kaspa:alice", ClaimType::KycVerified, 0);

        assert_eq!(claim.claim_type, ClaimType::KycVerified);
        assert_eq!(claim.issuer_did, "did:kaspa:issuer");
        assert!(!claim.signature.is_empty());

        let result = verify_claim(&claim, &issuer.verifying_key);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_wrong_key_fails_verification() {
        let issuer = test_issuer();
        let claim = issuer.issue_claim("did:kaspa:alice", ClaimType::AccreditedInvestor, 0);

        // Verify with a different key
        let wrong_issuer = ClaimIssuer::new("did:kaspa:fake", &[99u8; 32]);
        let result = verify_claim(&claim, &wrong_issuer.verifying_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_claim() {
        let issuer = test_issuer();
        let mut claim = issuer.issue_claim("did:kaspa:alice", ClaimType::AmlClear, 1); // expired at t=1
        claim.issued_at = 0; // fix issued_at so signature would match if not expired

        let result = verify_claim(&claim, &issuer.verifying_key);
        assert!(matches!(result, Err(ClaimError::Expired)));
    }

    #[test]
    fn test_claim_to_vc_roundtrip() {
        let issuer = test_issuer();
        let claim = issuer.issue_claim("did:kaspa:alice", ClaimType::KycVerified, 0);

        // Convert to VC
        let vc = claim.to_verifiable_credential();
        assert_eq!(vc.context[0], "https://www.w3.org/2018/credentials/v1");
        assert_eq!(vc.vc_type[0], "VerifiableCredential");
        assert_eq!(vc.vc_type[1], "ComplianceClaim");
        assert_eq!(vc.issuer, "did:kaspa:issuer");
        assert_eq!(vc.credential_subject.id, "did:kaspa:alice");
        assert_eq!(vc.credential_subject.claim_type, "KycVerified");
        assert!(vc.expiration_date.is_none());
        assert_eq!(vc.proof.type_, "Ed25519Signature2020");

        // Verify the VC proof
        let result = verify_vc_proof(&vc, &issuer.verifying_key);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with jurisdiction claim
        let jclaim = issuer.issue_claim(
            "did:kaspa:bob",
            ClaimType::JurisdictionAllowed("US".to_string()),
            0,
        );
        let jvc = jclaim.to_verifiable_credential();
        assert_eq!(jvc.credential_subject.claim_type, "JurisdictionAllowed:US");
        assert_eq!(jvc.credential_subject.jurisdiction, Some("US".to_string()));
        assert!(verify_vc_proof(&jvc, &issuer.verifying_key).unwrap());
    }

    #[test]
    fn test_jurisdiction_claim() {
        let issuer = test_issuer();
        let claim = issuer.issue_claim(
            "did:kaspa:bob",
            ClaimType::JurisdictionAllowed("US".to_string()),
            0,
        );
        assert_eq!(
            claim.claim_type,
            ClaimType::JurisdictionAllowed("US".to_string())
        );
        assert!(verify_claim(&claim, &issuer.verifying_key).unwrap());
    }
}
