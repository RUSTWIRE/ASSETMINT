// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! AssetMint Identity Registry — implements Polymesh IdentityRegistry patterns.
//! Ported from Polymesh SDK `IdentityApi` into standalone Rust with SQLite storage.
//! Supports DID registration, claim-based identity, and Merkle tree of approved addresses.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::info;

use crate::claims::Claim;
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("[K-RWA] Identity not found: {0}")]
    NotFound(String),
    #[error("[K-RWA] Identity already exists: {0}")]
    AlreadyExists(String),
    #[error("[K-RWA] Storage error: {0}")]
    StorageError(String),
    #[error("[K-RWA] Invalid DID: {0}")]
    InvalidDid(String),
}

/// A registered identity (DID) in the compliance system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Decentralized identifier
    pub did: String,
    /// Primary public key (hex-encoded)
    pub primary_key: String,
    /// Associated claims (loaded separately)
    pub claims: Vec<Claim>,
    /// Whether this identity has been revoked
    pub revoked: bool,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
}

/// Identity registry backed by SQLite
pub struct IdentityRegistry {
    db: Arc<Mutex<Connection>>,
}

impl IdentityRegistry {
    /// Create a new identity registry with SQLite storage
    pub fn new(db_path: &str) -> Result<Self, IdentityError> {
        info!("{} Initializing identity registry (db={})", LOG_PREFIX, db_path);
        let conn = Connection::open(db_path)
            .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS identities (
                did TEXT PRIMARY KEY,
                primary_key TEXT NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS claims (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subject_did TEXT NOT NULL,
                issuer_did TEXT NOT NULL,
                claim_type TEXT NOT NULL,
                claim_data TEXT,
                expiry INTEGER NOT NULL DEFAULT 0,
                signature TEXT NOT NULL DEFAULT '',
                issued_at INTEGER NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (subject_did) REFERENCES identities(did)
            );
            CREATE INDEX IF NOT EXISTS idx_claims_subject ON claims(subject_did);",
        )
        .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory registry (for testing)
    pub fn in_memory() -> Result<Self, IdentityError> {
        Self::new(":memory:")
    }

    /// Create an IdentityRegistry backed by a file on disk.
    /// Data persists across restarts.
    pub fn from_file(path: &str) -> Result<Self, IdentityError> {
        Self::new(path)
    }

    /// Register a new identity
    pub fn register(&self, did: &str, primary_key: &str) -> Result<Identity, IdentityError> {
        // Validate DID format: must match did:kaspa:<identifier>
        let did_re = regex::Regex::new(r"^did:kaspa:[a-zA-Z0-9_-]{1,128}$").unwrap();
        if !did_re.is_match(did) {
            return Err(IdentityError::InvalidDid(format!(
                "DID must match format did:kaspa:<identifier> (1-128 alphanumeric chars), got: {}",
                did
            )));
        }

        // Validate primary_key: must be valid hex, exactly 64 chars (32 bytes)
        if primary_key.len() != 64 || hex::decode(primary_key).is_err() {
            return Err(IdentityError::InvalidDid(format!(
                "primary_key must be 64 hex characters (32 bytes), got length: {}",
                primary_key.len()
            )));
        }

        info!("{} Registering identity: {}", LOG_PREFIX, did);
        let db = self.db.lock().map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        db.execute(
            "INSERT INTO identities (did, primary_key, revoked, created_at) VALUES (?1, ?2, 0, ?3)",
            params![did, primary_key, now as i64],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                IdentityError::AlreadyExists(did.to_string())
            } else {
                IdentityError::StorageError(e.to_string())
            }
        })?;

        info!("{} Identity registered: {}", LOG_PREFIX, did);
        Ok(Identity {
            did: did.to_string(),
            primary_key: primary_key.to_string(),
            claims: Vec::new(),
            revoked: false,
            created_at: now,
        })
    }

    /// Get an identity by DID, including all active claims
    pub fn get(&self, did: &str) -> Result<Identity, IdentityError> {
        let db = self.db.lock().map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let mut stmt = db
            .prepare("SELECT did, primary_key, revoked, created_at FROM identities WHERE did = ?1")
            .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let identity = stmt
            .query_row(params![did], |row| {
                Ok(Identity {
                    did: row.get(0)?,
                    primary_key: row.get(1)?,
                    claims: Vec::new(),
                    revoked: row.get::<_, i64>(2)? != 0,
                    created_at: row.get::<_, i64>(3)? as u64,
                })
            })
            .map_err(|_| IdentityError::NotFound(did.to_string()))?;

        // Load claims
        let mut claim_stmt = db
            .prepare(
                "SELECT claim_type, claim_data, issuer_did, subject_did, expiry, signature, issued_at
                 FROM claims WHERE subject_did = ?1 AND revoked = 0",
            )
            .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let claims: Vec<Claim> = claim_stmt
            .query_map(params![did], |row| {
                let claim_type_str: String = row.get(0)?;
                let claim_data: Option<String> = row.get(1)?;
                let claim_type = deserialize_claim_type(&claim_type_str, claim_data.as_deref());
                Ok(Claim {
                    claim_type,
                    issuer_did: row.get(2)?,
                    subject_did: row.get(3)?,
                    expiry: row.get::<_, i64>(4)? as u64,
                    signature: row.get(5)?,
                    issued_at: row.get::<_, i64>(6)? as u64,
                })
            })
            .map_err(|e| IdentityError::StorageError(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Identity { claims, ..identity })
    }

    /// Revoke an identity
    pub fn revoke(&self, did: &str) -> Result<(), IdentityError> {
        info!("{} Revoking identity: {}", LOG_PREFIX, did);
        let db = self.db.lock().map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let affected = db
            .execute("UPDATE identities SET revoked = 1 WHERE did = ?1", params![did])
            .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        if affected == 0 {
            return Err(IdentityError::NotFound(did.to_string()));
        }
        Ok(())
    }

    /// Store a claim in the database
    pub fn add_claim(&self, claim: &Claim) -> Result<(), IdentityError> {
        info!(
            "{} Storing claim {:?} for {}",
            LOG_PREFIX, claim.claim_type, claim.subject_did
        );
        let db = self.db.lock().map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let (type_str, data_str) = serialize_claim_type(&claim.claim_type);

        db.execute(
            "INSERT INTO claims (subject_did, issuer_did, claim_type, claim_data, expiry, signature, issued_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                claim.subject_did,
                claim.issuer_did,
                type_str,
                data_str,
                claim.expiry as i64,
                claim.signature,
                claim.issued_at as i64,
            ],
        )
        .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get all registered (non-revoked) addresses for Merkle tree construction
    pub fn get_approved_addresses(&self) -> Result<Vec<String>, IdentityError> {
        let db = self.db.lock().map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let mut stmt = db
            .prepare("SELECT primary_key FROM identities WHERE revoked = 0")
            .map_err(|e| IdentityError::StorageError(e.to_string()))?;

        let addresses: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| IdentityError::StorageError(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(addresses)
    }

    /// Get the database connection (for sharing with API)
    pub fn db(&self) -> Arc<Mutex<Connection>> {
        self.db.clone()
    }
}

use crate::claims::ClaimType;

fn serialize_claim_type(ct: &ClaimType) -> (String, Option<String>) {
    match ct {
        ClaimType::KycVerified => ("KycVerified".into(), None),
        ClaimType::AccreditedInvestor => ("AccreditedInvestor".into(), None),
        ClaimType::JurisdictionAllowed(j) => ("JurisdictionAllowed".into(), Some(j.clone())),
        ClaimType::AmlClear => ("AmlClear".into(), None),
        ClaimType::ExemptedEntity => ("ExemptedEntity".into(), None),
    }
}

fn deserialize_claim_type(type_str: &str, data: Option<&str>) -> ClaimType {
    match type_str {
        "KycVerified" => ClaimType::KycVerified,
        "AccreditedInvestor" => ClaimType::AccreditedInvestor,
        "JurisdictionAllowed" => ClaimType::JurisdictionAllowed(data.unwrap_or("").to_string()),
        "AmlClear" => ClaimType::AmlClear,
        "ExemptedEntity" => ClaimType::ExemptedEntity,
        _ => ClaimType::KycVerified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex64(byte: u8) -> String {
        format!("{}", std::iter::repeat(format!("{:02x}", byte)).take(32).collect::<String>())
    }

    #[test]
    fn test_register_and_get() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let key = hex64(0xab);
        let id = registry.register("did:kaspa:alice", &key).unwrap();
        assert_eq!(id.did, "did:kaspa:alice");
        assert!(!id.revoked);

        let fetched = registry.get("did:kaspa:alice").unwrap();
        assert_eq!(fetched.primary_key, key);
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let key1 = hex64(0xab);
        let key2 = hex64(0xcd);
        registry.register("did:kaspa:alice", &key1).unwrap();
        let err = registry.register("did:kaspa:alice", &key2);
        assert!(err.is_err());
    }

    #[test]
    fn test_revoke() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let key = hex64(0xbb);
        registry.register("did:kaspa:bob", &key).unwrap();
        registry.revoke("did:kaspa:bob").unwrap();
        let id = registry.get("did:kaspa:bob").unwrap();
        assert!(id.revoked);
    }

    #[test]
    fn test_approved_addresses() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let key_a = hex64(0x0a);
        let key_b = hex64(0x0b);
        let key_c = hex64(0x0c);
        registry.register("did:kaspa:a", &key_a).unwrap();
        registry.register("did:kaspa:b", &key_b).unwrap();
        registry.register("did:kaspa:c", &key_c).unwrap();
        registry.revoke("did:kaspa:b").unwrap();

        let approved = registry.get_approved_addresses().unwrap();
        assert_eq!(approved.len(), 2);
        assert!(approved.contains(&key_a));
        assert!(approved.contains(&key_c));
    }

    #[test]
    fn test_add_and_load_claims() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let key = hex64(0xab);
        registry.register("did:kaspa:alice", &key).unwrap();

        let claim = Claim {
            claim_type: ClaimType::KycVerified,
            issuer_did: "did:kaspa:issuer".into(),
            subject_did: "did:kaspa:alice".into(),
            expiry: 0,
            signature: "sig_placeholder".into(),
            issued_at: 1000,
        };
        registry.add_claim(&claim).unwrap();

        let id = registry.get("did:kaspa:alice").unwrap();
        assert_eq!(id.claims.len(), 1);
        assert_eq!(id.claims[0].claim_type, ClaimType::KycVerified);
    }

    #[test]
    fn test_invalid_did_format_rejected() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let result = registry.register("invalid-did", &"aa".repeat(32));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_primary_key_rejected() {
        let registry = IdentityRegistry::in_memory().unwrap();
        let result = registry.register("did:kaspa:alice", "not-hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_backed_persistence() {
        let path = "/tmp/assetmint_test_identity.db";
        // Clean up from previous runs
        let _ = std::fs::remove_file(path);

        // Create and register
        {
            let registry = IdentityRegistry::from_file(path).unwrap();
            registry.register("did:kaspa:persist-test", &"aa".repeat(32)).unwrap();
        }

        // Reopen and verify
        {
            let registry = IdentityRegistry::from_file(path).unwrap();
            let identity = registry.get("did:kaspa:persist-test").unwrap();
            assert_eq!(identity.did, "did:kaspa:persist-test");
        }

        // Clean up
        let _ = std::fs::remove_file(path);
    }
}
