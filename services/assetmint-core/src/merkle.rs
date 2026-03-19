// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Merkle tree of approved addresses for on-chain ZK/Merkle verification.
//! SHA-256 hashing consistent with SilverScript's OP_SHA256.

use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum MerkleError {
    #[error("[K-RWA] Address not in tree: {0}")]
    AddressNotFound(String),
    #[error("[K-RWA] Tree is empty")]
    EmptyTree,
}

/// A Merkle proof path for inclusion verification
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// Leaf hash
    pub leaf: [u8; 32],
    /// Sibling hashes from leaf to root
    pub path: Vec<[u8; 32]>,
    /// Direction flags (false = left, true = right)
    pub directions: Vec<bool>,
}

/// Merkle tree for KYC-approved addresses
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    tree: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build a Merkle tree from a list of approved addresses
    pub fn build(addresses: &[String]) -> Result<Self, MerkleError> {
        info!(
            "{} Building Merkle tree with {} leaves",
            LOG_PREFIX,
            addresses.len()
        );

        if addresses.is_empty() {
            return Err(MerkleError::EmptyTree);
        }

        // Hash each address to create leaves
        let leaves: Vec<[u8; 32]> = addresses
            .iter()
            .map(|addr| {
                let mut hasher = Sha256::new();
                hasher.update(addr.as_bytes());
                hasher.finalize().into()
            })
            .collect();

        // Pad to power of 2
        let mut padded = leaves.clone();
        while padded.len().count_ones() != 1 {
            padded.push([0u8; 32]);
        }

        // Build tree bottom-up
        let mut tree = vec![padded.clone()];
        let mut current = padded;

        while current.len() > 1 {
            let mut next = Vec::new();
            for chunk in current.chunks(2) {
                let mut hasher = Sha256::new();
                if chunk[0] <= chunk[1] {
                    hasher.update(chunk[0]);
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[1]);
                    hasher.update(chunk[0]);
                }
                next.push(hasher.finalize().into());
            }
            tree.push(next.clone());
            current = next;
        }

        info!(
            "{} Merkle tree built: root={}",
            LOG_PREFIX,
            hex::encode(current[0])
        );

        Ok(Self { leaves, tree })
    }

    /// Get the Merkle root hash
    pub fn root(&self) -> [u8; 32] {
        self.tree.last().map(|level| level[0]).unwrap_or([0u8; 32])
    }

    /// Generate a Merkle proof for an address
    pub fn get_proof(&self, address: &str) -> Result<MerkleProof, MerkleError> {
        info!("{} Generating Merkle proof for {}", LOG_PREFIX, address);

        let mut hasher = Sha256::new();
        hasher.update(address.as_bytes());
        let leaf: [u8; 32] = hasher.finalize().into();

        let index = self.tree[0]
            .iter()
            .position(|l| *l == leaf)
            .ok_or_else(|| MerkleError::AddressNotFound(address.to_string()))?;

        let mut path = Vec::new();
        let mut directions = Vec::new();
        let mut idx = index;

        for level in &self.tree[..self.tree.len() - 1] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            if sibling_idx < level.len() {
                path.push(level[sibling_idx]);
                directions.push(idx % 2 != 0);
            }
            idx /= 2;
        }

        Ok(MerkleProof {
            leaf,
            path,
            directions,
        })
    }

    /// Verify a Merkle proof against a root
    pub fn verify_proof(proof: &MerkleProof, root: &[u8; 32]) -> bool {
        let mut current = proof.leaf;

        for sibling in &proof.path {
            let mut hasher = Sha256::new();
            // Sort pair to match build() ordering
            if current <= *sibling {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }
            current = hasher.finalize().into();
        }

        current == *root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_build_and_verify() {
        let addresses = vec![
            "kaspatest:addr1".to_string(),
            "kaspatest:addr2".to_string(),
            "kaspatest:addr3".to_string(),
            "kaspatest:addr4".to_string(),
        ];

        let tree = MerkleTree::build(&addresses).unwrap();
        let root = tree.root();

        // Verify each address has a valid proof
        for addr in &addresses {
            let proof = tree.get_proof(addr).unwrap();
            assert!(MerkleTree::verify_proof(&proof, &root));
        }
    }

    #[test]
    fn test_merkle_invalid_address() {
        let addresses = vec!["kaspatest:addr1".to_string()];
        let tree = MerkleTree::build(&addresses).unwrap();
        assert!(tree.get_proof("kaspatest:unknown").is_err());
    }
}
