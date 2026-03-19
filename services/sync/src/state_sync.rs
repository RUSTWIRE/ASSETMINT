// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! State-verity sync: polls DKG Edge Node for Knowledge Asset changes
//! and creates new state-transition UTXOs on Kaspa Testnet-12.
//!
//! State chain pattern: spend previous state UTXO → new state UTXO
//! enforced by state-verity.sil covenant.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("[K-RWA] DKG poll failed: {0}")]
    DkgPollFailed(String),
    #[error("[K-RWA] State transition failed: {0}")]
    TransitionFailed(String),
    #[error("[K-RWA] No state change detected")]
    NoChange,
}

/// Current state of an asset on-chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetState {
    /// DKG Universal Asset Locator
    pub dkg_ual: String,
    /// Last known assertion ID from DKG
    pub assertion_id: String,
    /// SHA-256 hash of the DKG UAL (for covenant)
    pub dkg_ual_hash: [u8; 32],
    /// Oracle attestation hash
    pub oracle_attestation_hash: [u8; 32],
    /// Compliance Merkle root
    pub compliance_merkle_root: [u8; 32],
    /// Current UTXO holding the state
    pub state_utxo_txid: String,
    pub state_utxo_index: u32,
    /// State version counter
    pub version: u64,
}

/// A state transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state hash
    pub prev_state_hash: [u8; 32],
    /// New state hash
    pub new_state_hash: [u8; 32],
    /// What changed
    pub change_type: ChangeType,
    /// Timestamp of transition
    pub timestamp: u64,
}

/// Types of state changes that trigger a transition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    /// DKG Knowledge Asset was updated
    DkgUpdate,
    /// Oracle attestation was refreshed
    OracleUpdate,
    /// Compliance Merkle root changed (new KYC approval/revocation)
    ComplianceUpdate,
    /// Multiple changes in a single transition
    Combined(Vec<ChangeType>),
}

/// Deployed StateVerity covenant P2SH address on Kaspa TN12
pub const STATE_VERITY_P2SH: &str =
    "kaspatest:pq6xyf8f4tzpeuz4s6yy8063j6g6dwv0a4lcerv4uc98m99shgpcsftdcl5d7";
/// StateVerity deployment TX
pub const STATE_VERITY_TX: &str =
    "94c50753b05e7d998af30fa51aad4d27f2e7fdd0e9ae48b655255b94d129fe5f";

/// State-verity sync service
pub struct StateSyncService {
    dkg_endpoint: String,
    poll_interval_secs: u64,
    current_state: Option<AssetState>,
}

impl StateSyncService {
    /// Create a new state sync service
    pub fn new(dkg_endpoint: &str, poll_interval_secs: u64) -> Self {
        info!(
            "{} Initializing state sync service: dkg={}, interval={}s",
            LOG_PREFIX, dkg_endpoint, poll_interval_secs
        );
        Self {
            dkg_endpoint: dkg_endpoint.to_string(),
            poll_interval_secs,
            current_state: None,
        }
    }

    /// Set the initial asset state (loaded from chain or initialized)
    pub fn set_initial_state(&mut self, state: AssetState) {
        info!(
            "{} Setting initial state: ual={}, version={}",
            LOG_PREFIX, state.dkg_ual, state.version
        );
        self.current_state = Some(state);
    }

    /// Check DKG for changes and build a state transition if needed
    pub fn check_and_transition(
        &mut self,
        new_assertion_id: Option<&str>,
        new_oracle_hash: Option<[u8; 32]>,
        new_merkle_root: Option<[u8; 32]>,
    ) -> Result<StateTransition, SyncError> {
        let current = self
            .current_state
            .as_ref()
            .ok_or_else(|| SyncError::TransitionFailed("No current state set".into()))?;

        let mut changes = Vec::new();
        let mut new_state = current.clone();
        new_state.version += 1;

        // Check DKG change
        if let Some(assertion_id) = new_assertion_id {
            if assertion_id != current.assertion_id {
                info!(
                    "{} DKG assertion changed: {} -> {}",
                    LOG_PREFIX, current.assertion_id, assertion_id
                );
                new_state.assertion_id = assertion_id.to_string();
                new_state.dkg_ual_hash = compute_ual_hash(&new_state.dkg_ual, assertion_id);
                changes.push(ChangeType::DkgUpdate);
            }
        }

        // Check oracle attestation change
        if let Some(hash) = new_oracle_hash {
            if hash != current.oracle_attestation_hash {
                info!("{} Oracle attestation updated", LOG_PREFIX);
                new_state.oracle_attestation_hash = hash;
                changes.push(ChangeType::OracleUpdate);
            }
        }

        // Check compliance Merkle root change
        if let Some(root) = new_merkle_root {
            if root != current.compliance_merkle_root {
                info!("{} Compliance Merkle root updated", LOG_PREFIX);
                new_state.compliance_merkle_root = root;
                changes.push(ChangeType::ComplianceUpdate);
            }
        }

        if changes.is_empty() {
            return Err(SyncError::NoChange);
        }

        let prev_hash = compute_state_hash(current);
        let new_hash = compute_state_hash(&new_state);

        let change_type = if changes.len() == 1 {
            changes.into_iter().next().unwrap()
        } else {
            ChangeType::Combined(changes)
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let transition = StateTransition {
            prev_state_hash: prev_hash,
            new_state_hash: new_hash,
            change_type,
            timestamp: now,
        };

        info!(
            "{} State transition v{} -> v{}: prev={}, new={}",
            LOG_PREFIX,
            current.version,
            new_state.version,
            hex::encode(&prev_hash[..8]),
            hex::encode(&new_hash[..8])
        );

        // Clear UTXO info (will be set after broadcast)
        new_state.state_utxo_txid = String::new();
        new_state.state_utxo_index = 0;

        self.current_state = Some(new_state);
        Ok(transition)
    }

    /// Get the current state
    pub fn current_state(&self) -> Option<&AssetState> {
        self.current_state.as_ref()
    }

    /// Get the DKG endpoint
    pub fn dkg_endpoint(&self) -> &str {
        &self.dkg_endpoint
    }

    /// Get the poll interval
    pub fn poll_interval(&self) -> u64 {
        self.poll_interval_secs
    }

    /// Start the sync loop (runs indefinitely, polls DKG endpoint only)
    ///
    /// This is the original loop that only logs; kept for backward compatibility.
    pub async fn run(&self) -> Result<(), SyncError> {
        info!("{} Starting state-verity sync loop", LOG_PREFIX);
        loop {
            info!(
                "{} Polling DKG at {} for state changes...",
                LOG_PREFIX, self.dkg_endpoint
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(self.poll_interval_secs)).await;
        }
    }

    /// Start the compliance-polling sync loop (runs indefinitely).
    ///
    /// Polls the compliance API's `/merkle-root` endpoint for Merkle root
    /// changes. When the root changes, triggers a state transition via
    /// [`check_and_transition`].
    pub async fn run_polling(&mut self, compliance_api_url: &str) -> Result<(), SyncError> {
        info!(
            "{} Starting state-verity sync loop, polling {}",
            LOG_PREFIX, compliance_api_url
        );
        let client = reqwest::Client::new();
        let mut last_merkle_root: Option<[u8; 32]> = None;

        loop {
            // Poll compliance API for current Merkle root
            match client
                .get(format!("{}/merkle-root", compliance_api_url))
                .send()
                .await
            {
                Ok(response) => {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        // The compliance API returns { "root": "<hex>", "leaf_count": N }
                        if let Some(root_hex) = data["root"].as_str() {
                            let root_bytes = hex::decode(root_hex).unwrap_or_default();
                            if root_bytes.len() == 32 {
                                let mut new_root = [0u8; 32];
                                new_root.copy_from_slice(&root_bytes);

                                if last_merkle_root.map_or(true, |old| old != new_root) {
                                    info!("{} Merkle root changed: {}", LOG_PREFIX, root_hex);
                                    // Trigger state transition
                                    match self.check_and_transition(None, None, Some(new_root)) {
                                        Ok(transition) => {
                                            info!(
                                                "{} State transition: {:?} (v{} prev={} new={})",
                                                LOG_PREFIX,
                                                transition.change_type,
                                                self.current_state
                                                    .as_ref()
                                                    .map_or(0, |s| s.version),
                                                hex::encode(&transition.prev_state_hash[..8]),
                                                hex::encode(&transition.new_state_hash[..8]),
                                            );
                                        }
                                        Err(SyncError::NoChange) => {
                                            // Root matched current state, not an error
                                        }
                                        Err(e) => {
                                            info!("{} State transition failed: {}", LOG_PREFIX, e);
                                        }
                                    }
                                    last_merkle_root = Some(new_root);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    info!(
                        "{} Compliance API unreachable: {}, retrying in {}s",
                        LOG_PREFIX, e, self.poll_interval_secs
                    );
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(self.poll_interval_secs)).await;
        }
    }
}

/// Compute SHA-256 hash of the full asset state (for state chain)
pub fn compute_state_hash(state: &AssetState) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(state.dkg_ual.as_bytes());
    hasher.update(state.assertion_id.as_bytes());
    hasher.update(&state.dkg_ual_hash);
    hasher.update(&state.oracle_attestation_hash);
    hasher.update(&state.compliance_merkle_root);
    hasher.update(state.version.to_le_bytes());
    hasher.finalize().into()
}

/// Compute UAL hash for covenant (DKG UAL + assertion ID)
pub fn compute_ual_hash(ual: &str, assertion_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ual.as_bytes());
    hasher.update(b":");
    hasher.update(assertion_id.as_bytes());
    hasher.finalize().into()
}

/// Build the OP_RETURN data for a state transition UTXO
pub fn state_transition_op_return(transition: &StateTransition) -> Vec<u8> {
    let mut data = b"ASTM_STATE_V1:".to_vec();
    data.extend_from_slice(&transition.prev_state_hash);
    data.extend_from_slice(&transition.new_state_hash);
    data.extend_from_slice(&transition.timestamp.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AssetState {
        AssetState {
            dkg_ual: "did:dkg:otp/0x123/456".into(),
            assertion_id: "assertion_v1".into(),
            dkg_ual_hash: compute_ual_hash("did:dkg:otp/0x123/456", "assertion_v1"),
            oracle_attestation_hash: [1u8; 32],
            compliance_merkle_root: [2u8; 32],
            state_utxo_txid: "abc123".into(),
            state_utxo_index: 0,
            version: 1,
        }
    }

    #[test]
    fn test_state_transition_dkg_update() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        let transition = svc
            .check_and_transition(Some("assertion_v2"), None, None)
            .unwrap();
        assert_eq!(transition.change_type, ChangeType::DkgUpdate);
        assert_ne!(transition.prev_state_hash, transition.new_state_hash);

        let new = svc.current_state().unwrap();
        assert_eq!(new.version, 2);
        assert_eq!(new.assertion_id, "assertion_v2");
    }

    #[test]
    fn test_state_transition_oracle_update() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        let new_hash = [99u8; 32];
        let transition = svc
            .check_and_transition(None, Some(new_hash), None)
            .unwrap();
        assert_eq!(transition.change_type, ChangeType::OracleUpdate);
    }

    #[test]
    fn test_state_transition_compliance_update() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        let new_root = [99u8; 32];
        let transition = svc
            .check_and_transition(None, None, Some(new_root))
            .unwrap();
        assert_eq!(transition.change_type, ChangeType::ComplianceUpdate);
    }

    #[test]
    fn test_state_transition_combined() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        let transition = svc
            .check_and_transition(Some("assertion_v2"), Some([99u8; 32]), None)
            .unwrap();
        match &transition.change_type {
            ChangeType::Combined(changes) => assert_eq!(changes.len(), 2),
            _ => panic!("Expected Combined change type"),
        }
    }

    #[test]
    fn test_no_change() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        // Same assertion ID, no oracle or merkle changes
        let result = svc.check_and_transition(Some("assertion_v1"), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_hash_deterministic() {
        let state = test_state();
        let h1 = compute_state_hash(&state);
        let h2 = compute_state_hash(&state);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn test_merkle_root_polling_transition() {
        // Simulates what run_polling does: detect a changed Merkle root
        // and trigger check_and_transition with the new root
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        svc.set_initial_state(test_state());

        let original_root = svc.current_state().unwrap().compliance_merkle_root;
        assert_eq!(original_root, [2u8; 32]); // from test_state()

        // Simulate a new root arriving from /merkle-root polling
        let new_root = [42u8; 32];
        let transition = svc
            .check_and_transition(None, None, Some(new_root))
            .unwrap();
        assert_eq!(transition.change_type, ChangeType::ComplianceUpdate);

        // State should now reflect the new root
        let updated = svc.current_state().unwrap();
        assert_eq!(updated.compliance_merkle_root, new_root);
        assert_eq!(updated.version, 2);

        // Same root again should produce NoChange
        let no_change = svc.check_and_transition(None, None, Some(new_root));
        assert!(matches!(no_change, Err(SyncError::NoChange)));

        // A third root triggers another transition
        let third_root = [99u8; 32];
        let transition2 = svc
            .check_and_transition(None, None, Some(third_root))
            .unwrap();
        assert_eq!(transition2.change_type, ChangeType::ComplianceUpdate);
        assert_eq!(svc.current_state().unwrap().version, 3);
    }

    #[test]
    fn test_no_state_set_errors() {
        let mut svc = StateSyncService::new("http://localhost:8900", 30);
        // No initial state set — should fail gracefully
        let result = svc.check_and_transition(None, None, Some([1u8; 32]));
        assert!(matches!(result, Err(SyncError::TransitionFailed(_))));
    }

    #[test]
    fn test_op_return_format() {
        let transition = StateTransition {
            prev_state_hash: [1u8; 32],
            new_state_hash: [2u8; 32],
            change_type: ChangeType::DkgUpdate,
            timestamp: 1700000000,
        };
        let data = state_transition_op_return(&transition);
        assert!(data.starts_with(b"ASTM_STATE_V1:"));
        // 14 prefix + 32 prev + 32 new + 8 timestamp = 86
        assert_eq!(data.len(), 86);
    }
}
