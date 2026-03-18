// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM governance: proposal creation, voting, execution.

use serde::{Deserialize, Serialize};
use tracing::info;
use crate::LOG_PREFIX;

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub execution_threshold: u64,
    pub created_at: u64,
    pub expires_at: u64,
}

/// Create a new governance proposal
pub fn create_proposal(title: &str, description: &str, proposer: &str) -> Proposal {
    info!("{} Creating governance proposal: {}", LOG_PREFIX, title);
    Proposal {
        id: 0, // TODO: Auto-increment
        title: title.to_string(),
        description: description.to_string(),
        proposer: proposer.to_string(),
        votes_for: 0,
        votes_against: 0,
        execution_threshold: 1000, // 1000 ASTM staked to pass
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        expires_at: 0, // TODO: Set expiry
    }
}
