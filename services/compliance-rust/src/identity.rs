// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Identity registry — ported from Polymesh SDK IdentityRegistry.
//! Manages DIDs, primary keys, and associated claims for RWA compliance.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;
use crate::claims::Claim;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("[K-RWA] Identity not found: {0}")]
    NotFound(String),
    #[error("[K-RWA] Identity already exists: {0}")]
    AlreadyExists(String),
    #[error("[K-RWA] Storage error: {0}")]
    StorageError(String),
}

/// A registered identity (DID) in the compliance system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Decentralized identifier
    pub did: String,
    /// Primary public key (hex-encoded)
    pub primary_key: String,
    /// Associated claims
    pub claims: Vec<Claim>,
    /// Whether this identity has been revoked
    pub revoked: bool,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
}

/// Identity registry — stores and manages all registered identities
pub struct IdentityRegistry {
    // TODO: Replace with rusqlite for persistence
    identities: std::collections::HashMap<String, Identity>,
}

impl IdentityRegistry {
    /// Create a new in-memory identity registry
    pub fn new() -> Self {
        info!("{} Initializing identity registry", LOG_PREFIX);
        Self {
            identities: std::collections::HashMap::new(),
        }
    }

    /// Register a new identity
    pub fn register(&mut self, did: &str, primary_key: &str) -> Result<Identity, IdentityError> {
        info!("{} Registering identity: {}", LOG_PREFIX, did);
        if self.identities.contains_key(did) {
            return Err(IdentityError::AlreadyExists(did.to_string()));
        }

        let identity = Identity {
            did: did.to_string(),
            primary_key: primary_key.to_string(),
            claims: Vec::new(),
            revoked: false,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        self.identities.insert(did.to_string(), identity.clone());
        info!("{} Identity registered: {}", LOG_PREFIX, did);
        Ok(identity)
    }

    /// Get an identity by DID
    pub fn get(&self, did: &str) -> Result<&Identity, IdentityError> {
        self.identities
            .get(did)
            .ok_or_else(|| IdentityError::NotFound(did.to_string()))
    }

    /// Revoke an identity
    pub fn revoke(&mut self, did: &str) -> Result<(), IdentityError> {
        info!("{} Revoking identity: {}", LOG_PREFIX, did);
        let identity = self.identities
            .get_mut(did)
            .ok_or_else(|| IdentityError::NotFound(did.to_string()))?;
        identity.revoked = true;
        Ok(())
    }

    /// Get all registered (non-revoked) addresses for Merkle tree construction
    pub fn get_approved_addresses(&self) -> Vec<String> {
        self.identities
            .values()
            .filter(|id| !id.revoked)
            .map(|id| id.primary_key.clone())
            .collect()
    }
}

impl Default for IdentityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
