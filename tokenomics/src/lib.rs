// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # tokenomics
//!
//! ASTM protocol token: KRC-20 inscription-based token on Kaspa Testnet-12.
//! Includes staking, governance, and fee model.

pub mod token;
pub mod staking;
pub mod governance;
pub mod fee_model;

/// Log prefix for all AssetMint tokenomics operations
pub const LOG_PREFIX: &str = "[K-RWA]";
