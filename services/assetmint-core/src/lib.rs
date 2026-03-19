// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # AssetMint Core — Institutional RWA Compliance Engine
//!
//! Full Polymesh compliance port for AssetMint on Kaspa Testnet-12.
//! Implements Polymesh SDK patterns (IdentityRegistry, ComplianceManager, CDD)
//! as a standalone Rust engine with SQLite storage and Groth16 ZK proofs.
//!
//! Modules:
//! - `identity` — DID-based identity registry (ported from Polymesh IdentityRegistry)
//! - `claims` — Claim types and issuance (CDD, KYC, Accredited)
//! - `rules` — Transfer restriction engine (ported from Polymesh Compliance.Requirements)
//! - `merkle` — Merkle tree of approved addresses for on-chain verification
//! - `zk_prover` — Groth16 proof generation (ark-groth16)
//! - `zk_verifier` — Groth16 proof verification
//! - `api` — Axum REST API endpoints

pub mod api;
pub mod auth;
pub mod claims;
pub mod identity;
pub mod merkle;
pub mod rate_limit;
pub mod rules;
pub mod zk_prover;
pub mod zk_verifier;

/// Log prefix for all AssetMint compliance operations
pub const LOG_PREFIX: &str = "[K-RWA]";
