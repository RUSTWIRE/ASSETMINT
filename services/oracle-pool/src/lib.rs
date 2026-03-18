// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # oracle-pool
//!
//! Simulated centralized multisig oracle for AssetMint.
//! Currently: 2-of-3 multisig with off-chain price aggregation.
//! Future: miner-attested oracle per Kaspa core team research (see IOraclePool trait).

pub mod oracle;
pub mod attestation;
pub mod interfaces;

/// Log prefix for all AssetMint oracle operations
pub const LOG_PREFIX: &str = "[K-RWA]";
