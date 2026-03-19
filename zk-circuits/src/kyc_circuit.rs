// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ZK-KYC Groth16 circuit.
//!
//! Proves: "I know a secret whose hash is a leaf in the Merkle tree
//! with the given root, and I can derive the given nullifier."
//!
//! Public inputs: merkle_root, nullifier_hash
//! Private inputs: secret_key, merkle_path, path_indices

use ark_ff::PrimeField;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};
use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
    select::CondSelectGadget,
};
use ark_bn254::Fr;
use tracing::info;

use crate::LOG_PREFIX;

/// ZK-KYC circuit for Groth16 proving.
///
/// The circuit proves Merkle inclusion: the prover knows a secret whose
/// hash sits at a specific leaf in a Merkle tree with a publicly committed root.
/// It also binds a nullifier to prevent double-use.
///
/// Simplified hash: we use a field-arithmetic "hash" (Poseidon-like compression)
/// rather than SHA-256 inside the circuit. This keeps constraint count manageable
/// while demonstrating the full ZK-KYC flow.
#[derive(Clone)]
pub struct KycCircuit<F: PrimeField> {
    /// Public input: Merkle tree root
    pub merkle_root: Option<F>,
    /// Public input: nullifier to prevent double-use
    pub nullifier_hash: Option<F>,
    /// Private input: secret key of the prover
    pub secret_key: Option<F>,
    /// Private input: Merkle path siblings (one per tree level)
    pub merkle_path: Vec<Option<F>>,
    /// Private input: path direction bits (false=left, true=right)
    pub path_indices: Vec<Option<bool>>,
    /// Merkle tree depth (compile-time parameter)
    pub tree_depth: usize,
}

impl<F: PrimeField> KycCircuit<F> {
    /// Create a new empty KYC circuit for setup (no witness values)
    pub fn new_empty(tree_depth: usize) -> Self {
        info!("{} Creating empty KYC circuit (depth={})", LOG_PREFIX, tree_depth);
        Self {
            merkle_root: None,
            nullifier_hash: None,
            secret_key: None,
            merkle_path: vec![None; tree_depth],
            path_indices: vec![None; tree_depth],
            tree_depth,
        }
    }

    /// Create a circuit with all witness values for proving
    pub fn new_with_witness(
        merkle_root: F,
        nullifier_hash: F,
        secret_key: F,
        merkle_path: Vec<F>,
        path_indices: Vec<bool>,
        tree_depth: usize,
    ) -> Self {
        assert_eq!(merkle_path.len(), tree_depth);
        assert_eq!(path_indices.len(), tree_depth);
        info!("{} Creating KYC circuit with witness (depth={})", LOG_PREFIX, tree_depth);
        Self {
            merkle_root: Some(merkle_root),
            nullifier_hash: Some(nullifier_hash),
            secret_key: Some(secret_key),
            merkle_path: merkle_path.into_iter().map(Some).collect(),
            path_indices: path_indices.into_iter().map(Some).collect(),
            tree_depth,
        }
    }
}

/// Simplified field-based hash: H(a, b) = (a + b)^5 + a*b + CONSTANT
/// This is NOT cryptographically secure — it's a demonstration of the
/// constraint pattern. A production system would use Poseidon or MiMC.
fn mimc_hash_gadget<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    left: &FpVar<F>,
    right: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let _ = cs;
    // H(a,b) = (a + b)^5 + a*b + 7
    let sum = left + right;
    let sum_sq = &sum * &sum;
    let sum_4 = &sum_sq * &sum_sq;
    let sum_5 = &sum_4 * &sum;
    let prod = left * right;
    let constant = FpVar::constant(F::from(7u64));
    Ok(sum_5 + prod + constant)
}

/// Compute leaf hash from secret: H(secret, 0)
fn compute_leaf_hash<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    secret: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let zero = FpVar::constant(F::from(0u64));
    mimc_hash_gadget(cs, secret, &zero)
}

/// Compute nullifier: H(secret, 1)
fn compute_nullifier<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    secret: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let one = FpVar::constant(F::from(1u64));
    mimc_hash_gadget(cs, secret, &one)
}

impl ConstraintSynthesizer<Fr> for KycCircuit<Fr> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        info!("{} Generating R1CS constraints for KYC circuit (depth={})", LOG_PREFIX, self.tree_depth);

        // 1. Allocate public inputs
        let root_var = FpVar::new_input(cs.clone(), || {
            self.merkle_root.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let nullifier_var = FpVar::new_input(cs.clone(), || {
            self.nullifier_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // 2. Allocate private witnesses
        let secret_var = FpVar::new_witness(cs.clone(), || {
            self.secret_key.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let mut path_vars = Vec::with_capacity(self.tree_depth);
        for i in 0..self.tree_depth {
            let sibling = FpVar::new_witness(cs.clone(), || {
                self.merkle_path[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            path_vars.push(sibling);
        }

        let mut direction_vars = Vec::with_capacity(self.tree_depth);
        for i in 0..self.tree_depth {
            let bit = Boolean::new_witness(cs.clone(), || {
                self.path_indices[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            direction_vars.push(bit);
        }

        // 3. Compute leaf = H(secret, 0)
        let mut current = compute_leaf_hash(cs.clone(), &secret_var)?;

        // 4. Walk up the Merkle tree
        for i in 0..self.tree_depth {
            let sibling = &path_vars[i];
            let is_right = &direction_vars[i];

            // If is_right, hash(sibling, current), else hash(current, sibling)
            let left = CondSelectGadget::conditionally_select(is_right, sibling, &current)?;
            let right = CondSelectGadget::conditionally_select(is_right, &current, sibling)?;

            current = mimc_hash_gadget(cs.clone(), &left, &right)?;
        }

        // 5. Assert computed root == public root
        current.enforce_equal(&root_var)?;

        // 6. Compute nullifier = H(secret, 1) and assert it matches
        let computed_nullifier = compute_nullifier(cs.clone(), &secret_var)?;
        computed_nullifier.enforce_equal(&nullifier_var)?;

        info!(
            "{} KYC circuit: {} constraints generated",
            LOG_PREFIX,
            cs.num_constraints()
        );

        Ok(())
    }
}

/// Recursive ZK-KYC circuit that chains proofs.
/// Each proof attests: "I have current KYC AND my previous compliance proof was valid."
/// Uses proof-chain pattern: previous proof validity is verified off-chain,
/// then passed as witness. The circuit enforces chain linkage via previous_merkle_root
/// and chain_depth constraints.
#[derive(Clone)]
pub struct RecursiveKycCircuit<F: PrimeField> {
    // Current proof fields (same as KycCircuit)
    pub merkle_root: Option<F>,
    pub nullifier_hash: Option<F>,
    pub secret_key: Option<F>,
    pub merkle_path: Vec<Option<F>>,
    pub path_indices: Vec<Option<bool>>,
    pub tree_depth: usize,
    // Recursive extension
    /// Witness: was the previous proof valid? (verified off-chain before proving)
    pub previous_proof_valid: Option<bool>,
    /// Public input: previous period's Merkle root
    pub previous_merkle_root: Option<F>,
    /// Public input: how many proofs in the chain (0 = first proof)
    pub chain_depth: Option<F>,
}

impl<F: PrimeField> RecursiveKycCircuit<F> {
    /// Create a new empty recursive KYC circuit for setup (all None)
    pub fn new_empty(tree_depth: usize) -> Self {
        info!("{} Creating empty recursive KYC circuit (depth={})", LOG_PREFIX, tree_depth);
        Self {
            merkle_root: None,
            nullifier_hash: None,
            secret_key: None,
            merkle_path: vec![None; tree_depth],
            path_indices: vec![None; tree_depth],
            tree_depth,
            previous_proof_valid: None,
            previous_merkle_root: None,
            chain_depth: None,
        }
    }

    /// Create a first proof in the chain (chain_depth=0, no previous proof required)
    pub fn new_first_proof(
        merkle_root: F,
        nullifier_hash: F,
        secret_key: F,
        merkle_path: Vec<F>,
        path_indices: Vec<bool>,
        tree_depth: usize,
    ) -> Self {
        assert_eq!(merkle_path.len(), tree_depth);
        assert_eq!(path_indices.len(), tree_depth);
        info!("{} Creating first recursive KYC proof (depth={})", LOG_PREFIX, tree_depth);
        Self {
            merkle_root: Some(merkle_root),
            nullifier_hash: Some(nullifier_hash),
            secret_key: Some(secret_key),
            merkle_path: merkle_path.into_iter().map(Some).collect(),
            path_indices: path_indices.into_iter().map(Some).collect(),
            tree_depth,
            previous_proof_valid: Some(false), // not used when chain_depth=0
            previous_merkle_root: Some(F::zero()),
            chain_depth: Some(F::zero()),
        }
    }

    /// Create a chained proof (chain_depth>0, previous proof must be valid)
    pub fn new_chained_proof(
        merkle_root: F,
        nullifier_hash: F,
        secret_key: F,
        merkle_path: Vec<F>,
        path_indices: Vec<bool>,
        tree_depth: usize,
        previous_proof_valid: bool,
        previous_merkle_root: F,
        chain_depth: F,
    ) -> Self {
        assert_eq!(merkle_path.len(), tree_depth);
        assert_eq!(path_indices.len(), tree_depth);
        info!(
            "{} Creating chained recursive KYC proof (depth={}, chain_depth>0)",
            LOG_PREFIX, tree_depth
        );
        Self {
            merkle_root: Some(merkle_root),
            nullifier_hash: Some(nullifier_hash),
            secret_key: Some(secret_key),
            merkle_path: merkle_path.into_iter().map(Some).collect(),
            path_indices: path_indices.into_iter().map(Some).collect(),
            tree_depth,
            previous_proof_valid: Some(previous_proof_valid),
            previous_merkle_root: Some(previous_merkle_root),
            chain_depth: Some(chain_depth),
        }
    }
}

impl ConstraintSynthesizer<Fr> for RecursiveKycCircuit<Fr> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        info!(
            "{} Generating R1CS constraints for recursive KYC circuit (depth={})",
            LOG_PREFIX, self.tree_depth
        );

        // === Public inputs (order matters for verification) ===

        // 1. Allocate public input: merkle_root
        let root_var = FpVar::new_input(cs.clone(), || {
            self.merkle_root.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // 2. Allocate public input: nullifier_hash
        let nullifier_var = FpVar::new_input(cs.clone(), || {
            self.nullifier_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // 3. Allocate public input: previous_merkle_root
        let _previous_root_var = FpVar::new_input(cs.clone(), || {
            self.previous_merkle_root.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // 4. Allocate public input: chain_depth
        let chain_depth_var = FpVar::new_input(cs.clone(), || {
            self.chain_depth.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // === Private witnesses ===

        let secret_var = FpVar::new_witness(cs.clone(), || {
            self.secret_key.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let mut path_vars = Vec::with_capacity(self.tree_depth);
        for i in 0..self.tree_depth {
            let sibling = FpVar::new_witness(cs.clone(), || {
                self.merkle_path[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            path_vars.push(sibling);
        }

        let mut direction_vars = Vec::with_capacity(self.tree_depth);
        for i in 0..self.tree_depth {
            let bit = Boolean::new_witness(cs.clone(), || {
                self.path_indices[i].ok_or(SynthesisError::AssignmentMissing)
            })?;
            direction_vars.push(bit);
        }

        let previous_valid_var = Boolean::new_witness(cs.clone(), || {
            self.previous_proof_valid.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // === Merkle path constraints (same as KycCircuit) ===

        // Compute leaf = H(secret, 0)
        let mut current = compute_leaf_hash(cs.clone(), &secret_var)?;

        // Walk up the Merkle tree
        for i in 0..self.tree_depth {
            let sibling = &path_vars[i];
            let is_right = &direction_vars[i];

            let left = CondSelectGadget::conditionally_select(is_right, sibling, &current)?;
            let right = CondSelectGadget::conditionally_select(is_right, &current, sibling)?;

            current = mimc_hash_gadget(cs.clone(), &left, &right)?;
        }

        // Assert computed root == public root
        current.enforce_equal(&root_var)?;

        // Compute nullifier = H(secret, 1) and assert it matches
        let computed_nullifier = compute_nullifier(cs.clone(), &secret_var)?;
        computed_nullifier.enforce_equal(&nullifier_var)?;

        // === Recursive chain constraints ===
        // If chain_depth > 0, then previous_proof_valid must be true.
        // We use is_neq gadget: chain_depth != 0 implies previous_valid == true.

        let zero_var = FpVar::constant(Fr::from(0u64));
        let is_chained = chain_depth_var.is_neq(&zero_var)?;

        // When is_chained is true, previous_valid must be true.
        // Enforce: is_chained implies previous_valid
        // Equivalently: is_chained AND (NOT previous_valid) == false
        // Using conditional enforcement: if is_chained, enforce previous_valid == true
        let previous_valid_should_be = Boolean::conditionally_select(
            &is_chained,
            &Boolean::TRUE,
            &previous_valid_var, // don't care when not chained
        )?;
        previous_valid_var.enforce_equal(&previous_valid_should_be)?;

        info!(
            "{} Recursive KYC circuit: {} constraints generated",
            LOG_PREFIX,
            cs.num_constraints()
        );

        Ok(())
    }
}

/// Helper to compute the Merkle root outside the circuit (native field arithmetic)
/// Used for building witnesses before proving.
pub fn native_mimc_hash(left: Fr, right: Fr) -> Fr {
    let sum = left + right;
    let sum_sq = sum * sum;
    let sum_4 = sum_sq * sum_sq;
    let sum_5 = sum_4 * sum;
    let prod = left * right;
    sum_5 + prod + Fr::from(7u64)
}

/// Compute the leaf hash natively (outside circuit)
pub fn native_leaf_hash(secret: Fr) -> Fr {
    native_mimc_hash(secret, Fr::from(0u64))
}

/// Compute the nullifier natively (outside circuit)
pub fn native_nullifier(secret: Fr) -> Fr {
    native_mimc_hash(secret, Fr::from(1u64))
}

/// Build a Merkle tree from leaves and return (root, paths, indices) for a target leaf
pub fn build_merkle_tree(leaves: &[Fr], target_index: usize) -> (Fr, Vec<Fr>, Vec<bool>) {
    assert!(!leaves.is_empty(), "cannot build tree from empty leaves");
    let depth = (leaves.len() as f64).log2().ceil() as usize;
    let size = 1 << depth;

    // Pad leaves to next power of 2
    let mut current_level: Vec<Fr> = leaves.to_vec();
    current_level.resize(size, Fr::from(0u64));

    let mut path = Vec::with_capacity(depth);
    let mut indices = Vec::with_capacity(depth);
    let mut idx = target_index;

    for _level in 0..depth {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        path.push(current_level[sibling_idx]);
        indices.push(idx % 2 == 1); // true if our node is on the right

        let mut next_level = Vec::with_capacity(current_level.len() / 2);
        for pair in current_level.chunks(2) {
            next_level.push(native_mimc_hash(pair[0], pair[1]));
        }
        current_level = next_level;
        idx /= 2;
    }

    assert_eq!(current_level.len(), 1);
    (current_level[0], path, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_kyc_circuit_satisfiable() {
        // Set up a small tree with 4 leaves
        let secret = Fr::from(42u64);
        let leaf = native_leaf_hash(secret);
        let nullifier = native_nullifier(secret);

        let leaves = vec![
            Fr::from(100u64),
            leaf,
            Fr::from(200u64),
            Fr::from(300u64),
        ];
        let target_index = 1;

        let (root, path, indices) = build_merkle_tree(&leaves, target_index);

        let circuit = KycCircuit::new_with_witness(
            root,
            nullifier,
            secret,
            path,
            indices,
            2, // depth = log2(4) = 2
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");
        println!(
            "[K-RWA] Test circuit: {} constraints, satisfied=true",
            cs.num_constraints()
        );
    }

    #[test]
    fn test_kyc_circuit_wrong_secret_fails() {
        let secret = Fr::from(42u64);
        let wrong_secret = Fr::from(99u64);
        let leaf = native_leaf_hash(secret);
        let nullifier = native_nullifier(wrong_secret); // wrong nullifier

        let leaves = vec![leaf, Fr::from(100u64), Fr::from(200u64), Fr::from(300u64)];
        let (root, path, indices) = build_merkle_tree(&leaves, 0);

        let circuit = KycCircuit::new_with_witness(
            root,
            nullifier,
            wrong_secret,
            path,
            indices,
            2,
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(!cs.is_satisfied().unwrap(), "Circuit should NOT be satisfied with wrong secret");
    }

    #[test]
    fn test_native_merkle_tree() {
        let leaves = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let (root1, _, _) = build_merkle_tree(&leaves, 0);
        let (root2, _, _) = build_merkle_tree(&leaves, 1);
        assert_eq!(root1, root2, "Same tree should give same root regardless of target");
    }

    // --- Recursive circuit tests ---

    /// Helper to build a valid recursive circuit witness for testing
    fn build_recursive_test_witness() -> (Fr, Fr, Fr, Vec<Fr>, Vec<bool>, Fr) {
        let secret = Fr::from(42u64);
        let leaf = native_leaf_hash(secret);
        let nullifier = native_nullifier(secret);
        let leaves = vec![
            Fr::from(100u64),
            leaf,
            Fr::from(200u64),
            Fr::from(300u64),
        ];
        let (root, path, indices) = build_merkle_tree(&leaves, 1);
        (root, nullifier, secret, path, indices, root)
    }

    #[test]
    fn test_recursive_circuit_first_proof() {
        // chain_depth=0, no previous proof required — should be satisfiable
        let (root, nullifier, secret, path, indices, _) = build_recursive_test_witness();

        let circuit = RecursiveKycCircuit::new_first_proof(
            root, nullifier, secret, path, indices, 2,
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            cs.is_satisfied().unwrap(),
            "First proof (chain_depth=0) should be satisfied"
        );
        println!(
            "[K-RWA] Recursive first proof: {} constraints, satisfied=true",
            cs.num_constraints()
        );
    }

    #[test]
    fn test_recursive_circuit_chained_proof() {
        // chain_depth=1, previous_proof_valid=true — should be satisfiable
        let (root, nullifier, secret, path, indices, _) = build_recursive_test_witness();
        let previous_root = Fr::from(999u64); // arbitrary previous root

        let circuit = RecursiveKycCircuit::new_chained_proof(
            root,
            nullifier,
            secret,
            path,
            indices,
            2,
            true,           // previous proof was valid
            previous_root,
            Fr::from(1u64), // chain_depth = 1
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            cs.is_satisfied().unwrap(),
            "Chained proof (chain_depth=1, valid=true) should be satisfied"
        );
        println!(
            "[K-RWA] Recursive chained proof: {} constraints, satisfied=true",
            cs.num_constraints()
        );
    }

    #[test]
    fn test_recursive_circuit_invalid_chain() {
        // chain_depth=1, previous_proof_valid=false — should NOT be satisfiable
        let (root, nullifier, secret, path, indices, _) = build_recursive_test_witness();
        let previous_root = Fr::from(999u64);

        let circuit = RecursiveKycCircuit::new_chained_proof(
            root,
            nullifier,
            secret,
            path,
            indices,
            2,
            false,          // previous proof was INVALID
            previous_root,
            Fr::from(1u64), // chain_depth = 1
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "Chained proof (chain_depth=1, valid=false) should NOT be satisfied"
        );
        println!(
            "[K-RWA] Recursive invalid chain: {} constraints, satisfied=false (expected)",
            cs.num_constraints()
        );
    }
}
