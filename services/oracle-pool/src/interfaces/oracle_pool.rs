// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! IOraclePool: Upgrade stub for future miner-attested oracle.
//!
//! Current implementation: centralized multisig (services/oracle-pool/)
//! Future: miner-attested per Kaspa core team research (Ori Newman et al.)
//!
//! When Kaspa native decentralized oracles become available:
//! 1. Implement this trait with the new oracle protocol
//! 2. Replace CentralizedOracle with DecentralizedOracle in services/oracle-pool/
//! 3. Update min_signers() to reflect miner attestation threshold

use async_trait::async_trait;
use crate::attestation::Attestation;

/// Oracle pool interface — upgrade stub for future miner-attested oracle.
///
/// Current: centralized 2-of-3 multisig
/// Future: 3-of-5 miner-attested (per Kaspa core team research)
#[async_trait]
pub trait IOraclePool: Send + Sync {
    /// Get a signed price attestation for an asset
    async fn get_attestation(&self, asset_id: &str) -> Result<Attestation, Box<dyn std::error::Error>>;

    /// Verify an attestation has sufficient valid signatures
    async fn verify_attestation(&self, attestation: &Attestation) -> Result<bool, Box<dyn std::error::Error>>;

    /// Minimum required signers
    /// Current: 2 (of 3 oracle keys)
    /// Future: 3 (of 5 miners)
    fn min_signers(&self) -> usize;

    /// Whether this is a decentralized (miner-attested) oracle
    fn is_decentralized(&self) -> bool;
}
