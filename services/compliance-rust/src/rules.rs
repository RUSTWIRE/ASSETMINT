// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Transfer restriction engine — ported from Polymesh Compliance.Requirements.
//! Evaluates whether a transfer is allowed based on composable rules.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;
use crate::claims::ClaimType;
use crate::identity::Identity;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("[K-RWA] Transfer denied: {violations:?}")]
    TransferDenied { violations: Vec<String> },
}

/// A single compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceRule {
    /// Sender must have a specific claim type
    SenderMustHaveClaim(ClaimType),
    /// Receiver must have a specific claim type
    ReceiverMustHaveClaim(ClaimType),
    /// Receiver jurisdiction must NOT be in blocked list
    ReceiverJurisdictionNotIn(Vec<String>),
    /// Maximum single transfer amount (in sompis)
    MaxTransferAmount(u64),
    /// Minimum hold period before transfer (seconds since mint)
    HoldPeriod(u64),
}

/// Result of a compliance evaluation
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub allowed: bool,
    pub violations: Vec<String>,
    pub rules_evaluated: usize,
}

/// Compliance engine — evaluates transfer rules
pub struct ComplianceEngine {
    rules: Vec<ComplianceRule>,
}

impl ComplianceEngine {
    /// Create a new compliance engine with default RWA rules
    pub fn new() -> Self {
        info!("{} Initializing compliance engine", LOG_PREFIX);
        Self {
            rules: vec![
                ComplianceRule::SenderMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverMustHaveClaim(ClaimType::KycVerified),
                ComplianceRule::ReceiverJurisdictionNotIn(vec![
                    "OFAC_BLOCKED".to_string(),
                ]),
            ],
        }
    }

    /// Add a rule to the engine
    pub fn add_rule(&mut self, rule: ComplianceRule) {
        info!("{} Adding compliance rule: {:?}", LOG_PREFIX, rule);
        self.rules.push(rule);
    }

    /// Evaluate whether a transfer is allowed
    pub fn evaluate_transfer(
        &self,
        sender: &Identity,
        receiver: &Identity,
        _asset_id: &str,
        amount: u64,
    ) -> ComplianceResult {
        info!(
            "{} Evaluating transfer: {} -> {} (amount={})",
            LOG_PREFIX, sender.did, receiver.did, amount
        );

        let mut violations = Vec::new();

        for rule in &self.rules {
            match rule {
                ComplianceRule::SenderMustHaveClaim(claim_type) => {
                    if !sender.claims.iter().any(|c| c.claim_type == *claim_type) {
                        violations.push(format!(
                            "Sender {} missing required claim: {:?}",
                            sender.did, claim_type
                        ));
                    }
                }
                ComplianceRule::ReceiverMustHaveClaim(claim_type) => {
                    if !receiver.claims.iter().any(|c| c.claim_type == *claim_type) {
                        violations.push(format!(
                            "Receiver {} missing required claim: {:?}",
                            receiver.did, claim_type
                        ));
                    }
                }
                ComplianceRule::MaxTransferAmount(max) => {
                    if amount > *max {
                        violations.push(format!(
                            "Transfer amount {} exceeds maximum {}",
                            amount, max
                        ));
                    }
                }
                ComplianceRule::ReceiverJurisdictionNotIn(blocked) => {
                    for claim in &receiver.claims {
                        if let ClaimType::JurisdictionAllowed(jurisdiction) = &claim.claim_type {
                            if blocked.contains(jurisdiction) {
                                violations.push(format!(
                                    "Receiver jurisdiction {} is blocked",
                                    jurisdiction
                                ));
                            }
                        }
                    }
                }
                ComplianceRule::HoldPeriod(_period) => {
                    // TODO: Check asset mint timestamp vs current time
                }
            }
        }

        let allowed = violations.is_empty();
        let rules_evaluated = self.rules.len();

        info!(
            "{} Compliance evaluation: allowed={}, violations={}, rules={}",
            LOG_PREFIX, allowed, violations.len(), rules_evaluated
        );

        ComplianceResult {
            allowed,
            violations,
            rules_evaluated,
        }
    }
}

impl Default for ComplianceEngine {
    fn default() -> Self {
        Self::new()
    }
}
