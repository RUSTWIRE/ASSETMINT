// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! AssetMint Compliance API Server
//!
//! Starts the Axum REST API with live Kaspa Testnet-12 connection.
//!
//! Usage:
//!   cargo run -p assetmint-core
//!
//! Environment:
//!   KASPA_RPC_URL  — Kaspa wRPC endpoint (default: ws://127.0.0.1:17210)
//!   PORT           — API port (default: 3001)

use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    // Start state-verity sync polling in background
    let compliance_url = format!("http://localhost:{}", port);
    tokio::spawn(async move {
        // Wait for the API server to start before polling it
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        info!("[K-RWA] Starting state-verity sync polling against {}", compliance_url);
        let mut svc = sync::state_sync::StateSyncService::new("http://localhost:8900", 30);
        let initial_state = sync::state_sync::AssetState {
            dkg_ual: "did:dkg:otp/0x1234/init".into(),
            assertion_id: "genesis".into(),
            dkg_ual_hash: [0u8; 32],
            oracle_attestation_hash: [0u8; 32],
            compliance_merkle_root: [0u8; 32],
            state_utxo_txid: String::new(),
            state_utxo_index: 0,
            version: 0,
        };
        svc.set_initial_state(initial_state);
        if let Err(e) = svc.run_polling(&compliance_url).await {
            info!("[K-RWA] State sync stopped: {}", e);
        }
    });

    assetmint_core::api::start_server(port).await
}
