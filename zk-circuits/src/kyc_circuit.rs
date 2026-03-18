// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ZK-KYC Groth16 circuit.
//!
//! Proves: "I know a private key whose public key hash is a leaf
//! in the Merkle tree with the given root."
//!
//! Public inputs: merkle_root, nullifier_hash
//! Private inputs: private_key, merkle_path, leaf_index

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::Fr;
use tracing::info;

use crate::LOG_PREFIX;

/// ZK-KYC circuit for Groth16 proving
#[derive(Clone)]
pub struct KycCircuit {
    /// Public input: Merkle tree root hash
    pub merkle_root: Option<Fr>,
    /// Public input: nullifier to prevent double-use
    pub nullifier_hash: Option<Fr>,
    /// Private input: secret key of the prover
    pub secret_key: Option<Fr>,
    /// Private input: Merkle path siblings
    pub merkle_path: Option<Vec<Fr>>,
    /// Private input: leaf index path directions
    pub path_indices: Option<Vec<bool>>,
    /// Merkle tree depth
    pub tree_depth: usize,
}

impl KycCircuit {
    /// Create a new KYC circuit with the given tree depth
    pub fn new(tree_depth: usize) -> Self {
        info!("{} Creating KYC circuit with depth={}", LOG_PREFIX, tree_depth);
        Self {
            merkle_root: None,
            nullifier_hash: None,
            secret_key: None,
            merkle_path: None,
            path_indices: None,
            tree_depth,
        }
    }
}

impl ConstraintSynthesizer<Fr> for KycCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        info!("{} Generating R1CS constraints for KYC circuit", LOG_PREFIX);

        // TODO: Implement full constraint system:
        //
        // 1. Allocate public inputs:
        //    - merkle_root (public)
        //    - nullifier_hash (public)
        //
        // 2. Allocate private witnesses:
        //    - secret_key (private)
        //    - merkle_path[i] for i in 0..tree_depth (private)
        //    - path_indices[i] for i in 0..tree_depth (private)
        //
        // 3. Constraints:
        //    a. Compute leaf = SHA256(secret_key)
        //    b. For each level i in 0..tree_depth:
        //       - If path_indices[i] == 0: hash = SHA256(current || merkle_path[i])
        //       - If path_indices[i] == 1: hash = SHA256(merkle_path[i] || current)
        //    c. Assert: computed_root == merkle_root
        //    d. Compute nullifier = SHA256(secret_key || external_nullifier)
        //    e. Assert: computed_nullifier == nullifier_hash
        //
        // This uses ark-r1cs-std for gadgets and ark-crypto-primitives for hashing

        let _ = cs; // Suppress unused warning for now
        Ok(())
    }
}
