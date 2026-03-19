// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! # oracle-pool
//!
//! Simulated centralized multisig oracle for AssetMint.
//! Currently: 2-of-3 multisig with off-chain price aggregation.
//! Future: miner-attested oracle per Kaspa core team research (see IOraclePool trait).

pub mod oracle;
pub mod attestation;
pub mod interfaces;

/// Log prefix for all AssetMint oracle operations
pub const LOG_PREFIX: &str = "[K-RWA]";

/// Deployed covenant P2SH addresses on Kaspa TN12 (for oracle attestation targets)
pub mod deployed {
    /// Reserves contract — oracle attests collateral ratios here
    pub const RESERVES_P2SH: &str =
        "kaspatest:prlsah5judppj9np80zzp4qyrf90ccjnvd3u9uvhx8gzf7pjej33vkl0ln4vg";
    /// StateVerity contract — oracle updates price attestation here
    pub const STATE_VERITY_P2SH: &str =
        "kaspatest:pq6xyf8f4tzpeuz4s6yy8063j6g6dwv0a4lcerv4uc98m99shgpcsftdcl5d7";
}
