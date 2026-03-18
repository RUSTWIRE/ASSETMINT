// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Axum REST API for the compliance service.
//! Endpoints: identity registration, claim issuance, transfer evaluation,
//! ZK proof generation, Merkle root queries.

use tracing::info;

use crate::LOG_PREFIX;

/// Start the compliance API server
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("{} Starting compliance API on port {}", LOG_PREFIX, port);

    // TODO: Implement Axum routes:
    // POST /identity         — register a new identity
    // POST /claim            — issue a claim
    // GET  /compliance/evaluate — evaluate transfer compliance
    // GET  /zk-proof/:address   — generate Groth16 proof
    // GET  /merkle-root         — current Merkle root hash

    info!("{} Compliance API ready on http://localhost:{}", LOG_PREFIX, port);
    Ok(())
}
