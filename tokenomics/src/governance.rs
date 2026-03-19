// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM governance: proposal creation, voting, execution.
//! Proposals are recorded on-chain via OP_RETURN.
//! Votes are weighted by staked ASTM balance.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("[K-RWA] Proposal not found: {0}")]
    NotFound(u64),
    #[error("[K-RWA] Proposal expired")]
    Expired,
    #[error("[K-RWA] Already voted")]
    AlreadyVoted,
    #[error("[K-RWA] Insufficient stake to propose: {stake} < {required}")]
    InsufficientStake { stake: u64, required: u64 },
    #[error("[K-RWA] Proposal not passed: votes_for={votes_for}, threshold={threshold}")]
    NotPassed { votes_for: u64, threshold: u64 },
}

/// Minimum staked ASTM to create a proposal (100 ASTM in sompis)
pub const MIN_PROPOSE_STAKE: u64 = 100_0000_0000;
/// Default voting period: 7 days
pub const VOTING_PERIOD: u64 = 7 * 24 * 60 * 60;
/// Default execution threshold (weighted ASTM votes)
pub const EXECUTION_THRESHOLD: u64 = 1_000_0000_0000; // 1000 ASTM

/// Proposal status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Expired,
}

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
    pub status: ProposalStatus,
    pub voters: Vec<String>,
}

/// A single vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: String,
    pub weight: u64,
    pub in_favor: bool,
}

/// Governance engine
pub struct GovernanceEngine {
    proposals: Vec<Proposal>,
    next_id: u64,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        info!("{} Initializing governance engine", LOG_PREFIX);
        Self {
            proposals: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        title: &str,
        description: &str,
        proposer: &str,
        proposer_stake: u64,
    ) -> Result<&Proposal, GovernanceError> {
        if proposer_stake < MIN_PROPOSE_STAKE {
            return Err(GovernanceError::InsufficientStake {
                stake: proposer_stake,
                required: MIN_PROPOSE_STAKE,
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let proposal = Proposal {
            id: self.next_id,
            title: title.to_string(),
            description: description.to_string(),
            proposer: proposer.to_string(),
            votes_for: 0,
            votes_against: 0,
            execution_threshold: EXECUTION_THRESHOLD,
            created_at: now,
            expires_at: now + VOTING_PERIOD,
            status: ProposalStatus::Active,
            voters: Vec::new(),
        };

        info!(
            "{} Proposal #{} created: \"{}\" by {}",
            LOG_PREFIX, proposal.id, proposal.title, proposal.proposer
        );

        self.next_id += 1;
        self.proposals.push(proposal);
        Ok(self.proposals.last().unwrap())
    }

    /// Cast a vote on a proposal (weighted by staked ASTM)
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: &str,
        weight: u64,
        in_favor: bool,
    ) -> Result<Vote, GovernanceError> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(GovernanceError::NotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::Expired);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            return Err(GovernanceError::Expired);
        }

        if proposal.voters.contains(&voter.to_string()) {
            return Err(GovernanceError::AlreadyVoted);
        }

        if in_favor {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }
        proposal.voters.push(voter.to_string());

        info!(
            "{} Vote on #{}: voter={}, weight={}, favor={}",
            LOG_PREFIX, proposal_id, voter, weight, in_favor
        );

        // Check if threshold is met
        if proposal.votes_for >= proposal.execution_threshold {
            proposal.status = ProposalStatus::Passed;
            info!("{} Proposal #{} PASSED", LOG_PREFIX, proposal_id);
        }

        Ok(Vote {
            proposal_id,
            voter: voter.to_string(),
            weight,
            in_favor,
        })
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    /// Mark a passed proposal as executed
    pub fn execute(&mut self, proposal_id: u64) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(GovernanceError::NotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Passed {
            return Err(GovernanceError::NotPassed {
                votes_for: proposal.votes_for,
                threshold: proposal.execution_threshold,
            });
        }

        proposal.status = ProposalStatus::Executed;
        info!("{} Proposal #{} EXECUTED", LOG_PREFIX, proposal_id);
        Ok(())
    }
}

/// Generate the OP_RETURN data for recording a proposal on-chain
pub fn proposal_op_return(proposal: &Proposal) -> Vec<u8> {
    let json = serde_json::to_vec(proposal).expect("proposal serialization");
    let mut data = b"ASTM_GOV_V1:".to_vec();
    data.extend_from_slice(&json);
    data
}

/// Compute proposal commitment hash for on-chain reference
pub fn proposal_hash(proposal: &Proposal) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ASTM_PROPOSAL:");
    hasher.update(proposal.id.to_le_bytes());
    hasher.update(proposal.title.as_bytes());
    hasher.update(proposal.proposer.as_bytes());
    hasher.finalize().into()
}

impl Default for GovernanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceEngine::new();
        let p = gov
            .create_proposal("Test", "A test proposal", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.status, ProposalStatus::Active);
    }

    #[test]
    fn test_insufficient_stake() {
        let mut gov = GovernanceEngine::new();
        assert!(gov
            .create_proposal("Test", "desc", "kaspatest:alice", 100)
            .is_err());
    }

    #[test]
    fn test_vote_and_pass() {
        let mut gov = GovernanceEngine::new();
        gov.create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();

        // Vote with enough weight to pass
        gov.vote(1, "kaspatest:bob", EXECUTION_THRESHOLD, true)
            .unwrap();

        let p = gov.get_proposal(1).unwrap();
        assert_eq!(p.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_double_vote_rejected() {
        let mut gov = GovernanceEngine::new();
        gov.create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        gov.vote(1, "kaspatest:bob", 100, true).unwrap();
        assert!(gov.vote(1, "kaspatest:bob", 100, true).is_err());
    }

    #[test]
    fn test_execute_requires_passed() {
        let mut gov = GovernanceEngine::new();
        gov.create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        // Not yet passed
        assert!(gov.execute(1).is_err());
    }

    #[test]
    fn test_execute_passed_proposal() {
        let mut gov = GovernanceEngine::new();
        gov.create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        gov.vote(1, "kaspatest:bob", EXECUTION_THRESHOLD, true)
            .unwrap();
        gov.execute(1).unwrap();
        assert_eq!(
            gov.get_proposal(1).unwrap().status,
            ProposalStatus::Executed
        );
    }

    #[test]
    fn test_proposal_hash_deterministic() {
        let mut gov = GovernanceEngine::new();
        let p = gov
            .create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        let h1 = proposal_hash(p);
        let h2 = proposal_hash(p);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn test_proposal_op_return() {
        let mut gov = GovernanceEngine::new();
        let p = gov
            .create_proposal("Test", "desc", "kaspatest:alice", MIN_PROPOSE_STAKE)
            .unwrap();
        let data = proposal_op_return(p);
        assert!(data.starts_with(b"ASTM_GOV_V1:"));
    }
}
