// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # zk-circuits
//!
//! Groth16 ZK-KYC circuits for AssetMint.
//! Uses ark-groth16 with BN254 curve.
//!
//! - `kyc_circuit` — Proves Merkle inclusion without revealing the address
//! - `setup` — Trusted setup for generating proving/verification keys

pub mod kyc_circuit;
pub mod setup;

/// Log prefix for all AssetMint ZK operations
pub const LOG_PREFIX: &str = "[K-RWA]";
