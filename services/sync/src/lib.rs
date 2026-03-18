// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # sync
//!
//! DKG state-verity sync service.
//! Monitors OriginTrail DKG Edge Node for Knowledge Asset changes
//! and creates state-transition UTXOs on Kaspa Testnet-12.

pub mod state_sync;

/// Log prefix for all AssetMint sync operations
pub const LOG_PREFIX: &str = "[K-RWA]";
