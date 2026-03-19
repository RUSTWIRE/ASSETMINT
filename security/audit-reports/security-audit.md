# AssetMint Compliance API — STRIDE Threat Model

Target: `services/assetmint-core/src/api.rs` (compliance REST API)
Supporting modules: `identity.rs`, `claims.rs`, `rules.rs`, `zk_prover.rs`, `zk_verifier.rs`

---

## 1. Spoofing

### 1.1 Claim Issuer Impersonation

**Threat:** An attacker could impersonate the claim issuer to issue fraudulent KYC/AML claims.

**Analysis:** The `ClaimIssuer` is initialized with an Ed25519 signing key loaded from the `ISSUER_PRIVATE_KEY` environment variable (`api.rs` line 837). If this key is compromised, an attacker can issue arbitrary claims that pass signature verification. The fallback key at line 840 (`91149fac...`) is a hardcoded testnet key, which would be publicly known from the source code.

**Current mitigation:** Ed25519 signatures on claims (`claims.rs` line 306, `signing_key.sign(&claim_data)`). Verification via `verify_claim()` (line 365) and `verify_vc_proof()` (line 228) both check the signature against the issuer's verifying key.

**Residual risk:** HIGH for production. The env-var key management provides no HSM protection, no key rotation mechanism, and no multi-sig requirement. The testnet fallback key is a critical vulnerability if deployed beyond demo use.

**Recommendation:** Replace env-var key management with an HSM-backed signing service. Implement key rotation with a grace period where both old and new keys are accepted. Add multi-party signing for high-value claim types (AccreditedInvestor, ExemptedEntity).

### 1.2 Identity Registration Spoofing

**Threat:** Anyone can register any DID string via `POST /identity` (`api.rs` line 217).

**Analysis:** The `register_identity` handler accepts arbitrary `did` and `primary_key` values (lines 220-221). There is no proof-of-ownership check: the caller does not need to prove they control the DID or the associated key. The only protection is the UNIQUE constraint on the `did` column in SQLite (`identity.rs` line 56), preventing duplicate registration.

**Residual risk:** MEDIUM. An attacker could register a DID they do not control, then obtain claims for that DID, potentially impersonating another entity in the compliance system.

**Recommendation:** Require a signature from the `primary_key` over the registration request to prove key ownership. Add DID resolution to verify the DID document matches the claimed key.

### 1.3 ZK Proof Identity Bypass

**Threat:** A prover could generate a ZK proof for an address they do not control.

**Analysis:** In `generate_zk_proof` (`api.rs` lines 576-658), the secret key for the proof is derived from `SHA256(address.as_bytes())` (line 622). This is deterministic and publicly computable by anyone who knows the address string. There is no authentication on the `GET /zk-proof/{address}` endpoint.

**Residual risk:** HIGH for production. Anyone can generate a valid ZK-KYC proof for any approved address without possessing its private key. This defeats the purpose of the ZK proof as an identity gate.

**Recommendation:** The secret key must come from the address holder's actual private key, not from a hash of the address. Add authentication (JWT or signature challenge) to the proof generation endpoint.

---

## 2. Tampering

### 2.1 Compliance Result Manipulation

**Threat:** An attacker could modify compliance evaluation results between the check and the transaction broadcast.

**Analysis:** In `compliance_transfer` (`api.rs` lines 428-569), the compliance evaluation (lines 448-454) and the transaction broadcast (lines 527-536) happen in the same handler. The compliance result is computed from the `ComplianceEngine` state under a `Mutex` lock (scoped to drop the guard before the `await` on line 454). Between the compliance check and the broadcast, another request could modify the compliance engine state.

**Current mitigation:** The compliance engine is stateless with respect to individual transfers; it only reads identity claims and rule definitions. The `Mutex` is on the engine, not on the transfer operation. The audit hash (`compute_audit_hash`, line 541) is computed after the compliance check and includes the decision, providing a tamper-evident record.

**Residual risk:** LOW. The TOCTOU window exists but is narrow, and the compliance state (claims, rules) is not modified by the transfer operation itself. The audit hash commitment to the Kaspa DAG (lines 548-561) provides post-facto non-repudiation.

### 2.2 Merkle Tree Manipulation

**Threat:** An attacker could manipulate the Merkle tree to include unauthorized addresses.

**Analysis:** The Merkle tree is built on-the-fly from `get_approved_addresses()` (`api.rs` line 322, `identity.rs` line 212-226). The approved address list comes from all non-revoked identities in SQLite. Any registered identity becomes an approved address. Since registration has no authentication (see Spoofing 1.2), an attacker could register a fraudulent identity and get it included in the Merkle tree.

**Residual risk:** MEDIUM. Chained with the identity registration spoofing, this enables unauthorized inclusion in the approved set.

### 2.3 ZK Proof Verification Bypass

**Threat:** An attacker could submit a forged ZK proof that passes verification.

**Analysis:** The ZK proof verification in `compliance_transfer` (lines 467-515) decodes the proof from hex, constructs a `ZkProof` struct, and calls `verifier.verify(&zk_proof)` (line 502). The verification uses the Groth16 verifying key from the trusted setup. The soundness of verification depends on the integrity of the trusted setup (`zk_circuits::setup::run_trusted_setup`, line 809).

**Current mitigation:** Groth16 soundness provides computational security against forged proofs (under the knowledge-of-exponent assumption). The trusted setup is run at server startup.

**Residual risk:** LOW (cryptographic). The trusted setup's toxic waste is generated in-process and discarded automatically. For production, a multi-party ceremony would be required. The `proof_hash` field is set to `[0u8; 32]` at line 493, meaning the proof hash is not checked during transfer verification, only the full Groth16 verification.

---

## 3. Repudiation

### 3.1 Compliance Decision Non-Repudiation

**Threat:** A participant could deny that a compliance decision was made, or claim it was different from what was actually decided.

**Analysis:** The `compute_audit_hash` function (`api.rs` lines 160-171) creates a SHA-256 hash of `sender_did + receiver_did + tx_id + ALLOWED/DENIED + rules_evaluated + timestamp`. This hash is committed to the Kaspa DAG via `commit_audit_hash` (line 555). The background task at lines 553-560 spawns an async commit.

**Current mitigation:** STRONG. The on-chain audit hash provides non-repudiation backed by Kaspa's PoW consensus. The hash binds the decision to the specific transfer, participants, and timestamp. The hash is deterministic, so any party with the inputs can verify the audit record independently.

**Residual risk:** LOW. The audit commit is fire-and-forget (line 540); if the commit fails (line 558), the transfer still succeeds but the audit trail has a gap. The failure is logged but not surfaced to the caller.

**Recommendation:** Consider making the audit commit mandatory (fail the transfer if the audit commit fails) or maintaining a local audit log as a backup.

### 3.2 Claim Issuance Audit

**Threat:** A claim issuer could deny having issued a claim.

**Analysis:** Claims are stored in SQLite (`identity.rs` lines 193-206) with the Ed25519 signature. The signature is verifiable against the issuer's public key (`claims.rs` line 365). However, there is no on-chain record of claim issuance, only of transfers.

**Residual risk:** MEDIUM. SQLite data is mutable; the issuer could delete or alter claim records. The Ed25519 signature provides cryptographic non-repudiation of claim content, but not of issuance timing or context.

---

## 4. Information Disclosure

### 4.1 PII Protection via ZK Proofs

**Threat:** The compliance system could leak personally identifiable information (identity details, KYC status).

**Analysis:** The ZK-KYC proof system (`zk_prover.rs`, `zk_verifier.rs`) uses Groth16 zero-knowledge proofs to prove membership in the approved Merkle tree without revealing the prover's identity. The public inputs contain only the Merkle root and a nullifier hash (`api.rs` lines 647-651), not the address or identity details.

**Current mitigation:** STRONG for on-chain data. The Groth16 proof reveals nothing beyond the statement (Merkle membership). The compliance API endpoint `/zk-proof/{address}` (`api.rs` line 792) does take the address as a path parameter, which appears in server logs (line 580) and URL history.

**Residual risk:** MEDIUM. The address is visible in the API request URL. Server logs at `info` level record DIDs and addresses (`api.rs` lines 221, 244, 284, 397, 433, 580). The SQLite database stores the full identity registry in plaintext.

**Recommendation:** Reduce log verbosity for PII fields. Hash or encrypt DIDs in log output. Encrypt the SQLite database at rest.

### 4.2 Private Key Exposure

**Threat:** Private keys could be exposed through the API.

**Analysis:** The `POST /transfer` endpoint (`api.rs` line 428) accepts `sender_private_key` in the JSON request body (line 128). This key is used to construct a `Wallet` (line 523) and sign the transaction. The key is transmitted over the network in the request.

**Residual risk:** CRITICAL. Sending private keys in API request bodies is fundamentally insecure. Even over TLS, the key appears in server memory, potentially in logs, and in any request logging middleware.

**Recommendation:** Replace with a signing scheme where the client signs the transaction locally and submits the signed transaction. Never transmit private keys over the network.

### 4.3 Issuer Key in Error Messages

**Threat:** Error messages could leak sensitive information.

**Analysis:** Error responses use the `ApiError` struct (`api.rs` lines 39-42) with user-facing messages. Some error paths include internal details: `format!("Invalid key: {}", e)` (line 524), `format!("Broadcast failed: {}", e)` (line 536). These could leak internal error details to the client.

**Residual risk:** LOW. The error messages are generic enough to not expose implementation details directly, but they pass through underlying library errors which could contain stack traces or memory addresses in debug builds.

---

## 5. Denial of Service

### 5.1 No Rate Limiting

**Threat:** An attacker could overwhelm the API with requests.

**Analysis:** The Axum router (`api.rs` lines 771-794) uses `tower_http::cors::CorsLayer` (line 773) but no rate limiting middleware. Every endpoint is publicly accessible without authentication. The computationally expensive endpoints are:
- `GET /zk-proof/{address}` (line 792) — generates a Groth16 proof (several seconds of CPU)
- `POST /transfer` (line 788) — performs compliance evaluation + ZK verification + transaction broadcast
- `POST /vc/issue` (line 789) — Ed25519 signing

**Residual risk:** HIGH. A single attacker could exhaust server CPU by repeatedly requesting ZK proof generation. The `Mutex<ZkProver>` (line 34) serializes proof generation, creating a single-threaded bottleneck.

**Recommendation:** Add `tower::limit::RateLimitLayer` to the router. Implement per-IP rate limiting. Consider moving ZK proof generation to a queue-based background worker with capacity limits. Add authentication to the proof generation endpoint.

### 5.2 SQLite Lock Contention

**Threat:** Concurrent requests could cause SQLite lock contention.

**Analysis:** The `IdentityRegistry` wraps a `Connection` in `Arc<Mutex<Connection>>` (`identity.rs` line 44). Every identity lookup, registration, and claim storage acquires this lock. Under high load, all identity operations serialize.

**Residual risk:** MEDIUM. SQLite is adequate for demo/testing but will bottleneck under production load. The `Mutex` is held across database operations, not just for the lock acquisition.

**Recommendation:** Replace SQLite with PostgreSQL or a connection-pooled database. Use `tokio::sync::Mutex` instead of `std::sync::Mutex` for async compatibility (currently the code drops the guard before `.await` points, but this is fragile).

### 5.3 Unbounded Request Bodies

**Threat:** An attacker could send extremely large request bodies.

**Analysis:** Axum has default body size limits, but the API does not explicitly set them. The `ZkProofResponse` (lines 146-155) and `TransferRequest` (lines 124-135) could potentially accept large hex-encoded proof data.

**Residual risk:** LOW. Axum's default 2MB body limit provides reasonable protection. Groth16 proofs are small (< 1KB).

---

## 6. Elevation of Privilege

### 6.1 Admin Key Management

**Threat:** An attacker could escalate privileges by obtaining the issuer key.

**Analysis:** The entire compliance system's trust is rooted in the `ISSUER_PRIVATE_KEY` env var (`api.rs` line 837). This single key controls:
- Claim issuance (KYC, AML, AccreditedInvestor, ExemptedEntity)
- VC signing
- The identity of all claims in the system

There is no role separation: the same key issues KYC claims and AccreditedInvestor claims. There is no admin API for key rotation, no multi-sig requirement, and no separation between claim types.

**Residual risk:** HIGH. Compromise of this single key compromises the entire compliance system. The testnet default key (line 840) is hardcoded in source, making it publicly known.

**Recommendation:** Implement role-based access control with separate keys for different claim types. Add multi-signature requirements for high-privilege operations (AccreditedInvestor, ExemptedEntity). Implement key rotation via an admin endpoint with its own authentication.

### 6.2 No Authentication on API Endpoints

**Threat:** Anyone can call any API endpoint.

**Analysis:** The router (`api.rs` lines 771-794) has no authentication middleware. All endpoints are publicly accessible:
- `POST /identity` — register arbitrary identities
- `POST /claim` — issue claims (uses server-side issuer key)
- `POST /transfer` — initiate transfers (requires ZK proof but no API auth)
- `POST /audit/commit` — commit audit hashes (requires private key in request body)
- `GET /zk-proof/{address}` — generate ZK proofs for any approved address

The CORS configuration is permissive: `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)` (lines 774-776).

**Residual risk:** HIGH. Any network-reachable attacker can interact with the full API surface. The claim issuance endpoint (`POST /claim`) is particularly sensitive because it uses the server-side issuer key to sign claims.

**Recommendation:** Add JWT or API key authentication middleware. Restrict claim issuance to authenticated admin sessions. The `POST /claim` endpoint should not be publicly accessible. Tighten CORS to specific allowed origins.

### 6.3 Compliance Engine State Manipulation

**Threat:** An attacker could modify the compliance rules.

**Analysis:** The `ComplianceEngine` exposes `add_requirement()` (`rules.rs` line 181), `add_rule()` (line 196), and `apply_jurisdiction_profile()` (line 187). However, these methods are not exposed through API endpoints. The engine is initialized with default rules at startup (`rules.rs` lines 165-169) and cannot be modified via HTTP.

**Residual risk:** LOW. The compliance rules are immutable after server startup. An attacker would need server-side code execution to modify them.

---

## Summary Risk Matrix

| Category | Threat | Severity | Current Mitigation | Residual Risk |
|---|---|---|---|---|
| Spoofing | Issuer key from env var | High | Ed25519 signatures | HIGH |
| Spoofing | Identity registration without auth | Medium | SQLite UNIQUE constraint | MEDIUM |
| Spoofing | ZK proof for any address | High | Groth16 soundness | HIGH |
| Tampering | Compliance result TOCTOU | Low | Scoped mutex, audit hash | LOW |
| Tampering | Merkle tree via fake registration | Medium | Chained with spoofing 1.2 | MEDIUM |
| Repudiation | Compliance decision denial | Low | On-chain audit hash | LOW |
| Repudiation | Claim issuance denial | Medium | Ed25519 sig, no on-chain record | MEDIUM |
| Info Disclosure | PII in ZK proofs | Low | Groth16 ZK property | MEDIUM (logs) |
| Info Disclosure | Private key in request body | Critical | None | CRITICAL |
| DoS | No rate limiting | High | None | HIGH |
| DoS | SQLite lock contention | Medium | Mutex scoping | MEDIUM |
| EoP | Single admin key | High | None | HIGH |
| EoP | No API authentication | High | None | HIGH |

---

## Recommended Priority Fixes

1. **CRITICAL:** Remove private key from `POST /transfer` request body. Implement client-side signing.
2. **HIGH:** Add API authentication middleware (JWT or API key) to all mutation endpoints.
3. **HIGH:** Add rate limiting via `tower::limit::RateLimitLayer`.
4. **HIGH:** Replace env-var issuer key with HSM-backed key management.
5. **HIGH:** Fix ZK proof generation to require actual private key knowledge, not just address hash.
6. **MEDIUM:** Add identity registration authentication (signature challenge).
7. **MEDIUM:** Encrypt SQLite at rest; reduce PII in log output.
8. **LOW:** Make audit hash commit mandatory or add local audit log backup.
