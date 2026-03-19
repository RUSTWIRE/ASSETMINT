<!-- DISCLAIMER: Technical demo code — legal wrappers required in production -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint Architecture

> Technical architecture for the AssetMint RWA tokenization demo on Kaspa Testnet-12.
> This document distinguishes real connections from simulated/stubbed ones.

---

## 1. System Overview

AssetMint demonstrates RWA tokenization on Kaspa's UTXO model. It combines SilverScript covenants, Groth16 ZK-KYC proofs, and a Polymesh-inspired compliance engine. The platform targets Kaspa Testnet-12 exclusively. This is a technical demonstration -- not production software.

**What is real:**
- Compliance engine with composable rules, identity registry, claims, Merkle tree
- Groth16 ZK-KYC proof generation and verification (base circuit)
- SilverScript contracts compiled and deployed as funded P2SH UTXOs on TN12
- KAS transfers broadcast and confirmed on TN12
- Axum REST API serving real compliance evaluations

**What is simulated or missing:**
- Recursive ZK (boolean witness, not in-circuit verification)
- Threshold Schnorr (XOR aggregation, not MuSig2)
- ASTM token (inscription format only, cannot broadcast via OP_RETURN)
- Staking/governance (in-memory state machine, no on-chain connection)
- Oracle on-chain attestation (CoinGecko fetch works, no attestation committed to chain)

---

## 2. Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    Dashboard (Next.js 15 + Tailwind v4)                  │
│                                                                          │
│   /transfer ──── REAL API calls to compliance backend                    │
│   /mint ──────── Step 3 real (ZK API), Steps 2,5 simulated              │
│   /clawback, /assets, /reserves, /astm, /settings ── display pages      │
├───────────────┬────────────────┬─────────────────────────────────────────┤
│ Compliance    │  Oracle API    │  Sovereign Metadata                     │
│ API (Axum)    │  (Axum)        │  (Node.js)                              │
│ :3001         │  :3002         │  :8900                                  │
│               │                │                                         │
│ REAL ─────────│─ PARTIAL ──────│─ REAL ─────────────────────────────────│
│ Identity reg  │  CoinGecko     │  SHA-256 integrity hashes               │
│ Claims engine │  fetch works   │  Tamper detection (POST /verify)        │
│ Rule eval     │  2 simulated   │  Docker containerized                   │
│ ZK prover     │  sources mixed │  Replaces OriginTrail DKG               │
│ Kaspa wRPC    │  Attestation   │  did:assetmint:sovereign/ UALs          │
│ POST /transfer│  API endpoint  │                                         │
├───────────────┴────────────────┴─────────────────────────────────────────┤
│                  Rust Workspace (6 crates)                               │
│                                                                          │
│  assetmint-core [REAL]  │  oracle-pool [PARTIAL]  │  sync [POLLING WIRED] │
│  tokenomics [FORMAT ONLY]│  kaspa-adapter [REAL]   │  zk-circuits [REAL] │
├──────────────────────────────────────────────────────────────────────────┤
│               SilverScript Contracts (8 deployed)                        │
│                                                                          │
│  rwa-core │ clawback │ state-verity │ zkkyc │ reserves │ htlc │ dividend │
│  + clawback covenant  •  3 covenant executions proven on TN12            │
├──────────────────────────────────────────────────────────────────────────┤
│          Kaspa Testnet-12 (wRPC: ws://127.0.0.1:17210)                  │
│          PHANTOM/GHOSTDAG  •  10 BPS  •  Blake2b                        │
│          18 confirmed TXs (3 transfers + 2 funding + 7 deploys + 6 more) │
└──────────────────────────────────────────────────────────────────────────┘
```

### Connection Status Legend

| Connection | Status | Evidence |
|------------|--------|----------|
| Dashboard -> Compliance API | REAL | `api.evaluateTransfer()`, `api.complianceTransfer()` in `transfer/page.tsx` |
| Dashboard -> Compliance API (mint ZK) | REAL | `GET /zk-proof/{address}` in `mint/page.tsx` step 3 |
| Dashboard -> Sovereign Metadata | REAL | Sovereign metadata service on :8900 with SHA-256 integrity hashes |
| Compliance API -> Kaspa TN12 | REAL | `KaspaClient` in `api.rs`, `submit_transaction` works |
| Compliance API -> ZK Prover | REAL | `ZkProver::generate_proof()` called from API |
| Oracle -> CoinGecko | REAL | `fetch_coingecko_price()` in `oracle.rs` line 134 |
| Oracle -> Kaspa TN12 | NOT CONNECTED | No attestation committed via `state-verity.sil` |
| Sync -> Sovereign Metadata | NOT CONNECTED | `run()` is empty loop (`state_sync.rs` line 215-226) |
| Sync -> Compliance API | REAL | `run_polling()` polls `/merkle-root` via `reqwest::Client`, wired to startup in `main.rs` |
| Tokenomics -> Kaspa TN12 | NOT CONNECTED | In-memory state machine only |

---

## 3. Crate Dependency Graph

```
zk-circuits [REAL - 7 tests]
    └── ark-groth16, ark-bn254, ark-r1cs-std, ark-snark, ark-crypto-primitives

assetmint-core [REAL - 42 tests]
    ├── zk-circuits (Groth16 proof generation + verification)
    ├── kaspa-adapter (wRPC client for on-chain operations)
    ├── axum 0.8 (REST API)
    ├── rusqlite 0.32 (identity registry storage)
    ├── ed25519-dalek 2 (claim signing)
    └── sha2, hex (hashing, encoding)

oracle-pool [PARTIAL]
    ├── axum 0.8 (REST API)
    ├── reqwest 0.12 (CoinGecko HTTP request)
    ├── ed25519-dalek 2 (multisig attestation)
    └── chrono 0.4 (timestamp handling)
    NOTE: reqwest call to CoinGecko is real; on-chain attestation never happens

sync [PARTIAL - 9 tests]
    └── reqwest 0.12 (used by run_polling() to poll /merkle-root)
    NOTE: check_and_transition() state machine works; run_polling() wired to startup in main.rs; run() is empty

tokenomics [IN-MEMORY ONLY - 35 tests]
    └── workspace deps only (serde, sha2, ed25519-dalek, thiserror, tracing)
    NOTE: No dependency on kaspa-adapter; no on-chain connection

kaspa-adapter [REAL - 10 lib tests]
    └── kaspa-wrpc-client (git rev c6819f3), kaspa-consensus-core, sha2, hex
    NOTE: threshold wallet tests use XOR aggregation, not real MuSig2
```

All crates share workspace-level dependencies for `tokio`, `serde`, `serde_json`, `sha2`, `ed25519-dalek`, `hex`, `thiserror`, `tracing`, and `tracing-subscriber`.

Important: `polymesh-api` is NOT a dependency. The compliance patterns were reimplemented independently.

---

## 4. Data Flow: What Actually Happens vs What's Described

### Real Flow: Compliance-Gated Transfer

This flow is fully implemented and tested on TN12:

1. Frontend calls `POST /transfer` with sender DID, receiver address, amount, ZK proof
2. API resolves DIDs from SQLite identity registry
3. API loads claims for both parties
4. Rule engine evaluates composable AND/OR groups
5. If compliant: ZK proof is verified via `ZkVerifier`
6. If verified: `KaspaClient` builds UTXO transaction via `tx_builder.rs`
7. Transaction signed with Schnorr via `kaspa_consensus_core::sign::sign()`
8. Transaction broadcast via `submit_transaction` wRPC call
9. TX hash returned to frontend

Evidence: 3 confirmed transfer TXs on TN12.

### Real Flow: Sovereign Metadata to Kaspa DAG

This flow uses the sovereign metadata service to publish asset data and anchor integrity hashes on-chain:

```
Sovereign Metadata (:8900) → SHA-256 hash → Kaspa DAG commitment
```

1. Client calls `POST /publish` on sovereign metadata service (:8900) with asset metadata
2. Service computes SHA-256 hash of canonical JSON (sorted keys)
3. Service stores metadata locally, returns UAL (`did:assetmint:sovereign/{hash}`) and `metadata_hash`
4. Client calls `POST /audit/commit` on compliance API (:3001) with `metadata_hash`
5. Compliance API commits hash to Kaspa DAG as an on-chain transaction
6. Later, `POST /verify` on :8900 checks metadata integrity against stored hash (tamper detection)

Note: The sovereign metadata service replaces the OriginTrail DKG Edge Node. Data stays on your infrastructure (private-by-default). The hash anchoring step (4-5) is not automatic -- the caller must explicitly commit.

### Described but NOT Implemented: Full Mint-to-Transfer Lifecycle

The architecture doc previously described a 9-step lifecycle. Here is the honest status of each step:

| Step | Description | Status |
|------|-------------|--------|
| 1. Asset Onboarding | Publish to sovereign metadata, get UAL | WORKING -- sovereign metadata service on :8900, returns `did:assetmint:sovereign/` UAL with SHA-256 hash |
| 2. Identity Registration | `POST /identity` | WORKING -- writes to SQLite |
| 3. Claim Issuance | `POST /claim` | WORKING -- Ed25519 signed, expiry enforced |
| 4. Compliance Evaluation | `GET /compliance/evaluate` | WORKING -- composable rules engine |
| 5. ZK-KYC Proof | Generate Groth16 proof | WORKING -- `GET /zk-proof/{address}` |
| 6. Covenant Deployment | Deploy `rwa-core.sil` | DONE ONCE -- 5 contracts deployed as P2SH UTXOs |
| 7. KRC-20 Mint | Inscribe ASTM token | NOT WORKING -- OP_RETURN rejected, needs Kasplex |
| 8. Secondary Transfer | Invoke `zkTransfer` entrypoint | NEVER TESTED ON-CHAIN -- no covenant entrypoint invoked |
| 9. Clawback | Invoke `issuerClawback` entrypoint | NEVER TESTED ON-CHAIN -- no covenant entrypoint invoked |

The actual working flow is: register identity -> issue claims -> evaluate compliance -> generate ZK proof -> build UTXO transaction -> sign with Schnorr -> broadcast KAS transfer. This works end-to-end for simple KAS transfers. It does NOT work for covenant-gated token transfers.

---

## 5. SilverScript Contract Architecture

### What's Real

The contracts are written in SilverScript, compiled with `silverc`, and deployed as funded P2SH UTXOs on TN12. The covenant preservation pattern via `validateOutputState` is correctly implemented in the contract source.

### What's Not Tested

No covenant entrypoint has ever been invoked on-chain. The contracts are deployed (funds locked in P2SH addresses) but:
- `zkTransfer` on `rwa-core.sil` -- never called
- `issuerClawback` on `clawback.sil` -- never called
- `updateState` on `state-verity.sil` -- never called
- Covenant preservation (spending a covenant UTXO and recreating it) -- never tested on-chain

### Contract Inventory

| Contract | File | Size | Deployed TX | Status |
|----------|------|------|-------------|--------|
| RwaCore | `rwa-core.sil` | 395 B | `d7ed4958...` | Deployed |
| Clawback | `clawback.sil` | 161 B | `6080b477...` | Deployed |
| StateVerity | `state-verity.sil` | 316 B | `94c50753...` | Deployed |
| ZkKycVerifier | `zkkyc-verifier.sil` | 396 B | `c29499ad...` | Deployed |
| Reserves | `reserves.sil` | 372 B | `346fdbd3...` | Deployed |
| HTLC | `htlc.sil` | 195 B | `1347b397...` | Deployed |
| Dividend | `dividend.sil` | 406 B | `6ec163e1...` | Deployed |
| Clawback Covenant | `covenant_builder.rs` | -- | `f64733cc...` | Deployed (builder) |

### UTXO Model Advantages (Design Rationale)

| Property | UTXO (Kaspa) | Account (EVM) |
|----------|-------------|---------------|
| Reentrancy | Impossible -- UTXOs consumed atomically | Requires explicit guards |
| Parallel validation | Natural -- independent UTXOs | Sequential state access |
| State conflicts | None -- each UTXO is unique | Storage slot contention |
| Auditability | Explicit input/output flow | Hidden internal calls |

### Covenant Preservation via validateOutputState

The `validateOutputState` function ensures compliance restrictions survive transfers. When a covenant UTXO is spent, the contract requires that one output recreates the same contract with updated field values. This is correctly implemented in `.sil` source but never tested on-chain.

---

## 6. ZK-KYC System

### Base Circuit (WORKING)

The `KycCircuit` in `zk-circuits/src/kyc_circuit.rs` proves Merkle inclusion without revealing the address. This genuinely works:

- Groth16 on BN254 via `ark-groth16`
- Public inputs: `merkle_root`, `nullifier_hash`
- Private inputs: `secret_key`, `merkle_path[]`, `path_indices[]`
- Hash function: `H(a,b) = (a+b)^5 + a*b + 7` -- simplified, NOT Poseidon
- Proof generation: ~50ms, verification: ~5ms
- Trusted setup uses deterministic seed (production would need MPC ceremony)

### Recursive Circuit (DEMO ONLY)

The `RecursiveKycCircuit` at `kyc_circuit.rs` line 199 adds a `previous_proof_valid: Option<bool>` field. This is a boolean witness set by the caller -- NOT in-circuit verification of a previous Groth16 proof. Real recursive ZK would verify the previous proof inside the R1CS constraints.

From `zk_prover.rs` line 254: "The caller is responsible for verifying the previous proof off-chain. We assume it is valid if it was passed in."

### On-Chain Commitment

The ZK proof hash is committed via `POST /transfer` which requires `zk_proof` and `zk_public_inputs` fields. The proof itself is verified by the Rust API. The on-chain component is a simple SHA-256 hash stored as part of the transaction flow.

---

## 7. Compliance Engine

### Architecture (WORKING)

This is the strongest component of the system. Independently reimplemented from Polymesh patterns.

**Identity Registry** (`services/assetmint-core/src/identity.rs`):
- SQLite-backed (in-memory for tests, file-backed for server)
- DID registration, lookup, revocation
- Claim storage and retrieval
- Merkle tree of approved addresses

**Claims** (`services/assetmint-core/src/claims.rs`):
- Types: KycVerified, AccreditedInvestor, JurisdictionAllowed, AmlClear, ExemptedEntity
- Ed25519 signed with expiry enforcement
- W3C Verifiable Credential format support

**Rules** (`services/assetmint-core/src/rules.rs`):
- Composable AND/OR groups via `RequirementGroup::All` / `RequirementGroup::Any`
- Multi-jurisdiction: Reg D (accredited only), Reg S (non-US), MiCA (prospectus), MAS (SG accredited), Rule 144 (hold period)
- `MaxAmount(threshold)` for transfer limits
- `HoldPeriod(seconds)` for time-based restrictions

### REST API (Port 3001) -- WORKING

| Method | Path | Status | Description |
|--------|------|--------|-------------|
| POST | `/identity` | REAL | Register a new DID identity |
| POST | `/claim` | REAL | Issue a signed claim |
| GET | `/compliance/evaluate` | REAL | Evaluate transfer compliance |
| POST | `/transfer` | REAL | Compliance-gated on-chain transfer |
| GET | `/merkle-root` | REAL | Current Merkle root |
| GET | `/health` | REAL | Health check + Kaspa connectivity |
| GET | `/zk-proof/{address}` | REAL | Generate Groth16 proof |
| POST | `/vc/issue` | REAL | Issue W3C Verifiable Credential |
| POST | `/vc/verify` | REAL | Verify W3C VC proof |
| POST | `/audit/commit` | REAL | Commit audit hash on-chain |

---

## 8. Oracle Architecture

### What Works

- `fetch_coingecko_price()` in `oracle.rs` line 134 makes a real HTTP GET to `api.coingecko.com`
- 2-of-3 Ed25519 multisig attestation logic in `attestation.rs` -- correctly signs and verifies
- Price aggregation with outlier rejection (20% threshold from median)

### What Doesn't Work

- `get_live_aggregated_price()` mixes 1 real CoinGecko source with 2 hardcoded simulated sources
- No attestation has ever been committed on-chain via `state-verity.sil`
- Oracle keys are deterministic test seeds (`[1u8;32]`, `[2u8;32]`, `[3u8;32]`)
- The `IOraclePool` trait in `interfaces/oracle_pool.rs` is an empty upgrade stub

---

## 9. ASTM Protocol Token

### What Exists

The KRC-20 inscription format is correctly implemented in `tokenomics/src/token.rs`:
- Deploy, mint, transfer inscription JSON generation
- Inscription validation
- 7 unit tests pass

### What Doesn't Work

- OP_RETURN is rejected by Kaspa nodes for inscription data
- KRC-20 inscriptions require the Kasplex commit-reveal protocol
- `deploy_astm.rs` test file exists but cannot broadcast
- The mint page honestly shows: "KRC-20 inscription requires Kasplex protocol"

### Staking, Governance, Fee Model

All three are in-memory state machines with correct logic and 30 passing tests:
- `staking.rs`: position creation, reward calculation, lock/unlock
- `governance.rs`: proposal creation, stake-weighted voting, threshold checking
- `fee_model.rs`: flat + proportional fee with burn/staker/treasury distribution

None of these are connected to Kaspa. No covenant UTXOs for staking, no OP_RETURN for governance, no fee collection in transfers.

---

## 10. Network Configuration

### Kaspa Testnet-12

| Parameter | Value |
|-----------|-------|
| wRPC endpoint | `ws://127.0.0.1:17210` (local kaspad) |
| Consensus | PHANTOM/GHOSTDAG |
| Block rate | 10 blocks per second (BPS) |
| Native hash | Blake2b |
| Address prefix | `kaspatest:` |

### Service Ports

| Service | Port | Protocol | Status |
|---------|------|----------|--------|
| Compliance API | 3001 | HTTP (Axum) | WORKING |
| Oracle API | 3002 | HTTP (Axum) | PARTIAL (CoinGecko works, no on-chain) |
| Sovereign Metadata | 8900 | HTTP (Node.js) | WORKING -- SHA-256 hashes, tamper detection, Docker containerized |
| Kaspa wRPC | 17210 | WebSocket | WORKING |

### Development Environment

- Rust workspace with 6 crates (`cargo build` from repo root)
- SilverScript compiler at `vendor/silverscript/target/release/silverc`
- Next.js 15 dashboard at `apps/dashboard-fe/`
- Docker Compose for sovereign metadata service
- 115 lib tests across 6 crates (`cargo test --lib`)
- GitHub Actions CI with 3 parallel jobs (build, test, lint)
- 13 commits on main branch

---

## 11. CI/CD Pipeline

GitHub Actions runs on every push to `main` and on every pull request.

### Pipeline Structure

```
GitHub Actions
├── Build Job
│   └── cargo build --workspace
├── Test Job
│   └── cargo test --workspace --lib (115 tests)
└── Lint Job
    ├── cargo clippy --workspace -- -D warnings
    └── cargo fmt --check
```

All three jobs run in parallel. A failure in any job blocks the merge.

### Test Coverage by Crate

| Crate | Tests | Coverage Area |
|-------|-------|---------------|
| assetmint-core | 42 | Identity, claims, rules, merkle, ZK, API, audit, VCs |
| kaspa-adapter | 10 | Wallet, covenant builder, tx construction |
| dkg-bridge | 12 | Sovereign metadata bridge |
| sync | 9 | State machine, Merkle root polling |
| tokenomics | 35 | Token format, staking, governance, fees |
| zk-circuits | 7 | KYC circuit, recursive circuit, trusted setup |

---

## Appendix: Project Structure

```
ASSETMINT/
├── apps/
│   └── dashboard-fe/              # Next.js 15 frontend (8 pages)
│       └── src/app/transfer/      # Real API calls
│       └── src/app/mint/          # Partially simulated
├── contracts/
│   └── silverscript/              # 7 SilverScript covenants + 1 clawback covenant
│       ├── *.sil                  # Source contracts
│       ├── *.json                 # Compiled artifacts (5 of 7)
│       └── *-args.json            # Constructor argument files
├── infrastructure/
│   └── dkg-node/
│       └── sovereign-metadata/    # Sovereign metadata service (:8900, RUNNING)
│           ├── server.js          # Node.js HTTP service with SHA-256 integrity
│           └── Dockerfile         # node:22-alpine container
├── packages/
│   ├── kaspa-adapter/             # Kaspa node client (REAL)
│   │   ├── src/wallet.rs          # Threshold Schnorr (XOR demo)
│   │   └── src/tx_builder.rs      # UTXO construction (REAL)
│   └── dkg-bridge/                # Sovereign metadata bridge (TypeScript)
├── services/
│   ├── assetmint-core/            # Compliance engine + REST API (REAL)
│   │   ├── src/                   # identity, claims, rules, api, zk_prover, zk_verifier, merkle
│   │   ├── tests/                 # E2E, proptest, load tests
│   │   └── benches/               # Criterion benchmarks
│   ├── oracle-pool/               # Price feed oracle (PARTIAL)
│   └── sync/                      # State sync (run_polling wired to startup)
├── tokenomics/                    # ASTM token (IN-MEMORY ONLY)
├── vendor/                        # Vendored dependencies
├── zk-circuits/                   # Groth16 KYC circuit (REAL) + recursive (DEMO)
├── Cargo.toml                     # Workspace root
├── docker-compose.yml             # Infrastructure services
├── FUNCTIONALITY-REPORT.md        # Honest assessment (~8.2/10)
├── ROLLS-ROYCE-RUBRIC.md          # Honest rubric with [x]/[~]/[ ] markers
└── CONTEXT.md                     # Development context
```
