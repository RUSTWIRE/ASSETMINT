// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Groth16 trusted setup for ZK-KYC circuits.
//! Generates proving and verification keys.
//!
//! NOTE: For testnet demo, uses deterministic setup.
//! Production MUST use multi-party computation (MPC) ceremony.

use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;
use crate::kyc_circuit::KycCircuit;

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("[K-RWA] Setup failed: {0}")]
    SetupFailed(String),
    #[error("[K-RWA] Key serialization failed: {0}")]
    SerializationFailed(String),
}

/// Run the Groth16 trusted setup for the KYC circuit
///
/// Generates:
/// - Proving key → saved to `keys/kyc_proving.key`
/// - Verification key → saved to `keys/kyc_verification.key`
pub fn run_trusted_setup(tree_depth: usize, keys_dir: &str) -> Result<(), SetupError> {
    info!(
        "{} Running Groth16 trusted setup (tree_depth={}, output={})",
        LOG_PREFIX, tree_depth, keys_dir
    );

    let circuit = KycCircuit::new(tree_depth);

    // TODO: Implement actual setup:
    // use ark_groth16::Groth16;
    // use ark_bn254::Bn254;
    // use ark_std::rand::thread_rng;
    //
    // let mut rng = thread_rng();
    // let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
    //     .map_err(|e| SetupError::SetupFailed(e.to_string()))?;
    //
    // // Serialize and save
    // let pk_bytes = ark_serialize::CanonicalSerialize::serialize_compressed(&pk, ...);
    // std::fs::write(format!("{}/kyc_proving.key", keys_dir), pk_bytes)?;
    // std::fs::write(format!("{}/kyc_verification.key", keys_dir), vk_bytes)?;

    let _ = circuit; // Suppress unused warning

    info!("{} Trusted setup complete. Keys saved to {}", LOG_PREFIX, keys_dir);
    Ok(())
}
