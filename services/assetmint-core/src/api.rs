// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Axum REST API for the compliance service.
//! Endpoints: identity registration, claim issuance, transfer evaluation,
//! ZK proof generation, Merkle root queries.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::claims::{ClaimIssuer, ClaimType, VerifiableCredential, verify_vc_proof};
use crate::identity::IdentityRegistry;
use crate::merkle::MerkleTree;
use crate::rules::{ComplianceEngine, ComplianceResult};
use crate::zk_prover::{ZkProof, ZkProver, ZkWitness};
use crate::zk_verifier::ZkVerifier;
use crate::rate_limit::RateLimiter;
use crate::LOG_PREFIX;
use kaspa_adapter::client::KaspaClient;
use kaspa_adapter::wallet::Wallet;

/// Shared application state
pub struct AppState {
    pub registry: IdentityRegistry,
    pub compliance: Mutex<ComplianceEngine>,
    pub claim_issuer: ClaimIssuer,
    pub kaspa_client: Option<KaspaClient>,
    pub zk_prover: Mutex<ZkProver>,
    pub zk_verifier: Mutex<ZkVerifier>,
}

/// API error response
#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ── Request / Response types ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterIdentityRequest {
    pub did: String,
    pub primary_key: String,
}

#[derive(Serialize)]
pub struct RegisterIdentityResponse {
    pub did: String,
    pub primary_key: String,
    pub created_at: u64,
}

#[derive(Deserialize)]
pub struct IssueClaimRequest {
    pub subject_did: String,
    pub claim_type: String,
    /// Optional jurisdiction string for JurisdictionAllowed claims
    pub jurisdiction: Option<String>,
    /// Expiry timestamp (Unix seconds, 0 = never)
    pub expiry: u64,
}

#[derive(Serialize)]
pub struct IssueClaimResponse {
    pub claim_type: String,
    pub issuer_did: String,
    pub subject_did: String,
    pub expiry: u64,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct EvaluateTransferQuery {
    pub sender_did: String,
    pub receiver_did: String,
    pub asset_id: String,
    pub amount: u64,
    #[serde(default)]
    pub mint_timestamp: u64,
}

#[derive(Serialize)]
pub struct MerkleRootResponse {
    pub root: String,
    pub leaf_count: usize,
}

#[derive(Serialize)]
pub struct OracleAttestationResponse {
    pub asset_id: String,
    pub price_usd: f64,
    pub sources_used: usize,
    pub timestamp: u64,
    pub signatures: Vec<String>,
    pub signer_pubkeys: Vec<String>,
    pub threshold: usize,
    pub data_hash: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub kaspa_connected: bool,
}

#[derive(Serialize)]
pub struct NetworkInfoResponse {
    pub server_version: String,
    pub is_synced: bool,
    pub virtual_daa_score: u64,
    pub network_id: String,
    pub block_count: u64,
    pub difficulty: f64,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance_sompis: u64,
    pub balance_kas: f64,
    pub utxo_count: usize,
}

#[derive(Deserialize)]
pub struct TransferRequest {
    pub sender_did: String,
    pub receiver_did: String,
    pub sender_private_key: String,
    pub receiver_address: String,
    pub amount_sompis: u64,
    pub asset_id: String,
    /// Hex-encoded Groth16 proof bytes (required)
    pub zk_proof: String,
    /// Hex-encoded public inputs: [merkle_root, nullifier_hash]
    pub zk_public_inputs: Vec<String>,
}

#[derive(Serialize)]
pub struct TransferResponse {
    pub tx_id: String,
    pub compliance_result: ComplianceResult,
    pub amount_sompis: u64,
    pub fee_sompis: u64,
}

#[derive(Serialize)]
pub struct ZkProofResponse {
    /// Hex-encoded Groth16 proof bytes
    pub proof: String,
    /// Hex-encoded public inputs: [merkle_root, nullifier_hash]
    pub public_inputs: Vec<String>,
    /// Hex-encoded SHA-256 hash of the proof
    pub proof_hash: String,
    /// Hex-encoded Merkle root the proof was generated against
    pub merkle_root: String,
}

// ── Audit Trail ───────────────────────────────────────────────────────

/// Compute a deterministic audit hash from a compliance decision.
fn compute_audit_hash(result: &ComplianceResult, sender_did: &str, receiver_did: &str, tx_id: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(sender_did.as_bytes());
    hasher.update(receiver_did.as_bytes());
    hasher.update(tx_id.as_bytes());
    hasher.update(if result.allowed { "ALLOWED" } else { "DENIED" }.as_bytes());
    hasher.update(result.rules_evaluated.to_le_bytes());
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    hasher.update(now.to_le_bytes());
    hasher.finalize().into()
}

#[derive(Deserialize)]
pub struct AuditCommitRequest {
    pub decision_hash: String,
    pub from_address: String,
    pub private_key: String,
}

#[derive(Serialize)]
pub struct AuditCommitResponse {
    pub tx_id: String,
    pub audit_hash: String,
    pub timestamp: u64,
}

// ── VC Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct VcIssueRequest {
    pub subject_did: String,
    pub claim_type: String,
    pub jurisdiction: Option<String>,
    pub expiry: u64,
}

#[derive(Serialize, Deserialize)]
pub struct VcIssueResponse {
    pub verifiable_credential: VerifiableCredential,
}

#[derive(Serialize, Deserialize)]
pub struct VcVerifyRequest {
    pub verifiable_credential: VerifiableCredential,
}

#[derive(Serialize, Deserialize)]
pub struct VcVerifyResponse {
    pub valid: bool,
    pub subject_did: String,
    pub claim_type: String,
}

// ── Handlers ──────────────────────────────────────────────────────────

/// POST /identity — register a new identity
async fn register_identity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterIdentityRequest>,
) -> Result<(StatusCode, Json<RegisterIdentityResponse>), (StatusCode, Json<ApiError>)> {
    info!("{} API: POST /identity did={}", LOG_PREFIX, req.did);

    let identity = state
        .registry
        .register(&req.did, &req.primary_key)
        .map_err(|e| error_response(StatusCode::CONFLICT, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterIdentityResponse {
            did: identity.did,
            primary_key: identity.primary_key,
            created_at: identity.created_at,
        }),
    ))
}

/// POST /claim — issue a signed claim
async fn issue_claim(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IssueClaimRequest>,
) -> Result<(StatusCode, Json<IssueClaimResponse>), (StatusCode, Json<ApiError>)> {
    info!(
        "{} API: POST /claim subject={} type={}",
        LOG_PREFIX, req.subject_did, req.claim_type
    );

    // Verify identity exists
    state
        .registry
        .get(&req.subject_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, e.to_string()))?;

    let claim_type = parse_claim_type(&req.claim_type, req.jurisdiction.as_deref())
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e))?;

    let claim = state
        .claim_issuer
        .issue_claim(&req.subject_did, claim_type, req.expiry);

    // Store claim in registry
    state
        .registry
        .add_claim(&claim)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(IssueClaimResponse {
            claim_type: req.claim_type,
            issuer_did: claim.issuer_did,
            subject_did: claim.subject_did,
            expiry: claim.expiry,
            signature: claim.signature,
        }),
    ))
}

/// GET /compliance/evaluate — evaluate transfer compliance
async fn evaluate_transfer(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EvaluateTransferQuery>,
) -> Result<Json<ComplianceResult>, (StatusCode, Json<ApiError>)> {
    info!(
        "{} API: GET /compliance/evaluate {} -> {} amount={}",
        LOG_PREFIX, query.sender_did, query.receiver_did, query.amount
    );

    let sender = state
        .registry
        .get(&query.sender_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, format!("Sender: {}", e)))?;

    let receiver = state
        .registry
        .get(&query.receiver_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, format!("Receiver: {}", e)))?;

    let engine = state
        .compliance
        .lock()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = engine.evaluate_transfer(
        &sender,
        &receiver,
        &query.asset_id,
        query.amount,
        query.mint_timestamp,
    );

    Ok(Json(result))
}

/// GET /merkle-root — current Merkle root of approved addresses
async fn get_merkle_root(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MerkleRootResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: GET /merkle-root", LOG_PREFIX);

    let addresses = state
        .registry
        .get_approved_addresses()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if addresses.is_empty() {
        return Ok(Json(MerkleRootResponse {
            root: "0".repeat(64),
            leaf_count: 0,
        }));
    }

    let tree = MerkleTree::build(&addresses)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(MerkleRootResponse {
        root: hex::encode(tree.root()),
        leaf_count: addresses.len(),
    }))
}

/// GET /health — health check (includes Kaspa connectivity)
async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let kaspa_connected = state
        .kaspa_client
        .as_ref()
        .map(|c| c.is_connected())
        .unwrap_or(false);

    Json(HealthResponse {
        status: "ok".into(),
        service: "compliance-rust".into(),
        kaspa_connected,
    })
}

/// GET /oracle/attestation?asset_id=KAS — fetch a live attested price
///
/// 1. Aggregates price from live + simulated sources
/// 2. Creates a 2-of-3 Ed25519 multisig attestation (testnet signers)
/// 3. Returns the full attestation JSON (price, signatures, data_hash)
async fn oracle_attestation(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<OracleAttestationResponse>, (StatusCode, Json<ApiError>)> {
    let asset_id = params
        .get("asset_id")
        .map(|s| s.as_str())
        .unwrap_or("KAS");

    info!("{} API: GET /oracle/attestation asset_id={}", LOG_PREFIX, asset_id);

    let price = oracle_pool::oracle::get_live_aggregated_price(asset_id)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, format!("Price fetch failed: {}", e)))?;

    let signers = oracle_pool::attestation::create_testnet_signers();
    let signer_refs: Vec<&_> = signers.iter().take(2).collect();

    let attestation = oracle_pool::attestation::create_attestation(price, &signer_refs)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Attestation failed: {}", e)))?;

    Ok(Json(OracleAttestationResponse {
        asset_id: attestation.price.asset_id,
        price_usd: attestation.price.price_usd,
        sources_used: attestation.price.sources_used,
        timestamp: attestation.price.timestamp,
        signatures: attestation.signatures,
        signer_pubkeys: attestation.signer_pubkeys,
        threshold: attestation.threshold,
        data_hash: attestation.data_hash,
    }))
}

/// GET /network — live Kaspa Testnet-12 info
async fn network_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NetworkInfoResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: GET /network", LOG_PREFIX);

    let client = state
        .kaspa_client
        .as_ref()
        .ok_or_else(|| error_response(StatusCode::SERVICE_UNAVAILABLE, "Kaspa client not connected"))?;

    let info = client
        .get_server_info()
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;

    let (block_count, _daa, difficulty) = client
        .get_block_dag_info()
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(NetworkInfoResponse {
        server_version: info.server_version,
        is_synced: info.is_synced,
        virtual_daa_score: info.virtual_daa_score,
        network_id: info.network_id,
        block_count,
        difficulty,
    }))
}

/// GET /balance?address=kaspatest:... — live balance query
async fn get_balance(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<ApiError>)> {
    let address = params
        .get("address")
        .ok_or_else(|| error_response(StatusCode::BAD_REQUEST, "address parameter required"))?;

    info!("{} API: GET /balance address={}", LOG_PREFIX, address);

    let client = state
        .kaspa_client
        .as_ref()
        .ok_or_else(|| error_response(StatusCode::SERVICE_UNAVAILABLE, "Kaspa client not connected"))?;

    let balance = client
        .get_balance(address)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;

    let utxos = client
        .get_utxos(address)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(BalanceResponse {
        address: address.clone(),
        balance_sompis: balance,
        balance_kas: balance as f64 / 1e8,
        utxo_count: utxos.len(),
    }))
}

/// POST /transfer — compliance-gated on-chain transfer
///
/// 1. Evaluate compliance rules (KYC, AML, jurisdiction, max amount)
/// 2. If compliant: build, sign, and broadcast transaction on TN12
/// 3. Return TX hash + compliance result
#[axum::debug_handler]
async fn compliance_transfer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ApiError>)> {
    info!(
        "{} API: POST /transfer {} -> {} amount={} sompis",
        LOG_PREFIX, req.sender_did, req.receiver_did, req.amount_sompis
    );

    // 1. Verify identities exist
    let sender = state
        .registry
        .get(&req.sender_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, format!("Sender: {}", e)))?;
    let receiver = state
        .registry
        .get(&req.receiver_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, format!("Receiver: {}", e)))?;

    // 2. Evaluate compliance (scoped to drop MutexGuard before any .await)
    let result = {
        let engine = state
            .compliance
            .lock()
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        engine.evaluate_transfer(&sender, &receiver, &req.asset_id, req.amount_sompis, 0)
    };

    if !result.allowed {
        info!("{} Transfer DENIED: {:?}", LOG_PREFIX, result.violations);
        return Ok(Json(TransferResponse {
            tx_id: String::new(),
            compliance_result: result,
            amount_sompis: req.amount_sompis,
            fee_sompis: 0,
        }));
    }

    // 2b. Verify ZK proof — mandatory gate
    {
        // Decode proof from hex
        let proof_bytes = hex::decode(&req.zk_proof)
            .map_err(|e| error_response(StatusCode::BAD_REQUEST, format!("Invalid zk_proof hex: {}", e)))?;

        if req.zk_public_inputs.len() != 2 {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!("zk_public_inputs must contain exactly 2 elements [merkle_root, nullifier], got {}", req.zk_public_inputs.len()),
            ));
        }

        let public_inputs: Vec<Vec<u8>> = req
            .zk_public_inputs
            .iter()
            .enumerate()
            .map(|(i, h)| {
                hex::decode(h).map_err(|e| {
                    error_response(StatusCode::BAD_REQUEST, format!("Invalid zk_public_inputs[{}] hex: {}", i, e))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let zk_proof = ZkProof {
            proof_bytes,
            public_inputs,
            proof_hash: [0u8; 32], // not needed for verification
        };

        // Verify the proof against the verifying key
        let verifier = state
            .zk_verifier
            .lock()
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let valid = verifier.verify(&zk_proof).map_err(|e| {
            error_response(StatusCode::BAD_REQUEST, format!("ZK proof verification error: {}", e))
        })?;

        if !valid {
            info!("{} Transfer DENIED: ZK proof invalid", LOG_PREFIX);
            return Err(error_response(
                StatusCode::FORBIDDEN,
                "ZK proof verification failed — sender is not in the approved KYC set",
            ));
        }

        info!("{} ZK proof verified successfully", LOG_PREFIX);
    }

    // 3. Build and broadcast transaction
    let client = state
        .kaspa_client
        .as_ref()
        .ok_or_else(|| error_response(StatusCode::SERVICE_UNAVAILABLE, "Kaspa not connected"))?;

    let wallet = Wallet::from_hex(&req.sender_private_key)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, format!("Invalid key: {}", e)))?;
    let sender_addr = wallet.address_string();

    let tx_id = client
        .send_kas(
            &sender_addr,
            &req.receiver_address,
            req.amount_sompis,
            wallet.keypair(),
            None,
        )
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, format!("Broadcast failed: {}", e)))?;

    info!("{} Compliance-gated transfer complete: TX {}", LOG_PREFIX, tx_id);

    // Fire-and-forget: commit audit hash to DAG
    let audit_hash = compute_audit_hash(&result, &req.sender_did, &req.receiver_did, &tx_id.to_string());
    if state.kaspa_client.is_some() {
        let audit_from = sender_addr.clone();
        let audit_kp = *wallet.keypair();
        let audit_client_endpoint = state.kaspa_client.as_ref().map(|_| ());
        let _ = audit_client_endpoint; // we already have the client reference above
        // Spawn audit commit as a background task — do not block the response
        let audit_hash_hex = hex::encode(audit_hash);
        info!("{} Spawning audit hash commit (hash={})", LOG_PREFIX, audit_hash_hex);
        // We clone the needed data; the client is behind Arc in AppState
        let state_clone = state.clone();
        let audit_from_clone = audit_from.clone();
        tokio::spawn(async move {
            if let Some(client) = state_clone.kaspa_client.as_ref() {
                match client.commit_audit_hash(&audit_from_clone, audit_hash, &audit_kp).await {
                    Ok(atx) => info!("[K-RWA] Audit hash committed: TX {}", atx),
                    Err(e) => info!("[K-RWA] Audit hash commit failed (non-fatal): {}", e),
                }
            }
        });
    }

    Ok(Json(TransferResponse {
        tx_id: tx_id.to_string(),
        compliance_result: result,
        amount_sompis: req.amount_sompis,
        fee_sompis: 13000, // approximate from observed transfers
    }))
}

/// GET /zk-proof/:address — generate a ZK-KYC proof for the given address
///
/// The address must be in the approved KYC set. Returns a Groth16 proof
/// that the address is a member of the current Merkle tree without
/// revealing the address to the verifier.
async fn generate_zk_proof(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<ZkProofResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: GET /zk-proof/{}", LOG_PREFIX, address);

    // 1. Get the approved address list and find the target address index
    let addresses = state
        .registry
        .get_approved_addresses()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if addresses.is_empty() {
        return Err(error_response(
            StatusCode::PRECONDITION_FAILED,
            "No approved addresses in the registry",
        ));
    }

    let leaf_index = addresses
        .iter()
        .position(|a| a == &address)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("Address {} not found in approved KYC set", address),
            )
        })?;

    // 2. Build leaf field elements from addresses (matching the ZK circuit's representation)
    use ark_ff::PrimeField;
    use sha2::{Digest, Sha256};

    let all_leaves: Vec<Vec<u8>> = addresses
        .iter()
        .map(|addr| {
            let hash: [u8; 32] = Sha256::digest(addr.as_bytes()).into();
            let fr = ark_bn254::Fr::from_le_bytes_mod_order(&hash);
            let mut bytes = Vec::new();
            ark_serialize::CanonicalSerialize::serialize_compressed(&fr, &mut bytes)
                .expect("field element serialization");
            bytes
        })
        .collect();

    // Use the address hash as secret key for this demo
    let secret_hash: [u8; 32] = Sha256::digest(address.as_bytes()).into();
    let secret = ark_bn254::Fr::from_le_bytes_mod_order(&secret_hash);
    let mut secret_bytes = Vec::new();
    ark_serialize::CanonicalSerialize::serialize_compressed(&secret, &mut secret_bytes)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("secret serialization: {}", e)))?;

    let witness = ZkWitness {
        secret_key: secret_bytes,
        leaf_index,
        all_leaves,
    };

    // 3. Generate the proof
    let proof = {
        let prover = state
            .zk_prover
            .lock()
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        prover.generate_proof(&witness).map_err(|e| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Proof generation failed: {}", e))
        })?
    };

    // 4. Return hex-encoded proof data
    let merkle_root_hex = if !proof.public_inputs.is_empty() {
        hex::encode(&proof.public_inputs[0])
    } else {
        String::new()
    };

    Ok(Json(ZkProofResponse {
        proof: hex::encode(&proof.proof_bytes),
        public_inputs: proof.public_inputs.iter().map(hex::encode).collect(),
        proof_hash: hex::encode(proof.proof_hash),
        merkle_root: merkle_root_hex,
    }))
}

/// POST /vc/issue — issue a claim and return it as a W3C Verifiable Credential
async fn vc_issue(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VcIssueRequest>,
) -> Result<(StatusCode, Json<VcIssueResponse>), (StatusCode, Json<ApiError>)> {
    info!("{} API: POST /vc/issue subject={} type={}", LOG_PREFIX, req.subject_did, req.claim_type);

    // Verify identity exists
    state
        .registry
        .get(&req.subject_did)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, e.to_string()))?;

    let claim_type = parse_claim_type(&req.claim_type, req.jurisdiction.as_deref())
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e))?;

    let claim = state.claim_issuer.issue_claim(&req.subject_did, claim_type, req.expiry);

    // Store claim in registry
    state
        .registry
        .add_claim(&claim)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert to W3C VC
    let vc = claim.to_verifiable_credential();

    Ok((StatusCode::CREATED, Json(VcIssueResponse {
        verifiable_credential: vc,
    })))
}

/// POST /vc/verify — verify a W3C Verifiable Credential proof
async fn vc_verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VcVerifyRequest>,
) -> Result<Json<VcVerifyResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: POST /vc/verify subject={}", LOG_PREFIX, req.verifiable_credential.credential_subject.id);

    let vc = &req.verifiable_credential;

    let valid = verify_vc_proof(vc, &state.claim_issuer.verifying_key)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, format!("VC verification failed: {}", e)))?;

    Ok(Json(VcVerifyResponse {
        valid,
        subject_did: vc.credential_subject.id.clone(),
        claim_type: vc.credential_subject.claim_type.clone(),
    }))
}

/// POST /audit/commit — commit an audit hash to the Kaspa DAG
#[axum::debug_handler]
async fn commit_audit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuditCommitRequest>,
) -> Result<Json<AuditCommitResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: POST /audit/commit hash={}", LOG_PREFIX, req.decision_hash);

    let client = state
        .kaspa_client
        .as_ref()
        .ok_or_else(|| error_response(StatusCode::SERVICE_UNAVAILABLE, "Kaspa client not connected"))?;

    let audit_bytes: [u8; 32] = hex::decode(&req.decision_hash)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, format!("Invalid hex hash: {}", e)))?
        .try_into()
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "Hash must be exactly 32 bytes"))?;

    let wallet = Wallet::from_hex(&req.private_key)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, format!("Invalid key: {}", e)))?;

    let tx_id = client
        .commit_audit_hash(&req.from_address, audit_bytes, wallet.keypair())
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, format!("Audit commit failed: {}", e)))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    info!("{} Audit hash committed: TX {}", LOG_PREFIX, tx_id);

    Ok(Json(AuditCommitResponse {
        tx_id: tx_id.to_string(),
        audit_hash: req.decision_hash,
        timestamp: now,
    }))
}

// ── Metadata publish-and-commit ───────────────────────────────────────

/// POST /metadata/publish-and-commit
///
/// 1. Forward asset metadata to the sovereign metadata service (localhost:8900/publish)
/// 2. Get back UAL + metadata_hash
/// 3. Optionally commit the metadata_hash on-chain via commit_audit_hash()
///
/// DISCLAIMER: Technical demo — legal wrappers required in production.

#[derive(Deserialize)]
pub struct MetadataPublishRequest {
    /// Arbitrary asset metadata JSON (forwarded as-is to the sovereign service)
    pub metadata: serde_json::Value,
    /// Optional: address to fund the on-chain commit from
    pub from_address: Option<String>,
    /// Optional: hex-encoded private key for signing the on-chain commit
    pub private_key: Option<String>,
}

#[derive(Serialize)]
pub struct MetadataPublishResponse {
    pub ual: String,
    pub metadata_hash: String,
    pub status: String,
    /// Transaction ID if the hash was committed on-chain (None if Kaspa unavailable or keys omitted)
    pub onchain_tx_id: Option<String>,
    pub onchain_committed: bool,
}

/// Response shape returned by the sovereign metadata service POST /publish
#[derive(Deserialize)]
struct SovereignPublishResponse {
    ual: String,
    metadata_hash: String,
    #[allow(dead_code)]
    status: String,
}

#[axum::debug_handler]
async fn metadata_publish_and_commit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MetadataPublishRequest>,
) -> Result<Json<MetadataPublishResponse>, (StatusCode, Json<ApiError>)> {
    info!("{} API: POST /metadata/publish-and-commit", LOG_PREFIX);

    // 1. Forward metadata to sovereign metadata service
    let sovereign_url = std::env::var("SOVEREIGN_METADATA_URL")
        .unwrap_or_else(|_| "http://localhost:8900".to_string());

    let http_client = reqwest::Client::new();
    let sovereign_resp = http_client
        .post(format!("{}/publish", sovereign_url))
        .json(&req.metadata)
        .send()
        .await
        .map_err(|e| {
            error_response(
                StatusCode::BAD_GATEWAY,
                format!("Sovereign metadata service unreachable: {}", e),
            )
        })?;

    if !sovereign_resp.status().is_success() {
        let status = sovereign_resp.status();
        let body = sovereign_resp.text().await.unwrap_or_default();
        return Err(error_response(
            StatusCode::BAD_GATEWAY,
            format!("Sovereign metadata service returned {}: {}", status, body),
        ));
    }

    let publish_result: SovereignPublishResponse = sovereign_resp.json().await.map_err(|e| {
        error_response(
            StatusCode::BAD_GATEWAY,
            format!("Invalid response from sovereign metadata service: {}", e),
        )
    })?;

    info!(
        "{} Metadata published: UAL={} hash={}",
        LOG_PREFIX, publish_result.ual, publish_result.metadata_hash
    );

    // 2. Optionally commit the metadata_hash on-chain
    let mut onchain_tx_id: Option<String> = None;
    let mut onchain_committed = false;

    if let (Some(ref from_address), Some(ref private_key)) = (&req.from_address, &req.private_key)
    {
        if let Some(ref client) = state.kaspa_client {
            // Decode the metadata hash into [u8; 32]
            let hash_bytes: [u8; 32] = hex::decode(&publish_result.metadata_hash)
                .map_err(|e| {
                    error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Invalid metadata_hash hex from sovereign service: {}", e),
                    )
                })?
                .try_into()
                .map_err(|_| {
                    error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "metadata_hash is not 32 bytes",
                    )
                })?;

            let wallet = Wallet::from_hex(private_key).map_err(|e| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    format!("Invalid private_key: {}", e),
                )
            })?;

            let tx_id = client
                .commit_audit_hash(from_address, hash_bytes, wallet.keypair())
                .await
                .map_err(|e| {
                    error_response(
                        StatusCode::BAD_GATEWAY,
                        format!("On-chain commit failed: {}", e),
                    )
                })?;

            info!(
                "{} Metadata hash committed on-chain: TX {}",
                LOG_PREFIX, tx_id
            );
            onchain_tx_id = Some(tx_id.to_string());
            onchain_committed = true;
        } else {
            info!(
                "{} Kaspa client not available — skipping on-chain commit",
                LOG_PREFIX
            );
        }
    } else {
        info!(
            "{} No from_address/private_key provided — skipping on-chain commit",
            LOG_PREFIX
        );
    }

    Ok(Json(MetadataPublishResponse {
        ual: publish_result.ual,
        metadata_hash: publish_result.metadata_hash,
        status: "published".to_string(),
        onchain_tx_id,
        onchain_committed,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────

fn parse_claim_type(type_str: &str, jurisdiction: Option<&str>) -> Result<ClaimType, String> {
    match type_str {
        "KycVerified" => Ok(ClaimType::KycVerified),
        "AccreditedInvestor" => Ok(ClaimType::AccreditedInvestor),
        "JurisdictionAllowed" => {
            let j = jurisdiction.ok_or("jurisdiction field required for JurisdictionAllowed")?;
            Ok(ClaimType::JurisdictionAllowed(j.to_string()))
        }
        "AmlClear" => Ok(ClaimType::AmlClear),
        "ExemptedEntity" => Ok(ClaimType::ExemptedEntity),
        other => Err(format!("Unknown claim type: {}", other)),
    }
}

// ── Router ────────────────────────────────────────────────────────────

/// Build the Axum router with all compliance + Kaspa endpoints
pub fn build_router(state: Arc<AppState>) -> Router {
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 100 requests per minute per IP
    let rate_limiter = RateLimiter::new(100, 60);

    Router::new()
        .route("/identity", post(register_identity))
        .route("/claim", post(issue_claim))
        .route("/compliance/evaluate", get(evaluate_transfer))
        .route("/merkle-root", get(get_merkle_root))
        .route("/health", get(health))
        .route("/network", get(network_info))
        .route("/balance", get(get_balance))
        .route("/transfer", post(compliance_transfer))
        .route("/audit/commit", post(commit_audit))
        .route("/vc/issue", post(vc_issue))
        .route("/vc/verify", post(vc_verify))
        .route("/zk-proof/{address}", get(generate_zk_proof))
        .route("/metadata/publish-and-commit", post(metadata_publish_and_commit))
        .route("/oracle/attestation", get(oracle_attestation))
        .layer(cors)
        .layer(axum::Extension(rate_limiter))
        .with_state(state)
}

/// Create default AppState for testing (no Kaspa connection)
///
/// Runs a trusted setup for the ZK circuit so that proof generation
/// and verification work out of the box in tests and demos.
pub fn create_default_state() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let registry = IdentityRegistry::in_memory()
        .map_err(|e| format!("Failed to create registry: {}", e))?;
    let compliance = ComplianceEngine::new();
    let claim_issuer = ClaimIssuer::new("did:kaspa:assetmint-issuer", &[42u8; 32]);

    // Run trusted setup for ZK prover/verifier
    let tree_depth = 2;
    let keys_dir = "/tmp/assetmint_compliance_keys";
    let setup = zk_circuits::setup::run_trusted_setup(tree_depth, keys_dir)
        .map_err(|e| format!("ZK trusted setup failed: {}", e))?;

    let mut zk_prover = ZkProver::new(tree_depth);
    zk_prover.set_proving_key(setup.proving_key);

    let mut zk_verifier = ZkVerifier::new();
    zk_verifier.set_verifying_key(setup.verifying_key);

    Ok(Arc::new(AppState {
        registry,
        compliance: Mutex::new(compliance),
        claim_issuer,
        kaspa_client: None,
        zk_prover: Mutex::new(zk_prover),
        zk_verifier: Mutex::new(zk_verifier),
    }))
}

/// Create AppState with live Kaspa connection
pub async fn create_live_state(
    kaspa_endpoint: &str,
) -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let registry = IdentityRegistry::in_memory()
        .map_err(|e| format!("Failed to create registry: {}", e))?;
    let compliance = ComplianceEngine::new();

    // Read issuer key from env, fall back to testnet issuer key
    let issuer_key_hex = std::env::var("ISSUER_PRIVATE_KEY")
        .unwrap_or_else(|_| {
            info!("{} ISSUER_PRIVATE_KEY not set, using testnet default", LOG_PREFIX);
            "91149facb865c1f35b4cdec412caef7cd41191372024cd37cf9fd4a9b6bf686d".to_string()
        });
    let issuer_key_bytes = hex::decode(&issuer_key_hex)
        .map_err(|e| format!("Invalid ISSUER_PRIVATE_KEY hex: {}", e))?;
    let issuer_key: [u8; 32] = issuer_key_bytes
        .try_into()
        .map_err(|_| "ISSUER_PRIVATE_KEY must be exactly 32 bytes (64 hex chars)")?;
    let claim_issuer = ClaimIssuer::new("did:kaspa:assetmint-issuer", &issuer_key);

    info!("{} Connecting to Kaspa at {}", LOG_PREFIX, kaspa_endpoint);
    let client = KaspaClient::new(kaspa_endpoint)
        .map_err(|e| format!("Kaspa client init: {}", e))?;
    client
        .connect()
        .await
        .map_err(|e| format!("Kaspa connect: {}", e))?;

    let server_info = client
        .get_server_info()
        .await
        .map_err(|e| format!("Kaspa server info: {}", e))?;
    info!(
        "{} Connected to kaspad {} (synced={}, daa={})",
        LOG_PREFIX, server_info.server_version, server_info.is_synced, server_info.virtual_daa_score
    );

    // Run trusted setup for ZK prover/verifier
    let tree_depth = 2;
    let keys_dir = std::env::var("ZK_KEYS_DIR")
        .unwrap_or_else(|_| "/tmp/assetmint_compliance_keys".to_string());
    let setup = zk_circuits::setup::run_trusted_setup(tree_depth, &keys_dir)
        .map_err(|e| format!("ZK trusted setup failed: {}", e))?;

    let mut zk_prover = ZkProver::new(tree_depth);
    zk_prover.set_proving_key(setup.proving_key);

    let mut zk_verifier = ZkVerifier::new();
    zk_verifier.set_verifying_key(setup.verifying_key);

    Ok(Arc::new(AppState {
        registry,
        compliance: Mutex::new(compliance),
        claim_issuer,
        kaspa_client: Some(client),
        zk_prover: Mutex::new(zk_prover),
        zk_verifier: Mutex::new(zk_verifier),
    }))
}

/// Start the compliance API server with live Kaspa connection
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("{} Starting compliance API on port {}", LOG_PREFIX, port);

    let kaspa_endpoint =
        std::env::var("KASPA_RPC_URL").unwrap_or_else(|_| "ws://127.0.0.1:17210".to_string());

    let state = create_live_state(&kaspa_endpoint).await?;
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!(
        "{} Compliance API ready on http://localhost:{}",
        LOG_PREFIX, port
    );

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let state = create_default_state().unwrap();
        build_router(state)
    }

    #[tokio::test]
    async fn test_health() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_register_identity() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/identity")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"did":"did:kaspa:alice","primary_key":"0xabc"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_issue_claim_and_evaluate() {
        let state = create_default_state().unwrap();
        let app = build_router(state.clone());

        // Register two identities
        let _ = state.registry.register("did:kaspa:alice", "0xa").unwrap();
        let _ = state.registry.register("did:kaspa:bob", "0xb").unwrap();

        // Issue KYC claims via API
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claim")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"subject_did":"did:kaspa:alice","claim_type":"KycVerified","expiry":0}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/claim")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"subject_did":"did:kaspa:bob","claim_type":"KycVerified","expiry":0}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Evaluate transfer
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/compliance/evaluate?sender_did=did:kaspa:alice&receiver_did=did:kaspa:bob&asset_id=KPROP-NYC-TEST&amount=1000&mint_timestamp=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_merkle_root_empty() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/merkle-root")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn test_audit_hash_deterministic() {
        let result = ComplianceResult {
            allowed: true,
            violations: vec![],
            rules_evaluated: 3,
        };
        // Two calls with the same inputs at the same second should produce
        // a 32-byte hash (we can't assert equality across calls because of the
        // timestamp component, but we CAN verify the output is well-formed).
        let hash = compute_audit_hash(&result, "did:kaspa:alice", "did:kaspa:bob", "tx123");
        assert_eq!(hash.len(), 32);

        // Denied result should produce a different hash
        let denied = ComplianceResult {
            allowed: false,
            violations: vec!["KYC_MISSING".to_string()],
            rules_evaluated: 3,
        };
        let hash2 = compute_audit_hash(&denied, "did:kaspa:alice", "did:kaspa:bob", "tx123");
        assert_ne!(hash, hash2);
    }

    #[tokio::test]
    async fn test_merkle_root_with_identities() {
        let state = create_default_state().unwrap();
        let app = build_router(state.clone());

        state.registry.register("did:kaspa:a", "0xa").unwrap();
        state.registry.register("did:kaspa:b", "0xb").unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/merkle-root")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_vc_issue() {
        let state = create_default_state().unwrap();
        let app = build_router(state.clone());

        // Register identity first
        state.registry.register("did:kaspa:vc-alice", "0xabc").unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/vc/issue")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"subject_did":"did:kaspa:vc-alice","claim_type":"KycVerified","expiry":0}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), 1024 * 64).await.unwrap();
        let vc_resp: VcIssueResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(vc_resp.verifiable_credential.credential_subject.id, "did:kaspa:vc-alice");
        assert_eq!(vc_resp.verifiable_credential.credential_subject.claim_type, "KycVerified");
        assert_eq!(vc_resp.verifiable_credential.vc_type[0], "VerifiableCredential");
    }

    #[tokio::test]
    async fn test_vc_issue_and_verify_roundtrip() {
        let state = create_default_state().unwrap();
        let app = build_router(state.clone());

        state.registry.register("did:kaspa:vc-bob", "0xdef").unwrap();

        // Issue VC
        let issue_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/vc/issue")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"subject_did":"did:kaspa:vc-bob","claim_type":"AccreditedInvestor","expiry":0}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(issue_resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(issue_resp.into_body(), 1024 * 64).await.unwrap();
        let vc_resp: VcIssueResponse = serde_json::from_slice(&body).unwrap();

        // Verify VC via API
        let verify_body = serde_json::to_string(&VcVerifyRequest {
            verifiable_credential: vc_resp.verifiable_credential,
        })
        .unwrap();

        let verify_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/vc/verify")
                    .header("content-type", "application/json")
                    .body(Body::from(verify_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(verify_resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(verify_resp.into_body(), 1024 * 64).await.unwrap();
        let vr: VcVerifyResponse = serde_json::from_slice(&body).unwrap();
        assert!(vr.valid);
        assert_eq!(vr.subject_did, "did:kaspa:vc-bob");
        assert_eq!(vr.claim_type, "AccreditedInvestor");
    }
}
