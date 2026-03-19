// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! SilverScript compiled output → P2SH deployment on Kaspa Testnet-12.
//! Loads compiled .sil JSON output (from silverc), derives P2SH addresses,
//! and builds funding transactions to deploy covenant contracts on-chain.

use kaspa_addresses::{Address, Prefix, Version};
use kaspa_consensus_core::tx::ScriptPublicKey;
use kaspa_txscript::pay_to_script_hash_script;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("[K-RWA] Failed to load script: {0}")]
    LoadFailed(String),
    #[error("[K-RWA] Invalid script format: {0}")]
    InvalidFormat(String),
    #[error("[K-RWA] JSON parse error: {0}")]
    ParseError(String),
}

/// Entrypoint function metadata from compiled contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<AbiInput>,
}

/// ABI function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiInput {
    pub name: String,
    pub type_name: String,
}

/// Compiled SilverScript contract (loaded from silverc JSON output)
#[derive(Debug, Clone)]
pub struct CompiledContract {
    /// Contract name from the .sil source
    pub contract_name: String,
    /// Raw redeem script bytes (compiled bytecode)
    pub redeem_script: Vec<u8>,
    /// P2SH locking script (for creating outputs)
    pub script_public_key: ScriptPublicKey,
    /// P2SH address (kaspatest:pr...)
    pub p2sh_address: Address,
    /// ABI — entrypoint function signatures
    pub abi: Vec<AbiFunction>,
    /// Whether selector byte is needed (false = multiple entrypoints)
    pub without_selector: bool,
}

/// Raw JSON output from silverc compiler
#[derive(Deserialize)]
struct SilvercOutput {
    contract_name: String,
    script: Vec<u8>,
    abi: Vec<AbiFunction>,
    without_selector: bool,
}

/// Load a compiled SilverScript contract from its JSON file (silverc output)
pub fn load_contract_json(path: &str) -> Result<CompiledContract, ScriptError> {
    info!("{} Loading compiled contract from {}", LOG_PREFIX, path);

    let json_str = std::fs::read_to_string(path)
        .map_err(|e| ScriptError::LoadFailed(format!("{}: {}", path, e)))?;

    let output: SilvercOutput =
        serde_json::from_str(&json_str).map_err(|e| ScriptError::ParseError(format!("{}", e)))?;

    load_from_bytes(
        &output.contract_name,
        output.script,
        output.abi,
        output.without_selector,
    )
}

/// Load a contract from raw bytecode (for programmatic use)
pub fn load_from_bytes(
    name: &str,
    script_bytes: Vec<u8>,
    abi: Vec<AbiFunction>,
    without_selector: bool,
) -> Result<CompiledContract, ScriptError> {
    if script_bytes.is_empty() {
        return Err(ScriptError::InvalidFormat("Empty script".into()));
    }

    // Generate P2SH ScriptPublicKey: blake2b-256 hash of redeem script
    let script_public_key = pay_to_script_hash_script(&script_bytes);

    // Derive P2SH address from the script hash
    // The script_public_key payload contains: OP_BLAKE2B OP_DATA_32 <hash> OP_EQUAL
    // We need the 32-byte hash to build the address
    let spk_bytes = script_public_key.script();
    // P2SH script format: [0xaa, 0x20, <32 bytes hash>, 0x87]
    // Extract the 32-byte hash (bytes 2..34)
    let script_hash = &spk_bytes[2..34];
    let p2sh_address = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash);

    info!(
        "{} Contract '{}': {} bytes, {} entrypoints, P2SH={}",
        LOG_PREFIX,
        name,
        script_bytes.len(),
        abi.len(),
        p2sh_address
    );

    Ok(CompiledContract {
        contract_name: name.to_string(),
        redeem_script: script_bytes,
        script_public_key,
        p2sh_address,
        abi,
        without_selector,
    })
}
