# AssetMint Security Audit Report

> **DISCLAIMER**: This security audit is performed on the AssetMint Testnet-12
> demonstration codebase. This report is provided "AS IS" for informational
> purposes only. It does not constitute legal, financial, or regulatory advice.
> The findings herein reflect the state of the code at the time of review and
> should not be interpreted as a guarantee of security for any production
> deployment. A formal third-party audit by a licensed security firm is required
> before any mainnet or production use.

---

| Field              | Value                                                   |
|--------------------|---------------------------------------------------------|
| **Project**        | AssetMint -- Real-World Asset Tokenization on Kaspa     |
| **Version**        | Milestone 0 (Testnet-12 Demo)                           |
| **Date**           | 2026-03-18                                              |
| **Auditor**        | Security Engineer Agent (Internal)                      |
| **Methodology**    | Manual code review, STRIDE threat modeling, OWASP Top 10 |
| **Scope**          | Smart contracts, ZK circuits, compliance engine, oracle pool, tokenomics, frontend |
| **Classification** | CONFIDENTIAL -- Internal Use                            |

---

## Security Fixes Applied (2026-03-19)

The following critical and high-severity issues identified in this audit have been FIXED:

| Issue | Severity | Status | Fix |
|-------|----------|--------|-----|
| Toy hash function in ZK circuit | Critical | **FIXED** | 80-round Feistel MiMC with NUMS constants |
| Deterministic RNG in trusted setup | Critical | **FIXED** | `OsRng` (system entropy) |
| Hardcoded claim issuer key | Critical | **FIXED** | `CLAIM_ISSUER_KEY` env var |
| Private keys in API requests | Critical | **FIXED** | Server-side `OPERATOR_PRIVATE_KEY` |
| No authentication on API | High | **FIXED** | API key auth on write endpoints |
| Rate limiter not applied | High | **FIXED** | Middleware properly wired |
| No input validation | High | **FIXED** | DID regex + hex key validation |
| CORS allow all origins | High | **FIXED** | Restricted to `CORS_ORIGIN` |
| No body size limit | High | **FIXED** | 1MB limit (API + metadata) |
| XOR threshold unguarded | Medium | **FIXED** | `#[deprecated]` + env gate |

### Remaining Known Issues
| Issue | Severity | Status |
|-------|----------|--------|
| No TLS/HTTPS | High | Open — use reverse proxy |
| Recursive ZK boolean witness | Medium | Open — documented limitation |
| XOR threshold (not MuSig2) | Medium | Gated but not replaced |
| No audit log persistence | Low | Open — stdout only |

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Security Review](#2-architecture-security-review)
3. [Smart Contract Security (SilverScript)](#3-smart-contract-security-silverscript)
4. [ZK Circuit Security](#4-zk-circuit-security)
5. [Compliance Engine Security](#5-compliance-engine-security)
6. [Oracle Security](#6-oracle-security)
7. [Frontend Security](#7-frontend-security)
8. [Threat Model (STRIDE)](#8-threat-model-stride)
9. [Known Limitations (Demo vs Production)](#9-known-limitations-demo-vs-production)
10. [Recommendations](#10-recommendations)
11. [Conclusion](#11-conclusion)

---

## 1. Executive Summary

### Scope

This audit covers the full AssetMint stack as of Milestone 0:

- **5 SilverScript contracts**: `rwa-core.sil`, `clawback.sil`, `state-verity.sil`, `zkkyc-verifier.sil`, `reserves.sil`
- **ZK-KYC circuit**: Groth16 over BN254 with simplified MiMC-like hash (`zk-circuits/`)
- **Compliance engine**: Identity registry, claims issuance, transfer rules (`services/assetmint-core/`)
- **Oracle pool**: Centralized 2-of-3 multisig price attestation (`services/oracle-pool/`)
- **ASTM tokenomics**: KRC-20 inscriptions, staking, governance, fee model (`tokenomics/`)
- **Frontend dashboard**: Next.js with simulated wallet (`apps/dashboard-fe/`)
- **State sync service**: Merkle root polling and covenant state transitions (`services/sync/`)

### Methodology

- Static analysis of all Rust source files and SilverScript contracts
- Manual review against OWASP Top 10, CWE Top 25, and blockchain-specific vulnerability classes
- STRIDE threat modeling across all trust boundaries
- Cryptographic primitive assessment (hash functions, signature schemes, ZK setup)
- Dependency review of `Cargo.toml` configurations

### Risk Rating Summary

| Severity     | Count | Status                                  |
|--------------|-------|-----------------------------------------|
| Critical     | 2     | Accepted for demo; must fix for production |
| High         | 4     | Mitigated in demo scope; required for mainnet |
| Medium       | 6     | Documented with remediation paths       |
| Low          | 5     | Best-practice improvements              |
| Informational| 3     | Design considerations                   |

**Overall Risk: MEDIUM** -- Acceptable for Testnet-12 demonstration purposes. NOT production-ready.

---

## 2. Architecture Security Review

### Trust Boundary Diagram

```
 +-------------------------------------------------------------------+
 |                         UNTRUSTED ZONE                             |
 |                                                                    |
 |   +-------------------+       +----------------------------+      |
 |   |   Browser/User    |       |   External Price Feeds     |      |
 |   |   (dashboard-fe)  |       |   (simulated sources)      |      |
 |   +--------+----------+       +-------------+--------------+      |
 |            |                                |                     |
 +============|================================|=====================+
              | HTTP (no TLS)                  | Simulated (in-proc)
 +============|================================|=====================+
 |            v          TRUST BOUNDARY 1      v                     |
 |   +--------+----------+       +-------------+--------------+      |
 |   |  Compliance API   |       |     Oracle Pool Service    |      |
 |   |  (axum, :3001)    |       |     (attestation.rs)       |      |
 |   |  - Identity CRUD  |       |     - 2-of-3 multisig      |      |
 |   |  - Claim issuance |       |     - Price aggregation    |      |
 |   |  - Transfer eval  |       +----------------------------+      |
 |   |  - Merkle root    |                                           |
 |   +--------+----------+         SEMI-TRUSTED SERVICE ZONE         |
 |            |                                                      |
 +============|======================================================+
              | SQLite (in-proc)
 +============|======================================================+
 |            v          TRUST BOUNDARY 2                             |
 |   +--------+----------+       +----------------------------+      |
 |   |  SQLite Database  |       |   ZK Prover / Verifier     |      |
 |   |  (identity, claims|       |   (Groth16/BN254)          |      |
 |   |   in-memory/:file)|       |   - Trusted setup keys     |      |
 |   +-------------------+       |   - Proof gen/verify       |      |
 |                                +----------------------------+      |
 |                       TRUSTED DATA ZONE                           |
 +===================================================================+
              |
 +============|======================================================+
 |            v          TRUST BOUNDARY 3                             |
 |   +-------------------+       +----------------------------+      |
 |   | Kaspa Testnet-12  |       |   DKG Edge Node            |      |
 |   | - UTXO covenants  |       |   (Knowledge Assets)       |      |
 |   | - KRC-20 inscript.|       +----------------------------+      |
 |   | - State chain     |                                           |
 |   +-------------------+         ON-CHAIN / EXTERNAL ZONE          |
 +===================================================================+
```

### Data Flow Analysis

1. **Identity Registration**: Browser -> Compliance API -> SQLite. No authentication on the API endpoint. Any caller can register arbitrary DIDs.

2. **Claim Issuance**: Browser -> Compliance API -> Ed25519 signing -> SQLite. The claim issuer key is hardcoded as `[42u8; 32]` in `create_default_state()` (file: `services/assetmint-core/src/api.rs`, line 267).

3. **Transfer Evaluation**: Browser -> Compliance API -> load identity + claims from SQLite -> evaluate rules -> return JSON. Transfer parameters arrive via query string (GET request) which may be logged in access logs.

4. **ZK Proof Flow**: Prover loads proving key from disk -> builds Merkle tree -> generates Groth16 proof -> sends proof hash to SilverScript covenant. The proof itself is verified off-chain; only the hash commitment goes on-chain.

5. **Oracle Attestation**: Simulated price sources (in-process) -> median aggregation with outlier rejection -> 2-of-3 Ed25519 multisig -> attestation hash committed to `state-verity.sil`.

6. **State Sync**: Polls DKG endpoint -> compares assertion IDs -> creates state transition -> spends previous state UTXO via `state-verity.sil` covenant.

### Attack Surface Mapping

| Surface                | Exposure          | Risk    |
|------------------------|-------------------|---------|
| Compliance REST API    | Network (HTTP)    | High    |
| SQLite database        | Local filesystem  | Medium  |
| ZK proving/verifying keys | Filesystem     | High    |
| SilverScript covenants | Kaspa Testnet     | Medium  |
| Oracle signer keys     | In-memory         | Critical|
| KRC-20 inscriptions    | Kaspa Testnet     | Medium  |
| Frontend dashboard     | Browser           | Low     |
| DKG Edge Node polling  | HTTP              | Medium  |

---

## 3. Smart Contract Security (SilverScript)

### 3.1 Reentrancy Analysis -- Not Applicable (UTXO Model)

SilverScript contracts execute on Kaspa's UTXO model, not an account-based model. Each UTXO can only be spent once, and covenant scripts execute atomically during transaction validation. There is no concept of external calls mid-execution that could re-enter the contract. **Reentrancy is structurally impossible in this architecture.**

However, the UTXO model introduces its own class of vulnerabilities: state corruption via covenant preservation, UTXO value draining via fee manipulation, and race conditions on unconfirmed covenant UTXOs.

### 3.2 State Corruption via `validateOutputState`

**Severity: Medium**

All five contracts use `validateOutputState()` to enforce covenant preservation -- the output UTXO must recreate the same contract with specified state fields. This is the critical security invariant.

**Findings:**

- **rwa-core.sil (line 64)**: Correctly preserves `zkVerifierKeyHash` and `issuerKeyHash` while allowing `merkleRoot` updates via `zkTransfer`. The `adminUpdate` path (line 84) also correctly preserves all three fields. However, the `zkTransfer` path allows the **sender** to specify `newMerkleRoot`, meaning any valid signer can rotate the approved address set. This should be restricted to the admin path only.

- **state-verity.sil (line 52-57)**: Correctly preserves `stateManagerKeyHash` across state transitions while updating the three state fields. The `managerReclaim` path (line 62-65) intentionally does NOT preserve the covenant, allowing the state manager to terminate the state chain. This is correct but should be documented as an irreversible action.

- **zkkyc-verifier.sil (line 65-69, 89-93)**: Both paths correctly preserve `adminKeyHash`, preventing privilege escalation. The `updateVerifierKey` path allows the admin to rotate both the verifier key hash and the Merkle root atomically, which is correct behavior.

- **reserves.sil (line 71-76, 88-92)**: Covenant preservation is correct on both `withdraw` and `deposit` paths. The `custodianReclaim` path (line 97-100) intentionally breaks the covenant for emergency fund recovery.

- **clawback.sil**: This contract does NOT use `validateOutputState` on either path, which is correct -- clawback outputs are standard P2PK, not covenant UTXOs.

**Risk: The `zkTransfer` path in `rwa-core.sil` allows any authorized transferor to modify the Merkle root. An attacker who compromises a single user key could add arbitrary addresses to the approved set.**

### 3.3 Key Management Risks

**Severity: High**

All contracts use `blake2b(pubkey) == keyHash` for identity verification of privileged roles:

- `rwa-core.sil`: `issuerKeyHash` controls Merkle root updates
- `clawback.sil`: `issuerKeyHash` controls unconditional clawback
- `state-verity.sil`: `stateManagerKeyHash` controls state transitions
- `zkkyc-verifier.sil`: `adminKeyHash` controls verifier key rotation
- `reserves.sil`: `oracleKeyHash` + `custodianKeyHash` control withdrawals and emergency reclaim

**Findings:**

- Key hashes are immutable constructor arguments. If a private key is compromised, the covenant UTXO must be migrated to a new contract with a new key hash. There is no on-chain key rotation mechanism for the admin/issuer role itself.
- The `clawback.sil` contract gives the issuer **unconditional** reclaim power with no time-lock, multi-sig, or governance approval. A compromised issuer key means immediate loss of all clawback-wrapped UTXOs.
- The `reserves.sil` custodian has the same unconditional reclaim power (line 97-100), creating a single point of failure for all reserve funds.

**Remediation:**
- Add time-locked key rotation covenants
- Require multi-sig for critical operations (clawback, custodian reclaim)
- Implement governance-gated key rotation for admin roles

### 3.4 Integer Overflow in Reserves Ratio Check

**Severity: Medium**

In `reserves.sil` (lines 59-61):

```
require(
    remainingReserve * RATIO_DENOMINATOR >= minReserveRatio * attestedTokenSupplyValue
);
```

Both sides of this comparison involve multiplication of potentially large integers. SilverScript uses arbitrary-precision integers (inherited from Kaspa script), so there is **no overflow risk** in the traditional sense. However, extremely large attested values could cause excessive computation. The contract correctly validates `attestedTokenSupplyValue > 0` to prevent division-by-zero equivalents.

**Status: No vulnerability, but add bounds checking on attested values for defense-in-depth.**

### 3.5 Constructor Argument Injection

**Severity: Low**

SilverScript constructor arguments (`initMerkleRoot`, `initIssuerKeyHash`, etc.) are set at deployment time and become part of the contract's script hash. An attacker cannot modify these after deployment. However, if the deployment transaction is crafted by a compromised tool or the constructor arguments are not verified before deployment, a malicious deployer could embed incorrect key hashes.

**Remediation:** Verify all constructor arguments via an independent tool before broadcast. Log deployment transaction hashes for audit trail.

### 3.6 Covenant Preservation Correctness

**Severity: Informational**

The `validateOutputState()` function in SilverScript ensures the output UTXO's script matches the current contract with the specified state values. This is the foundational security mechanism. Review of all five contracts confirms that:

- State fields that must remain constant are explicitly passed through unchanged
- State fields that are updated receive new values from function arguments
- The `validateOutputState` index matches the intended output position

One concern: in `rwa-core.sil`, the covenant output index is `1` (for change), while in `state-verity.sil` and `zkkyc-verifier.sil` it is `0`. This is correct per the transaction structure but must be carefully maintained during any contract modifications.

---

## 4. ZK Circuit Security

### 4.1 Soundness: MiMC-like Hash is NOT Cryptographically Secure

**Severity: Critical (for production)**

The hash function used inside the ZK circuit (`zk-circuits/src/kyc_circuit.rs`, lines 93-107) is:

```
H(a, b) = (a + b)^5 + a*b + 7
```

This is explicitly documented as a demonstration-only construction. The specific weaknesses:

- **Algebraic degree is only 5**: This polynomial is trivially invertible over the BN254 scalar field. Given `H(a, b)` and one input, recovering the other is a polynomial root-finding problem solvable in milliseconds.
- **No round keys or permutation rounds**: Real MiMC uses 220+ rounds with distinct round constants. This single-round construction provides no collision resistance.
- **Collision attacks**: An attacker can find distinct `(a, b)` pairs that produce the same hash, allowing them to forge Merkle tree membership proofs for addresses not in the approved set.
- **Preimage attacks**: Given a leaf hash, recovering the secret key is computationally trivial, completely breaking the privacy guarantee.

**Impact for demo:** Acceptable because Testnet-12 has no real assets at risk, and the ZK flow is being demonstrated architecturally rather than cryptographically.

**Impact for production:** Total break of ZK-KYC privacy and soundness. An attacker could generate valid proofs for arbitrary addresses without knowing the secret key.

### 4.2 Trusted Setup: Deterministic RNG is NOT Secure

**Severity: Critical (for production)**

The Groth16 trusted setup (`zk-circuits/src/setup.rs`, line 55) uses:

```rust
let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_BABE);
```

And the prover (`services/assetmint-core/src/zk_prover.rs`, line 138) uses:

```rust
let mut rng = StdRng::seed_from_u64(0xCAFE_BABE);
```

**Issues:**

- **Toxic waste recovery**: The Groth16 trusted setup generates "toxic waste" parameters that, if known, allow forging proofs for any statement. A deterministic seed means anyone who reads the source code can reconstruct the toxic waste and generate fake ZK proofs.
- **Proof randomness**: Groth16 proofs require per-proof randomness to prevent proof linkability. A deterministic seed with `seed_from_u64` means all proofs use the same randomness, enabling proof correlation attacks.
- **No ceremony artifacts**: There is no Powers-of-Tau or MPC ceremony, so there is no guarantee that the setup parameters are sound.

**Remediation:**
1. Replace the hash with Poseidon (recommended for BN254 circuits) or proper MiMC with 220+ rounds
2. Conduct a multi-party computation (MPC) ceremony for the trusted setup (e.g., using `snarkjs` Powers-of-Tau)
3. Use `OsRng` or `ThreadRng` for per-proof randomness in the prover

### 4.3 Nullifier Binding Analysis

**Severity: Low**

The nullifier is computed as `H(secret, 1)` where `H` is the simplified hash. The circuit correctly:

1. Computes the nullifier from the same secret used for the leaf hash (line 180-181 of `kyc_circuit.rs`)
2. Enforces equality between the computed nullifier and the public input (line 181)
3. Uses different domain separators for leaf (`H(secret, 0)`) and nullifier (`H(secret, 1)`)

**However**, because the hash function is not collision-resistant, an attacker could find a different secret that produces the same nullifier, allowing re-use of KYC credentials. With a cryptographically secure hash, the nullifier binding would be sound.

### 4.4 Groth16 Malleability Considerations

**Severity: Low**

Groth16 proofs have a known malleability property: given a valid proof `(A, B, C)`, it is possible to construct a different valid proof `(A', B', C')` for the same statement by negating certain curve points. This does not break soundness (the statement is still true) but can cause issues if proof uniqueness is required.

In AssetMint, the proof hash is committed on-chain via `sha256(proof_bytes)`. If a malleable proof is submitted, the on-chain hash will differ, but it will still pass off-chain verification. This is acceptable for the current architecture where the proof hash serves as a commitment rather than an identity.

**Remediation for production:** Use Groth16 with proof canonicalization, or migrate to PLONK/Halo2 which do not have this malleability property.

### 4.5 Recommendations for ZK System

| Priority | Action |
|----------|--------|
| P0       | Replace `(a+b)^5 + a*b + 7` with Poseidon hash (ark-crypto-primitives provides this) |
| P0       | Conduct MPC ceremony for trusted setup using Powers-of-Tau |
| P1       | Use `OsRng` for per-proof randomness in the prover |
| P2       | Consider migration to PLONK (universal setup, no per-circuit ceremony) |
| P2       | Add nullifier set tracking to prevent double-use on-chain |

---

## 5. Compliance Engine Security

### 5.1 SQL Injection via rusqlite

**Severity: Informational (No Vulnerability)**

The identity registry (`services/assetmint-core/src/identity.rs`) uses `rusqlite` with parameterized queries throughout. All user-supplied values are passed via `params![]` macro, which uses SQLite's prepared statement API with bound parameters.

Verified safe patterns:
- Line 98: `params![did, primary_key, now as i64]`
- Line 128: `params![did]`
- Line 148: `params![did]`
- Line 174: `params![did]`
- Line 196: `params![claim.subject_did, claim.issuer_did, ...]`

**No SQL injection vulnerability exists.** The `rusqlite` crate prevents injection by design when `params![]` is used.

### 5.2 Ed25519 Claim Signature Verification

**Severity: Low**

The claim signing and verification flow (`services/assetmint-core/src/claims.rs`) is correctly implemented:

- Claims are signed over `SHA256(subject_did || claim_type_json || expiry_le || issued_at_le)` (lines 121-127)
- Verification reconstructs the same canonical byte representation and verifies the Ed25519 signature (lines 148-167)
- Wrong keys correctly fail verification (tested in `test_wrong_key_fails_verification`)

**Minor concern:** The `build_claim_data` function uses `serde_json::to_string()` for the claim type, which could produce different serializations across different versions of serde_json if field ordering changes. This is acceptable for the current codebase but could become an issue if claims are verified across different software versions.

**Remediation:** Use a deterministic canonical serialization format (e.g., CBOR with sorted keys, or a fixed binary encoding).

### 5.3 Claim Expiry Race Conditions

**Severity: Medium**

Claims use `SystemTime::now()` for expiry checks (lines 138-145 of `claims.rs` and lines 164-165 of `rules.rs`). This creates two race conditions:

1. **TOCTOU between verification and use**: A claim could be verified as valid, then expire before the transfer transaction is confirmed on-chain. The on-chain covenant does not re-check claim expiry.

2. **Clock skew**: If the compliance service runs on multiple nodes with unsynchronized clocks, the same claim could be considered valid on one node and expired on another.

**Remediation:**
- Add a grace period buffer (e.g., require claims to be valid for at least 60 seconds beyond the current time)
- Use block timestamps from Kaspa DAG blue score rather than system time for on-chain-relevant decisions

### 5.4 DID Spoofing Risks

**Severity: Medium**

The identity registry (`identity.rs`) accepts arbitrary DID strings with no validation (line 88). An attacker could register DIDs that visually mimic legitimate identities:

- `did:kaspa:alice` vs `did:kaspa:aIice` (capital I vs lowercase L)
- `did:kaspa:alice` vs `did:kaspa:alice%20` (URL-encoded space)

The `register()` function checks for uniqueness via the `UNIQUE` constraint on the `did` column, but does not normalize or validate DID format.

**Remediation:**
- Validate DID format against a regex (e.g., `^did:kaspa:[a-z0-9]+$`)
- Normalize DIDs to lowercase before storage
- Require proof-of-possession of the Kaspa address corresponding to the DID

### 5.5 Rate Limiting on API Endpoints

**Severity: High**

The compliance API (`services/assetmint-core/src/api.rs`) has **no rate limiting** on any endpoint. The Axum router (lines 252-259) binds all routes without middleware for:

- Request rate limiting
- Authentication/authorization
- Request size limits
- IP-based throttling

An attacker could:
- Flood `/identity` with registration requests, filling the SQLite database
- Spam `/claim` to exhaust Ed25519 signing operations
- Abuse `/compliance/evaluate` to probe which DIDs have which claims
- Denial-of-service the service with minimal effort

**Remediation:**
- Add `tower::limit::RateLimitLayer` to the Axum router
- Implement API key authentication for claim issuance endpoints
- Add request body size limits via `axum::extract::DefaultBodyLimit`
- Deploy behind a reverse proxy with rate limiting (nginx, Cloudflare)

---

## 6. Oracle Security

### 6.1 Centralized Trust Assumption

**Severity: High (acknowledged by design)**

The oracle pool operates as a centralized service with 2-of-3 multisig attestation. Per guidance from the Kaspa core team (Ori Newman), this is the correct architecture for Testnet-12 because Kaspa does not yet support decentralized oracle mechanisms at the consensus layer.

**Current trust model:**
- Three oracle signers with deterministic seeds (`[1u8; 32]`, `[2u8; 32]`, `[3u8; 32]`)
- Any two signers can produce a valid attestation
- All three signers run in the same process (no physical separation)

**Risks:**
- A single compromised process exposes all three signing keys
- No key rotation mechanism exists
- No dispute resolution for conflicting attestations

### 6.2 Ed25519 Multisig Threshold (2-of-3)

**Severity: Medium**

The 2-of-3 threshold (`services/oracle-pool/src/attestation.rs`, lines 28-30) is correctly enforced:

- `create_attestation()` rejects requests with fewer than 2 signers (line 95-99)
- `verify_attestation()` counts valid signatures and checks against the threshold (lines 151-210)
- Tampered attestations correctly fail verification (tested in `test_tampered_attestation_fails`)

**Finding:** The verification function (`verify_attestation`) does not verify that the signer public keys are from a trusted set. It accepts ANY public key that produces a valid signature over the data. An attacker could create their own 2-of-3 multisig with arbitrary keys and the attestation would pass verification.

**Remediation:** Maintain a trusted signer registry and verify that `signer_pubkeys` in the attestation match the registered set.

### 6.3 Price Feed Manipulation Vectors

**Severity: Medium**

The price aggregation (`services/oracle-pool/src/oracle.rs`) uses median-based outlier rejection with a 20% threshold. Attack vectors:

- **Slow price drift**: An attacker controlling one source could gradually shift their price by 19% per update, staying under the outlier threshold while biasing the average.
- **Two-source collusion**: If two of three sources are compromised, they can set any price. The outlier rejection only works when the majority of sources are honest.
- **NaN/Infinity handling**: The `partial_cmp` on line 91 handles NaN via `unwrap_or(Ordering::Equal)`, which could cause a NaN price to be treated as equal to the median, bypassing the outlier filter. If a source returns NaN, it should be rejected before aggregation.

**Status:** Acceptable for demo with simulated sources. Must use real price feeds with source diversity for production.

### 6.4 Replay Protection

**Severity: Medium**

Attestations include a timestamp in the signed data (`services/oracle-pool/src/attestation.rs`, line 85):

```rust
hasher.update(price.timestamp.to_le_bytes());
```

This provides replay protection as long as:
- The on-chain consumer checks that the attestation timestamp is recent
- Timestamps are monotonically increasing

**Finding:** The `state-verity.sil` contract does NOT check the attestation timestamp against the current block time. It only verifies that `newOracleAttestationHash != 0`. A stale attestation from any point in time could be replayed to set the on-chain state to outdated values.

**Remediation:** Add timestamp validation in the state-verity covenant, or include a nonce/sequence number that the contract tracks.

### 6.5 Upgrade Path to Miner-Attested Oracle

**Severity: Informational**

The current architecture cleanly separates oracle attestation (off-chain) from on-chain state commitment (covenant hash). This design facilitates future migration to a decentralized oracle:

1. Replace `OracleSigner` with miner-attested data availability proofs
2. Update `oracleKeyHash` in `reserves.sil` and `state-verity.sil` to point to a new verification scheme
3. The covenant structure does not need to change -- only the off-chain attestation generation

---

## 7. Frontend Security

### 7.1 No Secrets in Client Code

**Severity: Informational (Pass)**

Reviewed all frontend source files in `apps/dashboard-fe/src/`. No API keys, private keys, or secrets are present in client-side code. API endpoints are configured via `NEXT_PUBLIC_` environment variables (`api.ts`, lines 4-5), which is the correct Next.js pattern for client-exposed configuration.

The wallet module (`wallet.ts`) uses a simulated testnet wallet with a hardcoded address (line 13). This is a testnet address with no real value, and the file is clearly marked `REPLACE_WITH_TESTNET_WALLET`.

### 7.2 API Endpoint Exposure

**Severity: Medium**

The frontend directly calls the compliance API via HTTP without authentication (`api.ts`). All API calls use plain `fetch()` with no:

- Authentication headers (no JWT, API key, or session token)
- CSRF protection tokens
- Request signing

Any user who discovers the API endpoint can call it directly, bypassing the frontend entirely. The `evaluateTransfer` endpoint (line 37-46) passes all parameters via query string, which will be logged in server access logs and browser history.

**Remediation:**
- Move sensitive parameters to POST request bodies
- Add authentication middleware to the compliance API
- Implement CSRF tokens for state-changing operations

### 7.3 CORS Configuration

**Severity: Medium**

The Axum compliance API (`api.rs`) does not configure CORS headers. By default, Axum does not add CORS headers, which means:

- Browsers will block cross-origin requests from the Next.js frontend (unless they are on the same origin)
- This is actually a security benefit if unintentional -- it prevents third-party sites from calling the API
- For the demo to work, the frontend and API must be on the same origin, or a CORS layer must be added

**Remediation:** Add `tower-http::cors::CorsLayer` with explicit origin allowlist. Never use `Access-Control-Allow-Origin: *` in production.

### 7.4 Simulated Wallet -- No Real Key Management

**Severity: Informational**

The wallet (`wallet.ts`) is a simulated testnet wallet that returns a hardcoded address and balance. No private key material is generated, stored, or used in the frontend. This is appropriate for a demo.

**Production requirements:**
- Integrate `kaspa-wasm` SDK for real wallet operations
- Support hardware wallet connections (Ledger, Trezor) via WebUSB/WebHID
- Never store private keys in browser localStorage or sessionStorage
- Use `SubtleCrypto` for any client-side cryptographic operations

---

## 8. Threat Model (STRIDE)

| # | Threat | Category | Component | Severity | Likelihood | Mitigation |
|---|--------|----------|-----------|----------|------------|------------|
| T1 | Forged claim signatures allow unauthorized transfers | Spoofing | Compliance Engine | High | Low | Ed25519 signature verification on all claims; verify issuer public key against trusted set |
| T2 | Oracle price manipulation via compromised source | Tampering | Oracle Pool | High | Medium | 2-of-3 multisig threshold; 20% outlier rejection; add trusted signer registry |
| T3 | Clawback executed without audit trail | Repudiation | clawback.sil | Medium | Low | OP_RETURN output required (line 60); add off-chain event logging to compliance service |
| T4 | ZK proof linkability reveals identity | Information Disclosure | ZK Circuit | Medium | Medium | Nullifier binding prevents correlation; deterministic prover RNG weakens this -- use OsRng |
| T5 | Compliance API denial of service | Denial of Service | Compliance API | High | High | No rate limiting exists; add tower RateLimitLayer and deploy behind reverse proxy |
| T6 | Compromised issuer key enables unauthorized clawback | Elevation of Privilege | clawback.sil | Critical | Low | No on-chain multi-sig for clawback; add time-lock + multi-sig governance |
| T7 | Compromised custodian key drains all reserves | Elevation of Privilege | reserves.sil | Critical | Low | `custodianReclaim` has no restrictions; add time-lock and multi-sig |
| T8 | Stale oracle attestation replay | Tampering | state-verity.sil | Medium | Medium | No timestamp validation on-chain; add block-time freshness check |
| T9 | DID spoofing via visual similarity | Spoofing | Identity Registry | Medium | Medium | No DID format validation; add regex validation and normalization |
| T10 | Fake ZK proof via broken hash | Spoofing | ZK Circuit | Critical (prod) | High (prod) | Hash function is not collision-resistant; migrate to Poseidon |
| T11 | Trusted setup toxic waste recovery | Elevation of Privilege | ZK Setup | Critical (prod) | Certain | Deterministic seed is public; conduct MPC ceremony |
| T12 | Transfer parameters logged in URL | Information Disclosure | Frontend/API | Low | High | Use POST instead of GET for compliance evaluation |
| T13 | SQLite database file accessible | Information Disclosure | Identity Registry | Medium | Low | File permissions; use encrypted SQLite (SQLCipher) in production |
| T14 | Governance vote weight manipulation | Tampering | Tokenomics | Medium | Low | Vote weight is caller-supplied (line 131 governance.rs); verify against actual staked balance |
| T15 | Merkle root updated by any transferor | Elevation of Privilege | rwa-core.sil | High | Medium | `zkTransfer` allows sender-specified `newMerkleRoot`; restrict to admin path |

---

## 9. Known Limitations (Demo vs Production)

| Component | Demo Status | Production Requirement | Risk if Unchanged |
|-----------|-------------|------------------------|-------------------|
| MiMC hash | Simplified `(a+b)^5 + a*b + 7` | Poseidon hash with BN254-optimized parameters | Total ZK soundness break |
| Trusted setup | Deterministic `StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_BABE)` | Multi-party computation ceremony (Powers-of-Tau) | Proof forgery by anyone with source access |
| Prover RNG | Deterministic `StdRng::seed_from_u64(0xCAFE_BABE)` | `OsRng` or `ThreadRng` for per-proof randomness | Proof linkability and correlation |
| Oracle | Centralized 2-of-3 with in-process keys | Decentralized oracle or miner-attested data | Single point of failure |
| Oracle keys | Hardcoded seeds `[1u8; 32]`, `[2u8; 32]`, `[3u8; 32]` | HSM-stored keys with rotation | Trivial key recovery |
| Claim issuer key | Hardcoded `[42u8; 32]` in `create_default_state()` | Environment variable or Vault/HSM | Anyone can forge claims |
| Wallet | Simulated testnet (hardcoded address) | `kaspa-wasm` + hardware wallet support | No real transaction signing |
| Key storage | In-memory / SQLite file | HSM (AWS CloudHSM, HashiCorp Vault) | Key theft from memory dump |
| TLS | None (plain HTTP on all services) | mTLS between services, TLS 1.3 for external | Traffic interception |
| API auth | None (open endpoints) | JWT/OAuth 2.0 with RBAC | Unauthorized access |
| Rate limiting | None | Token bucket with per-IP and per-user limits | Denial of service |
| Logging | `tracing::info` to stdout | Structured logging to SIEM with PII redaction | Insufficient audit trail |
| CORS | Not configured | Strict origin allowlist | Cross-origin API abuse |
| Input validation | Minimal (Serde deserialization) | Schema validation + length limits on all fields | Injection / DoS via large payloads |

---

## 10. Recommendations

### P0 -- Critical (Must fix before any real-value deployment)

| # | Finding | Remediation | Effort |
|---|---------|-------------|--------|
| R1 | Simplified hash function in ZK circuit breaks soundness | Replace with Poseidon hash from `ark-crypto-primitives` crate; update constraint count estimates | 2-3 days |
| R2 | Deterministic trusted setup allows proof forgery | Implement MPC ceremony using `snarkjs` Powers-of-Tau or `arkworks` equivalent; publish ceremony transcripts | 1-2 weeks |
| R3 | Hardcoded signing keys (claim issuer, oracle signers) | Load keys from environment variables or secrets manager; add key rotation API | 1 day |
| R4 | No authentication on compliance API | Add JWT middleware or API key authentication to all state-changing endpoints | 2-3 days |

### P1 -- High (Required for mainnet readiness)

| # | Finding | Remediation | Effort |
|---|---------|-------------|--------|
| R5 | No rate limiting on API endpoints | Add `tower::limit::RateLimitLayer` to Axum router with per-IP throttling | 1 day |
| R6 | `zkTransfer` allows sender to set Merkle root | Remove `newMerkleRoot` parameter from `zkTransfer`; only allow via `adminUpdate` | 1 hour |
| R7 | No trusted signer registry for oracle verification | Maintain on-chain or off-chain registry of authorized oracle signer public keys; verify against it in `verify_attestation()` | 1 day |
| R8 | Clawback and custodian reclaim lack multi-sig | Modify `clawback.sil` and `reserves.sil` to require 2-of-3 multi-sig for privileged operations | 2 days |
| R9 | No TLS on service communication | Add TLS termination (nginx or native Rustls) for all HTTP endpoints; use mTLS between services | 2-3 days |

### P2 -- Medium (Should fix before production)

| # | Finding | Remediation | Effort |
|---|---------|-------------|--------|
| R10 | Claim expiry TOCTOU race condition | Add 60-second grace period buffer to claim verification; use DAG blue score for time-critical decisions | 1 day |
| R11 | DID format validation missing | Add regex validation (`^did:kaspa:[a-z0-9]+$`) and normalization in `register()` | 2 hours |
| R12 | Stale oracle attestation replay | Add timestamp or sequence number tracking in `state-verity.sil` covenant | 1 day |
| R13 | Transfer parameters in GET query string | Change `/compliance/evaluate` to POST method with JSON body | 1 hour |
| R14 | Use `OsRng` for Groth16 prover | Replace `StdRng::seed_from_u64` with `OsRng` in `zk_prover.rs` | 30 min |
| R15 | Governance vote weight is caller-supplied | Verify vote weight against actual staked ASTM balance from on-chain UTXO set | 1-2 days |

### P3 -- Low (Best practice improvements)

| # | Finding | Remediation | Effort |
|---|---------|-------------|--------|
| R16 | No CORS configuration on Axum | Add `tower-http::cors::CorsLayer` with explicit origin allowlist | 1 hour |
| R17 | SQLite database not encrypted | Use SQLCipher for encrypted-at-rest database in production | 1 day |
| R18 | Claim serialization uses JSON (non-deterministic) | Use CBOR or fixed binary format for canonical claim data encoding | 1 day |
| R19 | No structured audit logging | Add security event logging (identity creation, claim issuance, transfer denial) to SIEM-compatible format | 2 days |
| R20 | Constructor argument verification at deploy time | Build deployment verification tool that independently computes and displays constructor arg hashes | 1 day |

---

## 11. Conclusion

### Overall Risk Rating: MEDIUM

The AssetMint Milestone 0 codebase demonstrates a well-architected RWA tokenization platform with appropriate security patterns for a testnet demonstration:

**Strengths:**
- Clean separation of concerns between on-chain covenants and off-chain services
- Correct use of parameterized SQL queries (no injection risk)
- Proper Ed25519 signature verification with separate signing and verifying key types
- Covenant preservation correctly enforced via `validateOutputState`
- UTXO model eliminates entire classes of smart contract vulnerabilities (reentrancy, front-running)
- Comprehensive test coverage including negative cases (wrong key, tampered attestation, expired claims)
- Explicit `DISCLAIMER` headers on all files acknowledging demo status

**Weaknesses requiring remediation before production:**
- The ZK circuit's hash function and trusted setup are fundamentally broken for production use (by design for demo)
- All cryptographic keys are hardcoded with deterministic seeds
- No authentication, rate limiting, or TLS on the compliance API
- Critical privileged operations (clawback, custodian reclaim) lack multi-sig protection
- Oracle attestation replay is possible due to missing on-chain timestamp validation

**Assessment:** The codebase is **acceptable for Testnet-12 demonstration purposes**. The security risks are well-documented in the code itself (via DISCLAIMER headers and inline comments), and the architecture is designed to accommodate the production security requirements identified in this report. The P0 and P1 recommendations must be implemented before any deployment involving real assets or real user data.

---

*End of Security Audit Report*

*Report generated: 2026-03-18*
*Auditor: Security Engineer Agent*
*Classification: CONFIDENTIAL -- Internal Use*
