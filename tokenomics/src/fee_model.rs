// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Fee model for AssetMint platform.
//! Target: ≤0.001 KAS per transfer.

use tracing::info;
use crate::LOG_PREFIX;

/// Fee distribution percentages
pub struct FeeDistribution {
    /// Percentage burned
    pub burn_pct: u8,
    /// Percentage to stakers
    pub staker_pct: u8,
    /// Percentage to treasury
    pub treasury_pct: u8,
}

/// Default fee distribution
pub fn default_distribution() -> FeeDistribution {
    FeeDistribution {
        burn_pct: 30,
        staker_pct: 50,
        treasury_pct: 20,
    }
}

/// Calculate platform fee for a transfer
/// Target: ≤0.001 KAS = 100,000 sompis
pub fn calculate_fee(transfer_amount: u64) -> u64 {
    info!("{} Calculating platform fee for amount={}", LOG_PREFIX, transfer_amount);
    // Flat fee: 100,000 sompis = 0.001 KAS
    let flat_fee = 100_000u64;
    // Plus 0.01% of transfer amount
    let proportional_fee = transfer_amount / 10_000;
    let total = flat_fee + proportional_fee;
    info!("{} Fee calculated: {} sompis ({:.6} KAS)", LOG_PREFIX, total, total as f64 / 100_000_000.0);
    total
}
