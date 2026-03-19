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
  "service": "compliance-rust",
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

Compliance-gated on-chain transfer. Evaluates rules, verifies ZK proof, then builds and broadcasts a Kaspa transaction.

**Request body:**

```json
{
  "sender_did": "did:kaspa:alice",
  "receiver_did": "did:kaspa:bob",
  "sender_private_key": "hex-encoded-32-bytes",
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

Common HTTP status codes: 400 (bad request), 404 (identity not found), 409 (duplicate identity), 412 (precondition failed), 502 (kaspad unreachable), 503 (Kaspa client not connected).
