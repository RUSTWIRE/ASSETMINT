// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Groth16 trusted setup for ZK-KYC circuits.
//! Generates proving and verification keys.
//!
//! NOTE: For testnet demo, uses deterministic setup with a fixed seed.
//! Production MUST use multi-party computation (MPC) ceremony.

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use ark_std::rand::SeedableRng;
use rand::rngs::StdRng;
use thiserror::Error;
use tracing::info;

use crate::kyc_circuit::{KycCircuit, RecursiveKycCircuit};
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("[K-RWA] Setup failed: {0}")]
    SetupFailed(String),
    #[error("[K-RWA] Key serialization failed: {0}")]
    SerializationFailed(String),
    #[error("[K-RWA] IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result of a trusted setup: proving key + verification key
pub struct SetupKeys {
    pub proving_key: ProvingKey<Bn254>,
    pub verifying_key: VerifyingKey<Bn254>,
}

/// Run the Groth16 trusted setup for the KYC circuit.
///
/// Uses a deterministic seed for testnet reproducibility.
/// Production deployments MUST use a proper MPC ceremony.
///
/// Returns serialized keys saved to disk, and the keys in memory.
pub fn run_trusted_setup(tree_depth: usize, keys_dir: &str) -> Result<SetupKeys, SetupError> {
    info!(
        "{} Running Groth16 trusted setup (tree_depth={}, output={})",
        LOG_PREFIX, tree_depth, keys_dir
    );

    // Create empty circuit (no witness) for setup
    let circuit: KycCircuit<Fr> = KycCircuit::new_empty(tree_depth);

    // Deterministic RNG for testnet reproducibility
    // WARNING: Not secure for production — use MPC ceremony
    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_BABE);

    let start = std::time::Instant::now();

    let (pk, vk): (ProvingKey<Bn254>, VerifyingKey<Bn254>) =
        Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e: ark_relations::r1cs::SynthesisError| SetupError::SetupFailed(e.to_string()))?;

    let elapsed = start.elapsed();
    info!("{} Setup complete in {:?}", LOG_PREFIX, elapsed);

    // Ensure output directory exists
    std::fs::create_dir_all(keys_dir)?;

    // Serialize proving key
    let pk_path = format!("{}/kyc_proving.key", keys_dir);
    let mut pk_bytes: Vec<u8> = Vec::new();
    CanonicalSerialize::serialize_compressed(&pk, &mut pk_bytes)
        .map_err(|e: ark_serialize::SerializationError| SetupError::SerializationFailed(e.to_string()))?;
    std::fs::write(&pk_path, &pk_bytes)?;
    info!(
        "{} Proving key saved: {} ({} bytes)",
        LOG_PREFIX,
        pk_path,
        pk_bytes.len()
    );

    // Serialize verification key
    let vk_path = format!("{}/kyc_verification.key", keys_dir);
    let mut vk_bytes: Vec<u8> = Vec::new();
    CanonicalSerialize::serialize_compressed(&vk, &mut vk_bytes)
        .map_err(|e: ark_serialize::SerializationError| SetupError::SerializationFailed(e.to_string()))?;
    std::fs::write(&vk_path, &vk_bytes)?;
    info!(
        "{} Verification key saved: {} ({} bytes)",
        LOG_PREFIX,
        vk_path,
        vk_bytes.len()
    );

    Ok(SetupKeys {
        proving_key: pk,
        verifying_key: vk,
    })
}

/// Run the Groth16 trusted setup for the Recursive KYC circuit.
///
/// Same pattern as `run_trusted_setup` but uses `RecursiveKycCircuit::new_empty`.
/// Saves keys as `recursive_kyc_proving.key` and `recursive_kyc_verification.key`.
pub fn run_recursive_trusted_setup(tree_depth: usize, keys_dir: &str) -> Result<SetupKeys, SetupError> {
    info!(
        "{} Running recursive Groth16 trusted setup (tree_depth={}, output={})",
        LOG_PREFIX, tree_depth, keys_dir
    );

    // Create empty recursive circuit (no witness) for setup
    let circuit: RecursiveKycCircuit<Fr> = RecursiveKycCircuit::new_empty(tree_depth);

    // Deterministic RNG for testnet reproducibility
    // WARNING: Not secure for production — use MPC ceremony
    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_BABE);

    let start = std::time::Instant::now();

    let (pk, vk): (ProvingKey<Bn254>, VerifyingKey<Bn254>) =
        Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e: ark_relations::r1cs::SynthesisError| SetupError::SetupFailed(e.to_string()))?;

    let elapsed = start.elapsed();
    info!("{} Recursive setup complete in {:?}", LOG_PREFIX, elapsed);

    // Ensure output directory exists
    std::fs::create_dir_all(keys_dir)?;

    // Serialize proving key
    let pk_path = format!("{}/recursive_kyc_proving.key", keys_dir);
    let mut pk_bytes: Vec<u8> = Vec::new();
    CanonicalSerialize::serialize_compressed(&pk, &mut pk_bytes)
        .map_err(|e: ark_serialize::SerializationError| SetupError::SerializationFailed(e.to_string()))?;
    std::fs::write(&pk_path, &pk_bytes)?;
    info!(
        "{} Recursive proving key saved: {} ({} bytes)",
        LOG_PREFIX,
        pk_path,
        pk_bytes.len()
    );

    // Serialize verification key
    let vk_path = format!("{}/recursive_kyc_verification.key", keys_dir);
    let mut vk_bytes: Vec<u8> = Vec::new();
    CanonicalSerialize::serialize_compressed(&vk, &mut vk_bytes)
        .map_err(|e: ark_serialize::SerializationError| SetupError::SerializationFailed(e.to_string()))?;
    std::fs::write(&vk_path, &vk_bytes)?;
    info!(
        "{} Recursive verification key saved: {} ({} bytes)",
        LOG_PREFIX,
        vk_path,
        vk_bytes.len()
    );

    Ok(SetupKeys {
        proving_key: pk,
        verifying_key: vk,
    })
}

/// Load a proving key from file
pub fn load_proving_key(path: &str) -> Result<ProvingKey<Bn254>, SetupError> {
    info!("{} Loading proving key from: {}", LOG_PREFIX, path);
    let bytes = std::fs::read(path)?;
    ProvingKey::<Bn254>::deserialize_compressed(&bytes[..])
        .map_err(|e| SetupError::SerializationFailed(e.to_string()))
}

/// Load a verification key from file
pub fn load_verifying_key(path: &str) -> Result<VerifyingKey<Bn254>, SetupError> {
    info!("{} Loading verification key from: {}", LOG_PREFIX, path);
    let bytes = std::fs::read(path)?;
    VerifyingKey::<Bn254>::deserialize_compressed(&bytes[..])
        .map_err(|e| SetupError::SerializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trusted_setup_runs() {
        let dir = "/tmp/assetmint_test_keys";
        let _ = std::fs::remove_dir_all(dir);
        let result = run_trusted_setup(2, dir);
        assert!(result.is_ok(), "Trusted setup should succeed");
        let keys = result.unwrap();
        assert!(!keys.proving_key.vk.gamma_abc_g1.is_empty());
        // Clean up
        let _ = std::fs::remove_dir_all(dir);
    }
}
