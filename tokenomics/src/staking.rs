// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM staking mechanism.
//! Lock ASTM in covenant UTXO with time-lock, earn rewards.

use tracing::info;
use crate::LOG_PREFIX;

/// Staking position
#[derive(Debug, Clone)]
pub struct StakePosition {
    pub staker_address: String,
    pub amount: u64,
    pub lock_until: u64,
    pub covenant_utxo_txid: String,
    pub covenant_utxo_index: u32,
}

/// Calculate staking rewards based on lock duration
pub fn calculate_rewards(amount: u64, lock_duration_secs: u64) -> u64 {
    info!("{} Calculating staking rewards: amount={}, duration={}s",
        LOG_PREFIX, amount, lock_duration_secs);
    // Simple linear reward: 5% APY
    let annual_rate = 5u64; // 5%
    let seconds_per_year = 365 * 24 * 60 * 60;
    (amount * annual_rate * lock_duration_secs) / (100 * seconds_per_year)
}
