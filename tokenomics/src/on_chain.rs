// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! [K-RWA] On-chain staking via timelock covenant UTXOs on Kaspa TN12.
//!
//! Builds a CHECKSIG + CHECKLOCKTIMEVERIFY covenant script that locks funds
//! until a specific DAA score is reached. The script is deployed as a P2SH
//! UTXO on Kaspa Testnet-12.

use sha2::{Digest, Sha256};
use tracing::info;

use crate::staking::StakePosition;
use crate::LOG_PREFIX;

// Kaspa script opcodes
const OP_CHECKSIG: u8 = 0xac;
const OP_CHECKLOCKTIMEVERIFY: u8 = 0xb0;
const OP_DROP: u8 = 0x75;
const OP_VERIFY: u8 = 0x69;
const OP_TRUE: u8 = 0x51;

/// Push-data opcode for 32 bytes
const OP_DATA_32: u8 = 0x20;
/// Push-data opcode for 8 bytes
const OP_DATA_8: u8 = 0x08;

/// Kaspa testnet P2SH address prefix
const P2SH_PREFIX: &str = "kaspatest:p";

/// On-chain staking position backed by a timelock covenant UTXO.
///
/// The covenant uses a CHECKSIG + CHECKLOCKTIMEVERIFY pattern:
///   [push_32][pubkey] OP_CHECKSIG OP_VERIFY
///   [push_8][daa_score] OP_CHECKLOCKTIMEVERIFY OP_DROP
///   OP_TRUE
#[derive(Debug, Clone)]
pub struct OnChainStake {
    /// The in-memory staking position
    pub position: StakePosition,
    /// Raw covenant script bytes
    pub covenant_script: Vec<u8>,
    /// P2SH address derived from the covenant script hash
    pub p2sh_address: String,
    /// Transaction ID from deploying the covenant (None until broadcast)
    pub deploy_tx_id: Option<String>,
}

impl OnChainStake {
    /// Build the timelock covenant script.
    ///
    /// Layout (44 bytes total):
    /// ```text
    /// [0x20][32-byte pubkey] OP_CHECKSIG OP_VERIFY
    /// [0x08][8-byte LE daa_score] OP_CHECKLOCKTIMEVERIFY OP_DROP
    /// OP_TRUE
    /// ```
    ///
    /// Verification logic:
    /// 1. Push owner pubkey, verify Schnorr signature (CHECKSIG + VERIFY)
    /// 2. Push unlock DAA score, check block DAA >= score (CHECKLOCKTIMEVERIFY)
    /// 3. Drop the DAA score from the stack
    /// 4. Push TRUE to signal success
    pub fn build_covenant(owner_pubkey: &[u8; 32], unlock_daa_score: u64) -> Vec<u8> {
        let mut script = Vec::with_capacity(44);

        // [push_32][pubkey] OP_CHECKSIG OP_VERIFY
        script.push(OP_DATA_32);
        script.extend_from_slice(owner_pubkey);
        script.push(OP_CHECKSIG);
        script.push(OP_VERIFY);

        // [push_8][daa_score as LE u64] OP_CHECKLOCKTIMEVERIFY OP_DROP
        script.push(OP_DATA_8);
        script.extend_from_slice(&unlock_daa_score.to_le_bytes());
        script.push(OP_CHECKLOCKTIMEVERIFY);
        script.push(OP_DROP);

        // OP_TRUE
        script.push(OP_TRUE);

        script
    }

    /// Derive a P2SH address from a covenant script.
    ///
    /// Computes SHA-256 of the script and formats as a kaspatest P2SH address.
    /// Note: production would use blake2b and proper bech32 encoding.
    fn derive_p2sh_address(covenant_script: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(covenant_script);
        let hash = hasher.finalize();
        format!("{}:{}", P2SH_PREFIX, hex::encode(hash))
    }

    /// Create a new on-chain stake from an existing StakePosition.
    ///
    /// Computes the unlock DAA score from the lock duration. Uses an approximate
    /// conversion of 1 DAA score per second (Kaspa's block rate on TN12).
    pub fn new(position: StakePosition, owner_pubkey: [u8; 32]) -> Self {
        let lock_duration = position.lock_until.saturating_sub(position.staked_at);
        // Approximate: 1 DAA score ≈ 1 second on TN12
        let unlock_daa_score = lock_duration;

        info!(
            "{} Building on-chain stake: amount={}, unlock_daa_score={}",
            LOG_PREFIX, position.amount, unlock_daa_score
        );

        let covenant_script = Self::build_covenant(&owner_pubkey, unlock_daa_score);
        let p2sh_address = Self::derive_p2sh_address(&covenant_script);

        info!(
            "{} Covenant script: {} bytes, P2SH: {}",
            LOG_PREFIX,
            covenant_script.len(),
            p2sh_address
        );

        Self {
            position,
            covenant_script,
            p2sh_address,
            deploy_tx_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::staking::{create_stake, MIN_STAKE_AMOUNT};

    #[test]
    fn test_build_covenant_structure() {
        let pubkey = [0xAA; 32];
        let daa_score: u64 = 7_776_000; // ~90 days in seconds

        let script = OnChainStake::build_covenant(&pubkey, daa_score);

        // Total: 1 + 32 + 1 + 1 + 1 + 8 + 1 + 1 + 1 = 47 bytes
        // Wait — let's count: push32(1) + pubkey(32) + CHECKSIG(1) + VERIFY(1)
        //                    + push8(1) + daa(8) + CLTV(1) + DROP(1) + TRUE(1) = 47
        assert_eq!(script.len(), 47);

        // Verify opcode positions
        assert_eq!(script[0], OP_DATA_32, "first byte should be push-32");
        assert_eq!(&script[1..33], &[0xAA; 32], "bytes 1..33 should be pubkey");
        assert_eq!(script[33], OP_CHECKSIG, "byte 33 should be OP_CHECKSIG");
        assert_eq!(script[34], OP_VERIFY, "byte 34 should be OP_VERIFY");
        assert_eq!(script[35], OP_DATA_8, "byte 35 should be push-8");
        assert_eq!(
            &script[36..44],
            &daa_score.to_le_bytes(),
            "bytes 36..44 should be LE DAA score"
        );
        assert_eq!(
            script[44], OP_CHECKLOCKTIMEVERIFY,
            "byte 44 should be OP_CLTV"
        );
        assert_eq!(script[45], OP_DROP, "byte 45 should be OP_DROP");
        assert_eq!(script[46], OP_TRUE, "byte 46 should be OP_TRUE");
    }

    #[test]
    fn test_create_on_chain_stake() {
        // 1. Create a StakePosition (MIN_STAKE_AMOUNT = 100 ASTM, 90 days)
        let lock_90_days = 90 * 24 * 60 * 60;
        let position = create_stake("kaspatest:addr_test1", MIN_STAKE_AMOUNT, lock_90_days)
            .expect("should create valid stake");

        // 2. Build the on-chain stake with a test pubkey
        let owner_pubkey = [0x02; 32];
        let on_chain = OnChainStake::new(position, owner_pubkey);

        // 3. Verify covenant script was built
        assert_eq!(on_chain.covenant_script.len(), 47);

        // 4. Verify P2SH address was derived
        assert!(
            on_chain.p2sh_address.starts_with(P2SH_PREFIX),
            "P2SH address should start with testnet prefix"
        );

        // 5. deploy_tx_id should be None (not broadcast yet)
        assert!(on_chain.deploy_tx_id.is_none());
    }

    #[test]
    fn test_covenant_different_pubkeys_different_scripts() {
        let pubkey_a = [0x01; 32];
        let pubkey_b = [0x02; 32];
        let daa_score = 1_000_000u64;

        let script_a = OnChainStake::build_covenant(&pubkey_a, daa_score);
        let script_b = OnChainStake::build_covenant(&pubkey_b, daa_score);

        assert_ne!(
            script_a, script_b,
            "different pubkeys must produce different scripts"
        );
    }

    #[test]
    fn test_covenant_different_daa_scores_different_scripts() {
        let pubkey = [0x01; 32];

        let script_short = OnChainStake::build_covenant(&pubkey, 100_000);
        let script_long = OnChainStake::build_covenant(&pubkey, 9_000_000);

        assert_ne!(
            script_short, script_long,
            "different DAA scores must produce different scripts"
        );
    }

    #[test]
    fn test_p2sh_address_deterministic() {
        let pubkey = [0xFF; 32];
        let daa_score = 500_000u64;

        let script = OnChainStake::build_covenant(&pubkey, daa_score);
        let addr1 = OnChainStake::derive_p2sh_address(&script);
        let addr2 = OnChainStake::derive_p2sh_address(&script);

        assert_eq!(addr1, addr2, "same script must produce same P2SH address");
    }
}
