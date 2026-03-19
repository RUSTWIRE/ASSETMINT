# AssetMint Demo Walkthrough

> Step-by-step guide to seeing AssetMint's compliance-gated RWA platform in action
> on Kaspa Testnet-12.

**Prerequisites:**
- Rust toolchain installed
- Node.js 18+
- `kaspad` running on `ws://127.0.0.1:17210` (Testnet-12)
- Repository cloned and dependencies installed (`make setup`)

**Ports used:**
| Service | Port | Description |
|---------|------|-------------|
| Dashboard (Next.js) | 3000 | Investor-facing frontend |
| Compliance API (Axum) | 3001 | Identity, claims, ZK proofs, transfers |
| Oracle API | 3002 | Reserve attestations |
| Sovereign Metadata | 8900 | Private asset metadata with SHA-256 integrity |

---

## Step 1: Start the Platform

**What to do:**

```bash
cd /Users/rory/ASSETMINT
make demo
```

**What to expect:**

```
[K-RWA] Starting AssetMint Core API on :3001...
[K-RWA] Backend API ready: http://localhost:3001
[K-RWA] Health check: curl http://localhost:3001/health
[K-RWA] Starting dashboard on :3000...
[K-RWA] Frontend ready: http://localhost:3000
[K-RWA] ========================================
[K-RWA]  AssetMint Demo Running
[K-RWA]  Dashboard:  http://localhost:3000
[K-RWA]  API:        http://localhost:3001
[K-RWA]  Kaspad:     ws://127.0.0.1:17210
[K-RWA] ========================================
[K-RWA] Press Ctrl+C to stop all services
```

**Why this matters:** A single command starts the full stack: the Rust compliance
engine (with ZK trusted setup), the Next.js dashboard, and connects to Kaspa
Testnet-12.

---

## Step 2: Verify Services Are Running

### 2a. Health Check

**What to do:**

```bash
curl -s http://localhost:3001/health | jq
```

**What to expect:**

```json
{
  "status": "ok",
  "service": "assetmint-core",
  "kaspa_connected": true
}
```

**Why this matters:** Confirms the compliance API is up and has a live gRPC
connection to `kaspad`. If `kaspa_connected` is `false`, the service is running
in offline mode and on-chain operations will fail.

### 2b. Network Info

**What to do:**

```bash
curl -s http://localhost:3001/network | jq
```

**What to expect:**

```json
{
  "server_version": "0.16.1",
  "is_synced": true,
  "virtual_daa_score": 58123456,
  "network_id": "testnet-12",
  "block_count": 2841093,
  "difficulty": 1.234567890
}
```

**Why this matters:** Proves the API is reading live chain state from a synced
Testnet-12 node. The `virtual_daa_score` and `block_count` will increase on each
call.

---

## Step 3: Register Alice and Bob Identities

These DIDs represent two investors who will participate in a compliance-gated
token transfer.

### 3a. Register Alice

**What to do:**

```bash
curl -s -X POST http://localhost:3001/identity \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:kaspa:alice",
    "primary_key": "0xabc123"
  }' | jq
```

**What to expect:**

```json
{
  "did": "did:kaspa:alice",
  "primary_key": "0xabc123",
  "created_at": 1742400000
}
```

### 3b. Register Bob

**What to do:**

```bash
curl -s -X POST http://localhost:3001/identity \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:kaspa:bob",
    "primary_key": "0xdef456"
  }' | jq
```

**What to expect:**

```json
{
  "did": "did:kaspa:bob",
  "primary_key": "0xdef456",
  "created_at": 1742400001
}
```

**Why this matters:** Every participant must have a registered Decentralized
Identifier (DID) before they can receive KYC claims or participate in transfers.
The `created_at` is a Unix timestamp from registration time. Re-registering the
same DID returns HTTP 409 Conflict.

---

## Step 4: Issue KYC Claims for Both Identities

KYC claims attest that an identity has passed Know Your Customer verification.
The claim issuer (`did:kaspa:assetmint-issuer`) cryptographically signs each
claim.

### 4a. KYC Claim for Alice

**What to do:**

```bash
curl -s -X POST http://localhost:3001/claim \
  -H "Content-Type: application/json" \
  -d '{
    "subject_did": "did:kaspa:alice",
    "claim_type": "KycVerified",
    "expiry": 0
  }' | jq
```

**What to expect:**

```json
{
  "claim_type": "KycVerified",
  "issuer_did": "did:kaspa:assetmint-issuer",
  "subject_did": "did:kaspa:alice",
  "expiry": 0,
  "signature": "a3c1f9...hex-encoded-ed25519-signature..."
}
```

### 4b. KYC Claim for Bob

**What to do:**

```bash
curl -s -X POST http://localhost:3001/claim \
  -H "Content-Type: application/json" \
  -d '{
    "subject_did": "did:kaspa:bob",
    "claim_type": "KycVerified",
    "expiry": 0
  }' | jq
```

**What to expect:**

```json
{
  "claim_type": "KycVerified",
  "issuer_did": "did:kaspa:assetmint-issuer",
  "subject_did": "did:kaspa:bob",
  "expiry": 0,
  "signature": "b7d2e4...hex-encoded-ed25519-signature..."
}
```

**Why this matters:** Claims are signed by the issuer's Ed25519 key. The
`signature` field is a hex-encoded cryptographic signature that the compliance
engine verifies when evaluating transfers. An `expiry` of `0` means the claim
never expires.

**Valid claim types:** `KycVerified`, `AccreditedInvestor`, `AmlClear`,
`ExemptedEntity`, `JurisdictionAllowed` (requires `jurisdiction` field).

---

## Step 5: Evaluate a Compliant Transfer (Should PASS)

Both Alice and Bob have KYC claims. A transfer between them should pass all
compliance rules.

**What to do:**

```bash
curl -s "http://localhost:3001/compliance/evaluate?\
sender_did=did:kaspa:alice&\
receiver_did=did:kaspa:bob&\
asset_id=KPROP-NYC-TEST&\
amount=1000&\
mint_timestamp=0" | jq
```

**What to expect:**

```json
{
  "allowed": true,
  "violations": [],
  "rules_evaluated": 3
}
```

**Why this matters:** The compliance engine evaluated 3 rules (KYC verification,
AML screening, transfer limits) and found zero violations. The transfer is
cleared. This is the gate that every on-chain transfer must pass through.

---

## Step 6: Evaluate a Non-Compliant Transfer (Should FAIL)

Try a transfer where the sender has no KYC claim. Register a new identity
without issuing any claims.

### 6a. Register an un-verified identity

**What to do:**

```bash
curl -s -X POST http://localhost:3001/identity \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:kaspa:charlie",
    "primary_key": "0x999"
  }' | jq
```

### 6b. Evaluate the transfer

**What to do:**

```bash
curl -s "http://localhost:3001/compliance/evaluate?\
sender_did=did:kaspa:charlie&\
receiver_did=did:kaspa:bob&\
asset_id=KPROP-NYC-TEST&\
amount=1000&\
mint_timestamp=0" | jq
```

**What to expect:**

```json
{
  "allowed": false,
  "violations": [
    "Sender did:kaspa:charlie has no KycVerified claim"
  ],
  "rules_evaluated": 3
}
```

**Why this matters:** The compliance engine blocks transfers from identities
without valid KYC. This is enforced at the API layer before any on-chain
transaction is built. The `violations` array provides a human-readable
explanation of exactly which rules failed.

---

## Step 7: Generate a ZK Proof

Generate a Groth16 zero-knowledge proof that an address is in the approved KYC
Merkle tree, without revealing which address it is to the verifier.

**What to do:**

```bash
curl -s http://localhost:3001/zk-proof/0xabc123 | jq
```

The path parameter is the `primary_key` (address) that was registered for Alice
in Step 3.

**What to expect:**

```json
{
  "proof": "a4b3c2d1...long-hex-encoded-groth16-proof...",
  "public_inputs": [
    "e9f1a2...hex-merkle-root...",
    "c3d4e5...hex-nullifier-hash..."
  ],
  "proof_hash": "f0e1d2c3...sha256-of-proof-bytes...",
  "merkle_root": "e9f1a2...hex-merkle-root..."
}
```

**Why this matters:** This is a real Groth16 proof generated by the
`ark-groth16` library over the BN254 curve. The proof attests that the address
belongs to the KYC-approved set (the Merkle tree of registered identities) but
the verifier learns nothing about which specific address it is. The
`public_inputs` contain only the Merkle root and a nullifier hash -- not the
address itself.

If the address is not in the approved set, the endpoint returns HTTP 404.

---

## Step 8: Execute a Transfer via the Frontend

**What to do:**

1. Open [http://localhost:3000/transfer](http://localhost:3000/transfer)
2. Fill in the form:
   - **Sender DID:** `did:kaspa:alice`
   - **Receiver DID:** `did:kaspa:bob`
   - **Receiver Address:** (a `kaspatest:...` address)
   - **Asset ID:** `KPROP-NYC-TEST`
   - **Amount:** `10`
3. Click **Evaluate Compliance** to check rules first
4. If compliant, click **Send Transfer** to broadcast on-chain

**What to expect:**

- The compliance evaluation panel shows a green checkmark with
  `allowed: true` and `rules_evaluated: 3`
- After transfer, you see a transaction hash linking to the Kaspa Testnet-12
  block explorer at `https://explorer-tn12.kaspa.org/txs/{txId}`

**Why this matters:** The frontend demonstrates the full user flow: compliance
evaluation happens before the wallet signs anything. If the transfer would
violate any rule, the UI blocks the transaction and shows the violations.

---

## Step 9: View Deployed Contracts on the Dashboard

**What to do:**

Open [http://localhost:3000](http://localhost:3000)

**What to expect:**

The dashboard home page shows:
- **Network status** -- live Kaspa TN12 connection, DAA score, block count
- **Deployed SilverScript contracts** with on-chain transaction hashes:

| Contract | P2SH Address | Entrypoints |
|----------|-------------|-------------|
| Clawback | `kaspatest:ppztfhpz...` | `ownerSpend`, `issuerClawback` |
| RwaCore | `kaspatest:prhl2h3v...` | `zkTransfer`, `adminUpdate` |
| StateVerity | `kaspatest:pq6xyf8f...` | `updateState`, `managerReclaim` |
| ZkKycVerifier | `kaspatest:pzhqgz42...` | `verifyProof`, `updateVerifierKey` |
| Reserves | `kaspatest:prlsah5j...` | `withdraw`, `deposit`, `custodianReclaim` |
| HTLC | `kaspatest:prrz0mrx...` | `claimWithPreimage`, `refundAfterTimeout` |
| Dividend | `kaspatest:prrf9w05...` | `claimDividend`, `issuerTopUp`, `issuerReclaim` |

Each contract links to its deployment transaction on the Kaspa Testnet-12
explorer.

**Why this matters:** These are real SilverScript covenant contracts deployed to
live P2SH addresses on Testnet-12. Each transaction hash can be independently
verified on the block explorer.

---

## Step 10: View Reserves and Oracle Status

**What to do:**

Open [http://localhost:3000/reserves](http://localhost:3000/reserves)

**What to expect:**

- **Collateral ratio chart** showing historical ratio (currently 105-118%)
- **Reserve breakdown** pie chart:
  - Real Estate (appraised): $750,000
  - Cash Escrow (USD): $150,000
  - Insurance Bond: $80,000
  - KAS Collateral: $20,000
- **Attestation history** from the oracle pool
- **Reserves contract** details pointing to the on-chain P2SH address
  `kaspatest:prlsah5j...` with `withdraw`, `deposit`, and `custodianReclaim`
  entrypoints

**Why this matters:** Proves that the platform tracks off-chain reserve data and
presents it alongside on-chain contract state. The oracle attestation history
shows when reserves were last verified.

---

## Step 11: Issue a W3C Verifiable Credential

Issue a standards-compliant W3C Verifiable Credential that wraps a KYC claim.
This is the interoperable format that external systems can verify.

**What to do:**

```bash
curl -s -X POST http://localhost:3001/vc/issue \
  -H "Content-Type: application/json" \
  -d '{
    "subject_did": "did:kaspa:alice",
    "claim_type": "KycVerified",
    "expiry": 0
  }' | jq
```

**What to expect:**

```json
{
  "verifiable_credential": {
    "@context": [
      "https://www.w3.org/2018/credentials/v1"
    ],
    "type": [
      "VerifiableCredential",
      "KycVerified"
    ],
    "issuer": "did:kaspa:assetmint-issuer",
    "issuance_date": "2026-03-19T12:00:00Z",
    "credential_subject": {
      "id": "did:kaspa:alice",
      "claim_type": "KycVerified"
    },
    "proof": {
      "type": "Ed25519Signature2020",
      "created": "2026-03-19T12:00:00Z",
      "verification_method": "did:kaspa:assetmint-issuer#key-1",
      "proof_value": "a3c1f9...hex-encoded-ed25519-signature..."
    }
  }
}
```

**Why this matters:** The VC follows the W3C Verifiable Credentials Data Model.
It includes a cryptographic `proof` block with an Ed25519 signature. Any
external system can verify this credential using the issuer's public key by
calling `POST /vc/verify` with the full credential in the request body. The
`proof_value` is a real Ed25519 signature -- not a placeholder.

### Bonus: Verify the VC

```bash
# Take the full verifiable_credential object from the previous response
# and pass it to /vc/verify:

curl -s -X POST http://localhost:3001/vc/verify \
  -H "Content-Type: application/json" \
  -d '{
    "verifiable_credential": {
      "@context": ["https://www.w3.org/2018/credentials/v1"],
      "type": ["VerifiableCredential", "KycVerified"],
      "issuer": "did:kaspa:assetmint-issuer",
      "issuance_date": "2026-03-19T12:00:00Z",
      "credential_subject": {
        "id": "did:kaspa:alice",
        "claim_type": "KycVerified"
      },
      "proof": {
        "type": "Ed25519Signature2020",
        "created": "2026-03-19T12:00:00Z",
        "verification_method": "did:kaspa:assetmint-issuer#key-1",
        "proof_value": "USE_THE_ACTUAL_PROOF_VALUE_FROM_STEP_11"
      }
    }
  }' | jq
```

**Expected response:**

```json
{
  "valid": true,
  "subject_did": "did:kaspa:alice",
  "claim_type": "KycVerified"
}
```

---

## What's Real vs Demo

An honest breakdown of what is live on-chain versus simulated in this demo.

### Real (On-Chain / Cryptographic)

| Component | What's real |
|-----------|-------------|
| **Kaspa TN12 connection** | Live gRPC to `kaspad`, real `block_count`, `daa_score`, `difficulty` |
| **Deployed contracts** | 7 SilverScript covenants at real P2SH addresses with verifiable deployment TXs |
| **Balance queries** | Live UTXO lookups against the Kaspa node |
| **ZK proofs** | Real Groth16 proofs via `ark-groth16` over BN254; real Merkle tree construction |
| **Ed25519 signatures** | Claim and VC signatures use real Ed25519 key pairs |
| **Compliance-gated transfers** | When `kaspa_connected: true`, transfers build real TXs, sign with the sender's key, and broadcast to TN12 |
| **Audit hash commits** | SHA-256 hashes of compliance decisions committed as on-chain transactions |
| **W3C Verifiable Credentials** | Structurally compliant with the W3C VC Data Model; signatures are cryptographically valid |

### Simulated / Demo Data

| Component | What's simulated |
|-----------|-----------------|
| **Identity registry** | In-memory (`IdentityRegistry::in_memory()`); identities do not persist across restarts |
| **KYC verification** | Claims are self-issued by the platform's demo issuer key; no real KYC provider integration |
| **Reserve data** | The collateral ratio chart and reserve breakdown on `/reserves` use hardcoded sample data, not live oracle feeds |
| **Attestation history** | Oracle attestation entries on the reserves page are static demo fixtures |
| **Transfer amounts** | Asset IDs like `KPROP-NYC-TEST` are demo identifiers; no real tokenized property exists |
| **AML screening** | The `AmlClear` claim type exists but there is no integration with an AML data provider |
| **ZK trusted setup** | Uses a fresh ceremony on every startup (`/tmp/assetmint_compliance_keys`); a production deployment requires a multi-party ceremony |
| **Private keys in API** | The `POST /transfer` endpoint accepts raw private keys in the request body; production would use a secure enclave or hardware wallet |

### Partially Real

| Component | Status |
|-----------|--------|
| **Compliance engine** | Rule evaluation logic is real and deterministic, but the rule set is a demo subset (KYC check, transfer limits, jurisdiction) |
| **Merkle tree** | Real SHA-256 Merkle tree built from registered addresses, but the tree is rebuilt from scratch on each query since the registry is in-memory |
| **Oracle pool** | The oracle service code exists and runs on port 3002, but attestations are not yet anchored to on-chain state in the demo flow |

---

## Quick Reference: All API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Service health + Kaspa connectivity |
| `GET` | `/network` | Live Kaspa TN12 chain info |
| `GET` | `/balance?address=kaspatest:...` | UTXO balance for an address |
| `GET` | `/merkle-root` | Current Merkle root of approved addresses |
| `GET` | `/compliance/evaluate?sender_did=...&receiver_did=...&asset_id=...&amount=...` | Evaluate transfer compliance |
| `GET` | `/zk-proof/{address}` | Generate Groth16 ZK-KYC proof |
| `POST` | `/identity` | Register a new DID |
| `POST` | `/claim` | Issue a signed compliance claim |
| `POST` | `/transfer` | Compliance-gated on-chain transfer |
| `POST` | `/audit/commit` | Commit audit hash to Kaspa DAG |
| `POST` | `/vc/issue` | Issue a W3C Verifiable Credential |
| `POST` | `/vc/verify` | Verify a W3C Verifiable Credential |

---

## Step 12: Publish Asset to Sovereign Metadata

The sovereign metadata service replaces the OriginTrail DKG Edge Node with a
self-hosted, private-by-default metadata store. All data stays on your
infrastructure.

**What to do:**

```bash
curl -s -X POST http://localhost:8900/publish \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Asset","ticker":"KTEST","type":"real-estate","jurisdiction":"US"}' | jq
```

**What to expect:**

```json
{
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "metadata_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "status": "published",
  "private": true,
  "verify_instruction": "Commit metadata_hash on-chain via POST /audit/commit to make it verifiable on Kaspa DAG"
}
```

**Why this matters:** The metadata is stored locally with a SHA-256 integrity
hash. The UAL (`did:assetmint:sovereign/...`) is a unique identifier for the
asset. The `metadata_hash` can be committed to the Kaspa DAG via
`POST /audit/commit` to create a tamper-evident on-chain anchor.

---

## Step 13: Verify Metadata Integrity

Check that metadata has not been tampered with by recomputing its SHA-256 hash
and comparing against the stored hash.

**What to do:**

```bash
curl -s -X POST http://localhost:8900/verify \
  -H "Content-Type: application/json" \
  -d '{
    "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
    "metadata": {"name":"Test Asset","ticker":"KTEST","type":"real-estate","jurisdiction":"US"}
  }' | jq
```

**What to expect:**

```json
{
  "verified": true,
  "ual": "did:assetmint:sovereign/a1b2c3d4e5f67890",
  "stored_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "computed_hash": "f0e1d2c3b4a5968778695a4b3c2d1e0f...",
  "tampered": false
}
```

**Why this matters:** If the metadata has been modified since publication,
`verified` will be `false` and `tampered` will be `true`. This provides
tamper detection without requiring on-chain storage of the full metadata --
only the hash needs to be anchored on Kaspa.

---

## Step 13b: Publish Metadata with Hash Commitment

Publish metadata and automatically commit the SHA-256 hash to the Kaspa DAG in
a single call. This combines `POST /publish` and `POST /audit/commit` into one
atomic operation.

**What to do:**

```bash
curl -s -X POST http://localhost:8900/metadata/publish-and-commit \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Asset",
    "ticker": "KTEST",
    "type": "real-estate",
    "jurisdiction": "US",
    "from_address": "kaspatest:qq...",
    "private_key": "hex-encoded-32-bytes"
  }' | jq
```

**What to expect:**

A JSON response containing the UAL, metadata hash, and the Kaspa transaction ID
anchoring the hash on-chain.

**Why this matters:** Unlike the two-step flow (publish then manually call
`/audit/commit`), this endpoint ensures the metadata hash is committed to the
Kaspa DAG atomically. The on-chain TX ID provides a tamper-evident anchor that
anyone can verify independently.

---

## Step 14: Fetch Oracle Attestation

Get a live attested price with 2-of-3 Ed25519 multisig from the oracle pool.

**What to do:**

```bash
curl -s "http://localhost:3001/oracle/attestation?asset_id=KAS" | jq
```

**What to expect:**

A JSON response containing the aggregated KAS price, attestation timestamp,
and Ed25519 multisig signatures from the oracle pool.

**Why this matters:** The oracle endpoint aggregates one live CoinGecko price
with two simulated sources, then creates a 2-of-3 Ed25519 multisig attestation.
This is exposed as a real API endpoint but attestations are not yet committed
on-chain via `state-verity.sil`.

---

## Stopping the Demo

```bash
make stop
```

This kills both the Rust backend and the Next.js dev server.
