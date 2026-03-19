// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM staking mechanism.
//! Lock ASTM in covenant UTXO with time-lock, earn rewards.
//! Unstaking spends the covenant after the timelock expires.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum StakingError {
    #[error("[K-RWA] Stake amount below minimum: {amount} < {minimum}")]
    BelowMinimum { amount: u64, minimum: u64 },
    #[error("[K-RWA] Lock period too short: {duration}s < {minimum}s")]
    LockTooShort { duration: u64, minimum: u64 },
    #[error("[K-RWA] Stake still locked: {remaining}s remaining")]
    StillLocked { remaining: u64 },
}

/// Minimum stake amount (100 ASTM in sompis)
pub const MIN_STAKE_AMOUNT: u64 = 100_0000_0000; // 100 * 10^8
/// Minimum lock period: 7 days
pub const MIN_LOCK_PERIOD: u64 = 7 * 24 * 60 * 60;
/// Annual reward rate basis points (500 = 5%)
pub const REWARD_RATE_BPS: u64 = 500;
/// Seconds per year
const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

/// Staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePosition {
    /// Staker's Kaspa address
    pub staker_address: String,
    /// Amount staked (in sompis)
    pub amount: u64,
    /// Unix timestamp when the stake was created
    pub staked_at: u64,
    /// Unix timestamp when the lock expires
    pub lock_until: u64,
    /// UTXO holding the staked funds
    pub covenant_utxo_txid: String,
    pub covenant_utxo_index: u32,
}

/// Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    /// Total ASTM staked across all positions
    pub total_staked: u64,
    /// Number of active stake positions
    pub position_count: u64,
    /// Accumulated rewards available for distribution
    pub reward_pool: u64,
}

/// Create a new stake position
pub fn create_stake(
    staker_address: &str,
    amount: u64,
    lock_duration_secs: u64,
) -> Result<StakePosition, StakingError> {
    info!(
        "{} Creating stake: address={}, amount={}, lock={}s",
        LOG_PREFIX, staker_address, amount, lock_duration_secs
    );

    if amount < MIN_STAKE_AMOUNT {
        return Err(StakingError::BelowMinimum {
            amount,
            minimum: MIN_STAKE_AMOUNT,
        });
    }

    if lock_duration_secs < MIN_LOCK_PERIOD {
        return Err(StakingError::LockTooShort {
            duration: lock_duration_secs,
            minimum: MIN_LOCK_PERIOD,
        });
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let position = StakePosition {
        staker_address: staker_address.to_string(),
        amount,
        staked_at: now,
        lock_until: now + lock_duration_secs,
        covenant_utxo_txid: String::new(), // Set after tx broadcast
        covenant_utxo_index: 0,
    };

    info!(
        "{} Stake position created: lock_until={}",
        LOG_PREFIX, position.lock_until
    );
    Ok(position)
}

/// Calculate staking rewards based on lock duration and amount
pub fn calculate_rewards(amount: u64, lock_duration_secs: u64) -> u64 {
    info!(
        "{} Calculating staking rewards: amount={}, duration={}s",
        LOG_PREFIX, amount, lock_duration_secs
    );
    // Reward = amount * rate * duration / (10000 * seconds_per_year)
    let reward = (amount as u128 * REWARD_RATE_BPS as u128 * lock_duration_secs as u128)
        / (10_000u128 * SECONDS_PER_YEAR as u128);
    let reward = reward as u64;
    info!("{} Calculated reward: {} sompis", LOG_PREFIX, reward);
    reward
}

/// Check if a stake position can be unstaked
pub fn can_unstake(position: &StakePosition) -> Result<bool, StakingError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now < position.lock_until {
        return Err(StakingError::StillLocked {
            remaining: position.lock_until - now,
        });
    }

    Ok(true)
}

/// Compute the covenant script hash for a staking UTXO
/// This is the P2SH address the staked funds are locked to
pub fn staking_covenant_hash(staker_address: &str, lock_until: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ASTM_STAKE_V1");
    hasher.update(staker_address.as_bytes());
    hasher.update(lock_until.to_le_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_stake() {
        let pos = create_stake("kaspatest:addr1", MIN_STAKE_AMOUNT, MIN_LOCK_PERIOD).unwrap();
        assert_eq!(pos.staker_address, "kaspatest:addr1");
        assert_eq!(pos.amount, MIN_STAKE_AMOUNT);
        assert!(pos.lock_until > pos.staked_at);
    }

    #[test]
    fn test_stake_below_minimum() {
        let err = create_stake("kaspatest:addr1", 100, MIN_LOCK_PERIOD);
        assert!(err.is_err());
    }

    #[test]
    fn test_stake_lock_too_short() {
        let err = create_stake("kaspatest:addr1", MIN_STAKE_AMOUNT, 60);
        assert!(err.is_err());
    }

    #[test]
    fn test_calculate_rewards() {
        // 10000 ASTM staked for 1 year at 5% = 500 ASTM
        let amount = 10_000_0000_0000u64; // 10000 * 10^8
        let reward = calculate_rewards(amount, SECONDS_PER_YEAR);
        let expected = amount * REWARD_RATE_BPS / 10_000; // 5% of amount
        assert_eq!(reward, expected);
    }

    #[test]
    fn test_calculate_rewards_partial_year() {
        // Half year = half reward
        let amount = 10_000_0000_0000u64;
        let full_year = calculate_rewards(amount, SECONDS_PER_YEAR);
        let half_year = calculate_rewards(amount, SECONDS_PER_YEAR / 2);
        // Allow rounding difference of 1
        assert!((half_year as i64 - (full_year / 2) as i64).unsigned_abs() <= 1);
    }

    #[test]
    fn test_staking_covenant_hash() {
        let h1 = staking_covenant_hash("kaspatest:addr1", 1000);
        let h2 = staking_covenant_hash("kaspatest:addr2", 1000);
        assert_ne!(h1, h2); // Different addresses = different hashes
    }

    #[test]
    fn test_can_unstake_locked() {
        let pos = create_stake("kaspatest:addr1", MIN_STAKE_AMOUNT, MIN_LOCK_PERIOD).unwrap();
        // Just created, should still be locked
        assert!(can_unstake(&pos).is_err());
    }

    #[test]
    fn test_can_unstake_expired() {
        let mut pos =
            create_stake("kaspatest:addr1", MIN_STAKE_AMOUNT, MIN_LOCK_PERIOD).unwrap();
        // Set lock_until to the past
        pos.lock_until = 1000;
        assert!(can_unstake(&pos).is_ok());
    }
}
