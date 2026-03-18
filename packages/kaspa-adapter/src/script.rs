// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! SilverScript compiled output → P2SH address generation.
//! Loads compiled .sil output and derives Pay-to-Script-Hash addresses.

use sha2::{Sha256, Digest};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("[K-RWA] Failed to load script: {0}")]
    LoadFailed(String),
    #[error("[K-RWA] Invalid script hex: {0}")]
    InvalidHex(String),
}

/// Compiled SilverScript covenant
#[derive(Debug, Clone)]
pub struct CompiledScript {
    /// Raw script bytes (compiled from .sil)
    pub bytecode: Vec<u8>,
    /// SHA-256 hash of the script
    pub script_hash: [u8; 32],
    /// P2SH address (Kaspa testnet format)
    pub p2sh_address: String,
}

/// Load a compiled SilverScript file and derive P2SH address
pub fn load_compiled_script(script_hex: &str) -> Result<CompiledScript, ScriptError> {
    info!("{} Loading compiled SilverScript", LOG_PREFIX);

    let bytecode = hex::decode(script_hex)
        .map_err(|e| ScriptError::InvalidHex(e.to_string()))?;

    // SHA-256 hash for P2SH
    let mut hasher = Sha256::new();
    hasher.update(&bytecode);
    let script_hash: [u8; 32] = hasher.finalize().into();

    // TODO: Derive proper Kaspa testnet P2SH address from script hash
    // Format: kaspatest:pr<base32-encoded-script-hash>
    let p2sh_address = format!("kaspatest:pr{}", hex::encode(&script_hash[..20]));

    info!("{} Script loaded: hash={}, address={}",
        LOG_PREFIX, hex::encode(script_hash), p2sh_address);

    Ok(CompiledScript {
        bytecode,
        script_hash,
        p2sh_address,
    })
}

/// Load a .sil compiled output file from disk
pub fn load_from_file(path: &str) -> Result<CompiledScript, ScriptError> {
    info!("{} Loading script from file: {}", LOG_PREFIX, path);
    let hex_content = std::fs::read_to_string(path)
        .map_err(|e| ScriptError::LoadFailed(format!("{}: {}", path, e)))?;
    load_compiled_script(hex_content.trim())
}
