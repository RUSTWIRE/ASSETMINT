// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # kaspa-adapter
//!
//! Kaspa Testnet-12 adapter crate. Wraps rusty-kaspa WASM bindings for:
//! - WebSocket RPC client (`client`)
//! - Dynamic testnet wallet generation (`wallet`)
//! - UTXO selection and covenant transaction building (`tx_builder`)
//! - Compiled SilverScript → P2SH address generation (`script`)
//!
//! **Testnet-12 ONLY** — ws://tn12-node.kaspa.com:17210

pub mod client;
pub mod wallet;
pub mod tx_builder;
pub mod script;
pub mod covenant_builder;

pub use wallet::ThresholdWallet;

/// Log prefix for all AssetMint operations
pub const LOG_PREFIX: &str = "[K-RWA]";

/// Kaspa Testnet-12 RPC endpoint
pub const TESTNET_12_RPC: &str = "ws://tn12-node.kaspa.com:17210";
