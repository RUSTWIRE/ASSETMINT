// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! ASTM KRC-20 inscription token.
//! Deploy, mint, and transfer operations via Kaspa inscriptions.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::LOG_PREFIX;

/// KRC-20 inscription operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Krc20Inscription {
    /// Protocol identifier
    pub p: String,
    /// Operation type
    pub op: String,
    /// Token ticker
    pub tick: String,
    /// Additional fields depending on operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lim: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

/// Create the ASTM token deploy inscription
pub fn deploy_inscription() -> Krc20Inscription {
    info!("{} Creating ASTM KRC-20 deploy inscription", LOG_PREFIX);
    Krc20Inscription {
        p: "krc-20".to_string(),
        op: "deploy".to_string(),
        tick: "ASTM".to_string(),
        max: Some("1000000000".to_string()),
        lim: Some("1000".to_string()),
        amt: None,
        to: None,
    }
}

/// Create a mint inscription
pub fn mint_inscription(amount: u64) -> Krc20Inscription {
    info!("{} Creating ASTM mint inscription: amount={}", LOG_PREFIX, amount);
    Krc20Inscription {
        p: "krc-20".to_string(),
        op: "mint".to_string(),
        tick: "ASTM".to_string(),
        max: None,
        lim: None,
        amt: Some(amount.to_string()),
        to: None,
    }
}

/// Create a transfer inscription
pub fn transfer_inscription(to: &str, amount: u64) -> Krc20Inscription {
    info!("{} Creating ASTM transfer inscription: to={}, amount={}", LOG_PREFIX, to, amount);
    Krc20Inscription {
        p: "krc-20".to_string(),
        op: "transfer".to_string(),
        tick: "ASTM".to_string(),
        max: None,
        lim: None,
        amt: Some(amount.to_string()),
        to: Some(to.to_string()),
    }
}
