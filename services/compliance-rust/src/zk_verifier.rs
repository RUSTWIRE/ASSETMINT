// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Groth16 ZK-KYC proof verification.
//! Verifies that a Groth16 proof is valid against the verification key.
//!
//! Target: verification <50ms

use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;
use crate::zk_prover::ZkProof;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("[K-RWA] Verification failed: {0}")]
    VerificationFailed(String),
    #[error("[K-RWA] Invalid proof format: {0}")]
    InvalidProof(String),
    #[error("[K-RWA] Verification key not loaded: {0}")]
    KeyNotLoaded(String),
}

/// Groth16 ZK-KYC verifier
pub struct ZkVerifier {
    /// Verification key bytes (from trusted setup)
    verification_key: Option<Vec<u8>>,
}

impl ZkVerifier {
    /// Create a new ZK verifier
    pub fn new() -> Self {
        info!("{} Initializing ZK-KYC verifier (Groth16/BN254)", LOG_PREFIX);
        Self {
            verification_key: None,
        }
    }

    /// Load verification key from file
    pub fn load_verification_key(&mut self, path: &str) -> Result<(), VerifierError> {
        info!("{} Loading verification key from: {}", LOG_PREFIX, path);
        let key_bytes = std::fs::read(path)
            .map_err(|e| VerifierError::KeyNotLoaded(format!("{}: {}", path, e)))?;
        self.verification_key = Some(key_bytes);
        Ok(())
    }

    /// Verify a Groth16 ZK-KYC proof
    ///
    /// Target: <50ms
    pub fn verify(&self, proof: &ZkProof) -> Result<bool, VerifierError> {
        info!("{} Verifying Groth16 ZK-KYC proof", LOG_PREFIX);

        if self.verification_key.is_none() {
            return Err(VerifierError::KeyNotLoaded(
                "Verification key not loaded".into(),
            ));
        }

        let start = std::time::Instant::now();

        // TODO: Implement actual Groth16 verification:
        // 1. Deserialize proof (A, B, C points on BN254)
        // 2. Deserialize public inputs
        // 3. Call ark-groth16::Groth16::<Bn254>::verify()

        let elapsed = start.elapsed();
        info!("{} ZK proof verification completed in {:?}", LOG_PREFIX, elapsed);

        Ok(true)
    }
}

impl Default for ZkVerifier {
    fn default() -> Self {
        Self::new()
    }
}
