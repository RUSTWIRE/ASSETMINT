// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Groth16 ZK-KYC proof verification.
//! Verifies that a Groth16 proof is valid against the verification key.
//!
//! Target: verification <50ms

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use ark_snark::SNARK;
use thiserror::Error;
use tracing::info;

use crate::zk_prover::ZkProof;
use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("[K-RWA] Verification failed: {0}")]
    VerificationFailed(String),
    #[error("[K-RWA] Invalid proof format: {0}")]
    InvalidProof(String),
    #[error("[K-RWA] Verification key not loaded: {0}")]
    KeyNotLoaded(String),
    #[error("[K-RWA] IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Groth16 ZK-KYC verifier
pub struct ZkVerifier {
    /// Deserialized verification key
    verifying_key: Option<VerifyingKey<Bn254>>,
}

impl ZkVerifier {
    /// Create a new ZK verifier
    pub fn new() -> Self {
        info!("{} Initializing ZK-KYC verifier (Groth16/BN254)", LOG_PREFIX);
        Self {
            verifying_key: None,
        }
    }

    /// Load verification key from file
    pub fn load_verification_key(&mut self, path: &str) -> Result<(), VerifierError> {
        info!("{} Loading verification key from: {}", LOG_PREFIX, path);
        let key_bytes = std::fs::read(path)?;
        let vk = VerifyingKey::<Bn254>::deserialize_compressed(&key_bytes[..])
            .map_err(|e| VerifierError::KeyNotLoaded(format!("deserialization: {}", e)))?;
        self.verifying_key = Some(vk);
        info!("{} Verification key loaded successfully", LOG_PREFIX);
        Ok(())
    }

    /// Set verification key directly (from in-memory setup)
    pub fn set_verifying_key(&mut self, vk: VerifyingKey<Bn254>) {
        self.verifying_key = Some(vk);
    }

    /// Verify a Groth16 ZK-KYC proof.
    ///
    /// Returns `Ok(true)` if the proof is valid, `Ok(false)` if it is not,
    /// or `Err` if verification could not be performed.
    ///
    /// Target: <50ms
    pub fn verify(&self, proof: &ZkProof) -> Result<bool, VerifierError> {
        info!("{} Verifying Groth16 ZK-KYC proof", LOG_PREFIX);

        let vk = self.verifying_key.as_ref().ok_or_else(|| {
            VerifierError::KeyNotLoaded("Verification key not loaded".into())
        })?;

        let start = std::time::Instant::now();

        // Deserialize the Groth16 proof
        let groth_proof = Proof::<Bn254>::deserialize_compressed(&proof.proof_bytes[..])
            .map_err(|e| VerifierError::InvalidProof(format!("proof deserialization: {}", e)))?;

        // Deserialize public inputs
        if proof.public_inputs.len() != 2 {
            return Err(VerifierError::InvalidProof(format!(
                "expected 2 public inputs, got {}",
                proof.public_inputs.len()
            )));
        }

        let merkle_root = Fr::deserialize_compressed(&proof.public_inputs[0][..])
            .map_err(|e| VerifierError::InvalidProof(format!("root deserialization: {}", e)))?;
        let nullifier = Fr::deserialize_compressed(&proof.public_inputs[1][..])
            .map_err(|e| {
                VerifierError::InvalidProof(format!("nullifier deserialization: {}", e))
            })?;

        let public_inputs: Vec<Fr> = vec![merkle_root, nullifier];

        // Prepare verifying key for verification
        let pvk = ark_groth16::prepare_verifying_key(vk);

        // Verify the proof
        let valid = Groth16::<Bn254>::verify_with_processed_vk(&pvk, &public_inputs, &groth_proof)
            .map_err(|e: ark_relations::r1cs::SynthesisError| VerifierError::VerificationFailed(e.to_string()))?;

        let elapsed = start.elapsed();
        info!(
            "{} ZK proof verification: valid={}, elapsed={:?}",
            LOG_PREFIX, valid, elapsed
        );

        Ok(valid)
    }

    /// Compute the hash of the verification key (for on-chain commitment)
    pub fn verification_key_hash(&self) -> Result<[u8; 32], VerifierError> {
        let vk = self.verifying_key.as_ref().ok_or_else(|| {
            VerifierError::KeyNotLoaded("Verification key not loaded".into())
        })?;

        use ark_serialize::CanonicalSerialize;
        use sha2::{Digest, Sha256};

        let mut vk_bytes = Vec::new();
        vk.serialize_compressed(&mut vk_bytes)
            .map_err(|e| VerifierError::KeyNotLoaded(format!("vk serialization: {}", e)))?;
        Ok(Sha256::digest(&vk_bytes).into())
    }
}

impl Default for ZkVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk_prover::{ZkProver, ZkWitness};
    use zk_circuits::kyc_circuit::native_leaf_hash;
    use zk_circuits::setup::run_trusted_setup;

    #[test]
    fn test_full_prove_verify_cycle() {
        // 1. Trusted setup
        let keys_dir = "/tmp/assetmint_verifier_test";
        let _ = std::fs::remove_dir_all(keys_dir);
        let setup = run_trusted_setup(2, keys_dir).expect("setup should succeed");

        // 2. Create prover + verifier
        let mut prover = ZkProver::new(2);
        prover.set_proving_key(setup.proving_key);

        let mut verifier = ZkVerifier::new();
        verifier.set_verifying_key(setup.verifying_key);

        // 3. Create witness
        let secret = ark_bn254::Fr::from(42u64);
        let leaf = native_leaf_hash(secret);
        let leaves = vec![
            ark_bn254::Fr::from(100u64),
            leaf,
            ark_bn254::Fr::from(200u64),
            ark_bn254::Fr::from(300u64),
        ];

        let witness = ZkWitness {
            secret_key: {
                let mut bytes = Vec::new();
                ark_serialize::CanonicalSerialize::serialize_compressed(&secret, &mut bytes)
                    .unwrap();
                bytes
            },
            leaf_index: 1,
            all_leaves: leaves
                .iter()
                .map(|l| {
                    let mut bytes = Vec::new();
                    ark_serialize::CanonicalSerialize::serialize_compressed(l, &mut bytes).unwrap();
                    bytes
                })
                .collect(),
        };

        // 4. Generate proof
        let proof = prover.generate_proof(&witness).expect("proof gen should succeed");
        assert!(!proof.proof_bytes.is_empty());

        // 5. Verify proof
        let valid = verifier.verify(&proof).expect("verification should succeed");
        assert!(valid, "Valid proof should verify");

        // 6. Verify proof hash is non-zero
        assert_ne!(proof.proof_hash, [0u8; 32]);

        // 7. Verify vk hash
        let vk_hash = verifier.verification_key_hash().unwrap();
        assert_ne!(vk_hash, [0u8; 32]);

        // Clean up
        let _ = std::fs::remove_dir_all(keys_dir);
    }
}
