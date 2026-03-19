<!-- DISCLAIMER: Technical demo code -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint Compliance API Reference

Base URL: `http://localhost:3001`

Source: [`services/assetmint-core/src/api.rs`](../services/assetmint-core/src/api.rs)

## Endpoint Summary

| # | Method | Path | Description | Status |
|---|--------|------|-------------|--------|
| 1 | GET | `/health` | Service health check with Kaspa connectivity | Real |
| 2 | GET | `/network` | Live Kaspa Testnet-12 network info | Real |
| 3 | GET | `/balance` | Live on-chain balance query | Real |
| 4 | POST | `/identity` | Register a new DID identity | Real |
| 5 | POST | `/claim` | Issue a signed compliance claim | Real |
| 6 | GET | `/compliance/evaluate` | Evaluate transfer compliance rules | Real |
| 7 | GET | `/merkle-root` | Current Merkle root of approved addresses | Real |
| 8 | GET | `/zk-proof/{address}` | Generate Groth16 ZK-KYC proof | Real |
| 9 | POST | `/transfer` | Compliance-gated on-chain transfer | Real |
| 10 | POST | `/audit/commit` | Commit audit hash to Kaspa DAG | Real |
| 11 | POST | `/vc/issue` | Issue a W3C Verifiable Credential | Real |
| 12 | POST | `/vc/verify` | Verify a W3C Verifiable Credential proof | Real |

All endpoints operate against real in-memory state. Endpoints marked "Real" execute actual cryptographic operations (Groth16 proving, Ed25519 signatures, SHA-256 Merkle trees). Blockchain endpoints (network, balance, transfer, audit/commit) require a running kaspad on TN12.

---

## 1. GET /health

Service health check. Always returns 200.

**Response:**

```json
{
  "status": "ok",
  "service": "assetmint-core",
  "kaspa_connected": true
}
```

---

## 2. GET /network

Live Kaspa Testnet-12 network information. Requires kaspad connection.

**Response:**

```json
{
  "server_version": "0.13.4",
  "is_synced": true,
  "virtual_daa_score": 58234567,
  "network_id": "testnet-12",
  "block_count": 1234567,
  "difficulty": 1.23e+12
}
```

---

## 3. GET /balance

Live on-chain balance query.

**Query parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | string | Yes | Kaspa address (e.g. `kaspatest:qq...`) |

**Response:**

```json
{
  "address": "kaspatest:qq...",
  "balance_sompis": 500000000,
  "balance_kas": 5.0,
  "utxo_count": 3
}
```

---

## 4. POST /identity

Register a new DID identity in the compliance registry.

**Request body:**

```json
{
  "did": "did:kaspa:alice",
  "primary_key": "0xabc123..."
}
```

**Response (201 Created):**

```json
{
  "did": "did:kaspa:alice",
  "primary_key": "0xabc123...",
  "created_at": 1710000000
}
```

---

## 5. POST /claim

Issue a signed compliance claim for a registered identity.

**Request body:**

```json
{
  "subject_did": "did:kaspa:alice",
  "claim_type": "KycVerified",
  "jurisdiction": null,
  "expiry": 0
}
```

Supported claim types: `KycVerified`, `AccreditedInvestor`, `JurisdictionAllowed` (requires `jurisdiction` field), `AmlClear`, `ExemptedEntity`.

**Response (201 Created):**

```json
{
  "claim_type": "KycVerified",
  "issuer_did": "did:kaspa:assetmint-issuer",
  "subject_did": "did:kaspa:alice",
  "expiry": 0,
  "signature": "a1b2c3..."
}
```

---

## 6. GET /compliance/evaluate

Evaluate whether a transfer between two identities is compliant.

**Query parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `sender_did` | string | Yes | Sender DID |
| `receiver_did` | string | Yes | Receiver DID |
| `asset_id` | string | Yes | Asset identifier |
| `amount` | u64 | Yes | Transfer amount |
| `mint_timestamp` | u64 | No | Original mint timestamp (for lockup checks) |

**Response:**

```json
{
  "allowed": true,
  "violations": [],
  "rules_evaluated": 3
}
```

---

## 7. GET /merkle-root

Current Merkle root of all approved (KYC-verified) addresses.

**Response:**

```json
{
  "root": "a1b2c3d4...64-hex-chars",
  "leaf_count": 2
}
```

---

## 8. GET /zk-proof/{address}

Generate a Groth16 ZK-KYC proof that the given address is in the approved KYC set, without revealing the address to the verifier.

**Path parameters:**

| Param | Type | Description |
|-------|------|-------------|
| `address` | string | Address in the approved KYC set |

**Response:**

```json
{
  "proof": "hex-encoded-groth16-proof",
  "public_inputs": ["hex-merkle-root", "hex-nullifier-hash"],
  "proof_hash": "hex-sha256-of-proof",
  "merkle_root": "hex-merkle-root"
}
```

---

## 9. POST /transfer

Compliance-gated on-chain transfer. Evaluates rules, verifies ZK proof, then builds and broadcasts a Kaspa transaction. Uses server-side operator key (`OPERATOR_PRIVATE_KEY` env var) -- no private keys in API requests.

**Request body:**

```json
{
  "sender_did": "did:kaspa:alice",
  "receiver_did": "did:kaspa:bob",
  "receiver_address": "kaspatest:qq...",
  "amount_sompis": 100000000,
  "asset_id": "KPROP-NYC-TEST",
  "zk_proof": "hex-encoded-groth16-proof",
  "zk_public_inputs": ["hex-merkle-root", "hex-nullifier-hash"]
}
```

**Response:**

```json
{
  "tx_id": "abcdef1234...",
  "compliance_result": {
    "allowed": true,
    "violations": [],
    "rules_evaluated": 3
  },
  "amount_sompis": 100000000,
  "fee_sompis": 13000
}
```

---

## 10. POST /audit/commit

Commit a compliance audit hash to the Kaspa DAG for tamper-proof record keeping.

**Request body:**

```json
{
  "decision_hash": "hex-encoded-32-byte-hash",
  "from_address": "kaspatest:qq...",
  "private_key": "hex-encoded-32-bytes"
}
```

**Response:**

```json
{
  "tx_id": "abcdef1234...",
  "audit_hash": "hex-encoded-32-byte-hash",
  "timestamp": 1710000000
}
```

---

## 11. POST /vc/issue

Issue a compliance claim as a W3C Verifiable Credential.

**Request body:**

```json
{
  "subject_did": "did:kaspa:alice",
  "claim_type": "KycVerified",
  "jurisdiction": null,
  "expiry": 0
}
```

**Response (201 Created):**

```json
{
  "verifiable_credential": {
    "context": ["https://www.w3.org/2018/credentials/v1"],
    "vc_type": ["VerifiableCredential"],
    "issuer": "did:kaspa:assetmint-issuer",
    "issuance_date": "2026-03-19T00:00:00Z",
    "credential_subject": {
      "id": "did:kaspa:alice",
      "claim_type": "KycVerified"
    },
    "proof": { "...": "Ed25519 signature" }
  }
}
```

---

## 12. POST /vc/verify

Verify a W3C Verifiable Credential proof.

**Request body:**

```json
{
  "verifiable_credential": { "...VC object from /vc/issue..." }
}
```

**Response:**

```json
{
  "valid": true,
  "subject_did": "did:kaspa:alice",
  "claim_type": "KycVerified"
}
```

---

## Error Responses

All endpoints return errors in a consistent format:

```json
{
  "error": "Description of what went wrong"
}
```

Common HTTP status codes: 400 (bad request), 401 (unauthorized -- missing or invalid API key), 404 (identity not found), 409 (duplicate identity), 412 (precondition failed), 429 (rate limit exceeded), 502 (kaspad unreachable), 503 (Kaspa client not connected).

## Authentication

Write endpoints (`POST /identity`, `POST /claim`, `POST /transfer`, `POST /audit/commit`, `POST /vc/issue`) require an API key when `API_KEY` is set.

Pass the key via the `X-API-Key` header:

```bash
curl -X POST http://localhost:3001/identity \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"did": "did:kaspa:alice", "primary_key": "0xabc123"}'
```

Read endpoints (`GET /health`, `GET /network`, `GET /balance`, etc.) do not require authentication.

## Rate Limiting

All endpoints are rate-limited to 100 requests per minute per IP address. Exceeding this limit returns HTTP 429.

## Request Size Limits

Request bodies are limited to 1MB.

---

# CLI Reference

The `assetmint` CLI binary provides a command-line interface to the Compliance API. It communicates with the running Axum HTTP server.

Source: [`services/assetmint-core/src/bin/cli.rs`](../services/assetmint-core/src/bin/cli.rs)

## Running the CLI

```bash
# Via make
make cli ARGS="<command>"

# Via cargo
cargo run -p assetmint-core --bin assetmint -- <command>
```

## Commands

| Command | Description |
|---------|-------------|
| `health` | Check API health status |
| `network` | Display Kaspa network information |
| `identity register --did <DID> --key <HEX>` | Register a new DID identity |
| `identity get --did <DID>` | Look up an identity by DID |
| `claim issue --subject <DID> --type <TYPE> --expiry <UNIX>` | Issue a signed compliance claim |
| `compliance check --sender <DID> --receiver <DID> --asset <ID> --amount <N>` | Evaluate transfer compliance |
| `balance --address <ADDR>` | Query address balance on Kaspa |
| `transfer --sender-did <DID> --receiver-did <DID> --receiver-address <ADDR> --amount <N> --asset <ID>` | Execute a compliant transfer |
| `merkle-root` | Query current Merkle root |

## Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--api-url <URL>` | Base URL of the AssetMint API server | `http://localhost:3001` |
| `--api-key <KEY>` | API key for authenticated write endpoints | None |

## Example: Full Compliance Cycle

```bash
# Register two identities
assetmint identity register --did did:kaspa:alice --key 0xabc123
assetmint identity register --did did:kaspa:bob --key 0xdef456

# Issue KYC claims
assetmint --api-key YOUR_KEY claim issue --subject did:kaspa:alice --type KycVerified --expiry 0
assetmint --api-key YOUR_KEY claim issue --subject did:kaspa:bob --type KycVerified --expiry 0

# Check compliance
assetmint compliance check --sender did:kaspa:alice --receiver did:kaspa:bob --asset KPROP-NYC-TEST --amount 1000

# Query Merkle root
assetmint merkle-root
```

---

# Sovereign Metadata API Reference

Base URL: `http://localhost:8900`

Source: [`infrastructure/dkg-node/sovereign-metadata/server.js`](../infrastructure/dkg-node/sovereign-metadata/server.js)

Self-hosted, private-by-default metadata store with SHA-256 integrity hashes and tamper detection. Replaces OriginTrail DKG Edge Node.

## Endpoint Summary

| # | Method | Path | Description | Status |
|---|--------|------|-------------|--------|
| 1 | POST | `/publish` | Store asset metadata, returns UAL and SHA-256 hash | Real |
| 2 | GET | `/get` | Retrieve metadata by UAL | Real |
| 3 | POST | `/verify` | Verify metadata integrity against stored hash | Real |
| 4 | GET | `/assets` | List all published assets | Real |
| 5 | GET | `/info` | Service info (version, storage path) | Real |
| 6 | GET | `/health` | Service health check | Real |
| 7 | POST | `/metadata/publish-and-commit` | Publish metadata and commit hash to Kaspa DAG | Real |

---

## 1. POST /publish

Store asset metadata with a SHA-256 integrity hash.

**Request body:**

```json
{
  "name": "Test Asset",
  "ticker": "KTEST",
  "type": "real-estate",
  "jurisdiction": "US"
}
```

**Response:**

```json
{
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "metadata_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "status": "published",
  "private": true,
  "verify_instruction": "Commit metadata_hash on-chain via POST /audit/commit to make it verifiable on Kaspa DAG"
}
```

---

## 2. GET /get

Retrieve metadata by UAL.

**Query parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `ual` | string | Yes | Asset UAL (e.g. `did:assetmint:sovereign/a1b2c3d4e5f67890`) |

**Response:**

```json
{
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "metadata": {
    "name": "Test Asset",
    "ticker": "KTEST",
    "type": "real-estate",
    "jurisdiction": "US"
  },
  "metadata_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "created_at": "2026-03-19T12:00:00Z"
}
```

---

## 3. POST /verify

Verify metadata integrity against stored SHA-256 hash.

**Request body:**

```json
{
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "metadata": {
    "name": "Test Asset",
    "ticker": "KTEST",
    "type": "real-estate",
    "jurisdiction": "US"
  }
}
```

**Response:**

```json
{
  "verified": true,
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "stored_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "computed_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "tampered": false
}
```

---

## 4. GET /assets

List all published assets.

**Response:**

```json
{
  "assets": [
    {
      "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
      "metadata_hash": "f0e1d2c3...",
      "created_at": "2026-03-19T12:00:00Z"
    }
  ],
  "count": 1
}
```

---

## 5. GET /info

Service information.

**Response:**

```json
{
  "service": "sovereign-metadata",
  "version": "1.0.0",
  "storage": "/data/metadata.json"
}
```

---

## 6. GET /health

Service health check.

**Response:**

```json
{
  "status": "ok",
  "service": "sovereign-metadata"
}
```

---

## 7. POST /metadata/publish-and-commit

Publish metadata and commit the SHA-256 hash to the Kaspa DAG in a single atomic operation. Combines `POST /publish` with an on-chain hash commitment.

**Request body:**

```json
{
  "name": "Test Asset",
  "ticker": "KTEST",
  "type": "real-estate",
  "jurisdiction": "US",
  "from_address": "kaspatest:qq...",
  "private_key": "hex-encoded-32-bytes"
}
```

**Response:**

```json
{
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "metadata_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "status": "published_and_committed",
  "tx_id": "abcdef1234...",
  "private": true
}
```
