// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Fee model for AssetMint platform.
//! Target: ≤0.001 KAS per transfer.
//! Fee distribution: burn + staker rewards + treasury.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::LOG_PREFIX;

/// 1 KAS = 100,000,000 sompis
pub const SOMPIS_PER_KAS: u64 = 100_000_000;
/// Maximum platform fee: 0.001 KAS = 100,000 sompis
pub const MAX_FEE_SOMPIS: u64 = 100_000;
/// Flat fee component: 50,000 sompis = 0.0005 KAS
pub const FLAT_FEE_SOMPIS: u64 = 50_000;
/// Proportional fee: 1 basis point (0.01%) of transfer amount
pub const PROPORTIONAL_BPS: u64 = 1;

/// Fee distribution percentages (must sum to 100)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeDistribution {
    /// Percentage burned (deflationary pressure)
    pub burn_pct: u8,
    /// Percentage to stakers (staking rewards)
    pub staker_pct: u8,
    /// Percentage to treasury (platform ops)
    pub treasury_pct: u8,
}

/// Breakdown of a calculated fee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    /// Total fee in sompis
    pub total_sompis: u64,
    /// Total fee in KAS
    pub total_kas: f64,
    /// Amount burned
    pub burn_sompis: u64,
    /// Amount to stakers
    pub staker_sompis: u64,
    /// Amount to treasury
    pub treasury_sompis: u64,
    /// Whether the fee was capped at MAX_FEE
    pub was_capped: bool,
}

/// Default fee distribution: 30% burn, 50% stakers, 20% treasury
pub fn default_distribution() -> FeeDistribution {
    FeeDistribution {
        burn_pct: 30,
        staker_pct: 50,
        treasury_pct: 20,
    }
}

/// Calculate platform fee for a transfer (capped at MAX_FEE_SOMPIS)
pub fn calculate_fee(transfer_amount: u64) -> u64 {
    let proportional = (transfer_amount as u128 * PROPORTIONAL_BPS as u128 / 10_000) as u64;
    let raw_fee = FLAT_FEE_SOMPIS.saturating_add(proportional);
    let fee = raw_fee.min(MAX_FEE_SOMPIS);
    info!(
        "{} Fee calculated: {} sompis ({:.6} KAS) for transfer of {} sompis",
        LOG_PREFIX,
        fee,
        fee as f64 / SOMPIS_PER_KAS as f64,
        transfer_amount
    );
    fee
}

/// Calculate fee with full distribution breakdown
pub fn calculate_fee_breakdown(
    transfer_amount: u64,
    distribution: &FeeDistribution,
) -> FeeBreakdown {
    let proportional = (transfer_amount as u128 * PROPORTIONAL_BPS as u128 / 10_000) as u64;
    let raw_fee = FLAT_FEE_SOMPIS.saturating_add(proportional);
    let was_capped = raw_fee > MAX_FEE_SOMPIS;
    let total = raw_fee.min(MAX_FEE_SOMPIS);

    let burn_sompis = total * distribution.burn_pct as u64 / 100;
    let staker_sompis = total * distribution.staker_pct as u64 / 100;
    // Treasury gets the remainder to handle rounding
    let treasury_sompis = total - burn_sompis - staker_sompis;

    info!(
        "{} Fee breakdown: total={}, burn={}, stakers={}, treasury={}, capped={}",
        LOG_PREFIX, total, burn_sompis, staker_sompis, treasury_sompis, was_capped
    );

    FeeBreakdown {
        total_sompis: total,
        total_kas: total as f64 / SOMPIS_PER_KAS as f64,
        burn_sompis,
        staker_sompis,
        treasury_sompis,
        was_capped,
    }
}

/// Verify a fee meets the ≤0.001 KAS target
pub fn verify_fee_target(fee_sompis: u64) -> bool {
    fee_sompis <= MAX_FEE_SOMPIS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_small_transfer() {
        let fee = calculate_fee(1_000_000); // 0.01 KAS
        assert!(fee <= MAX_FEE_SOMPIS);
        assert!(fee >= FLAT_FEE_SOMPIS); // At least the flat fee
    }

    #[test]
    fn test_calculate_fee_large_transfer_capped() {
        // Very large transfer: fee should be capped
        let fee = calculate_fee(100_000_000_000_000); // 1M KAS
        assert_eq!(fee, MAX_FEE_SOMPIS);
    }

    #[test]
    fn test_fee_target_met() {
        // Test various transfer amounts
        for amount in [1000, 100_000, 10_000_000, 1_000_000_000, 100_000_000_000] {
            let fee = calculate_fee(amount);
            assert!(
                verify_fee_target(fee),
                "Fee {} exceeds target for amount {}",
                fee,
                amount
            );
        }
    }

    #[test]
    fn test_fee_breakdown_sums() {
        let dist = default_distribution();
        let breakdown = calculate_fee_breakdown(10_000_000, &dist);
        assert_eq!(
            breakdown.burn_sompis + breakdown.staker_sompis + breakdown.treasury_sompis,
            breakdown.total_sompis
        );
    }

    #[test]
    fn test_fee_breakdown_percentages() {
        let dist = default_distribution();
        let breakdown = calculate_fee_breakdown(1_000_000, &dist);
        // Burn should be ~30%
        let burn_pct = (breakdown.burn_sompis as f64 / breakdown.total_sompis as f64) * 100.0;
        assert!((burn_pct - 30.0).abs() < 1.0);
    }

    #[test]
    fn test_default_distribution_sums_to_100() {
        let dist = default_distribution();
        assert_eq!(dist.burn_pct + dist.staker_pct + dist.treasury_pct, 100);
    }

    #[test]
    fn test_zero_transfer_fee() {
        let fee = calculate_fee(0);
        assert_eq!(fee, FLAT_FEE_SOMPIS); // Just the flat fee
    }
}
