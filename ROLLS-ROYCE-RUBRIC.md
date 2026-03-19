<!-- DISCLAIMER: Technical demo code — legal wrappers required in production -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint Completion Rubric -- Honest Status

> Checklist tracking all deliverables across 5 milestones.
> Status markers: `[x]` = genuinely complete, `[~]` = partially done, `[ ]` = not done.

## Overall Status: PARTIALLY COMPLETE (7.3/10)

| Milestone | Status | Lib Tests | Description |
|-----------|--------|-----------|-------------|
| M0 | Complete | -- | Scaffold, vendor repos, workspace setup |
| M1 | Mostly complete | 7 | SilverScript contracts compiled + deployed; ZK circuits working |
| M2 | Complete | 33 | Compliance engine, identity, claims, API |
| M3 | Partially complete | 51 | Token format only; oracle partially real; sync empty loop |
| M4 | Partially complete | -- | Frontend exists, mint page mostly simulated |
| M5 | Partially complete | 5 | Some security tests; live TN12 deploys real; formal verification missing |

**Total lib tests: 96** (verified via `cargo test --lib`)
**On-chain TN12 transactions: 12** (3 transfers + 2 funding + 7 contract deployments)
**SilverScript contracts: 7 written, 7 deployed on TN12**

---

## M0: Infrastructure Scaffold

- [x] Cargo workspace with 6 crates
- [x] Vendor repos cloned (silverscript, kaspa-wasm, etc.)
- [x] SilverScript compiler built (`vendor/silverscript/target/release/silverc`)
- [~] DKG Edge Node config -- Docker Compose exists, node never started
- [x] Git branches created

## M1: SilverScript Contracts + ZK Circuits

### SilverScript Contracts
- [x] `rwa-core.sil` -- ZK-KYC transfer guard (395 bytes) -- deployed TX `d7ed4958...`
- [x] `clawback.sil` -- Issuer clawback (161 bytes) -- deployed TX `6080b477...`
- [x] `state-verity.sil` -- Oracle attestation + state (316 bytes) -- deployed TX `94c50753...`
- [x] `zkkyc-verifier.sil` -- ZK proof verification stub (396 bytes) -- deployed TX `c29499ad...`
- [x] `reserves.sil` -- Proof-of-reserves (372 bytes) -- deployed TX `346fdbd3...`
- [x] `htlc.sil` -- HTLC cross-chain swap (195 bytes) -- deployed TX `1347b397...`
- [x] `dividend.sil` -- Dividend distribution (406 bytes) -- deployed TX `6ec163e1...`
- [x] Constructor args JSON files with real wallet key hashes (blake2b)
- [x] All 5 core contracts compiled with `silverc` compiler
- [x] All 7 contracts deployed as funded P2SH UTXOs on Kaspa TN12
- [ ] Any covenant entrypoint invoked on-chain (no `zkTransfer`, `issuerClawback`, etc. ever called)

### ZK Circuits
- [x] Groth16 R1CS circuit (`zk-circuits/src/kyc_circuit.rs`) -- KycCircuit works
- [x] Trusted setup (`zk-circuits/src/setup.rs`) -- generates pk/vk
- [~] RecursiveKycCircuit -- circuit compiles and tests pass, but uses boolean witness (`previous_proof_valid: Option<bool>`) instead of in-circuit proof verification. This is a demonstration pattern, not real recursive ZK.
- [x] 7 unit tests passing (4 base circuit + 3 recursive)

## M2: Compliance Engine (Polymesh Pattern Reimplementation)

Note: This is NOT a port using `polymesh-api` crate. That crate is never imported. The compliance patterns were reimplemented independently.

- [x] `identity.rs` -- SQLite-backed identity registry
- [x] `claims.rs` -- Ed25519-signed claims with expiry
- [x] `rules.rs` -- Composable engine (RequirementGroup AND/OR)
- [x] `merkle.rs` -- SHA-256 Merkle tree for approved addresses
- [x] `zk_prover.rs` -- Groth16 proof generation (real, ~50ms)
- [x] `zk_verifier.rs` -- Groth16 proof verification (real, ~5ms)
- [x] `api.rs` -- Axum REST API with 8+ endpoints
- [x] Multi-jurisdiction compliance profiles (Reg D, Reg S, MiCA, MAS, Rule 144)
- [x] W3C Verifiable Credentials (`POST /vc/issue`, `POST /vc/verify`)
- [x] On-chain audit trail (`POST /audit/commit`)
- [x] 33 unit tests passing

## M3: ASTM Token + Oracle + Sync

### Tokenomics (format only -- not deployed)
- [x] `tokenomics/src/token.rs` -- KRC-20 inscription JSON format (7 tests)
- [x] `tokenomics/src/staking.rs` -- Time-locked staking with APY tiers (7 tests)
- [x] `tokenomics/src/governance.rs` -- Proposals + stake-weighted voting (7 tests)
- [x] `tokenomics/src/fee_model.rs` -- 0.5% fee with distribution splits (7 tests)
- [ ] ASTM token deployed on Kaspa -- OP_RETURN rejected, needs Kasplex protocol
- [ ] Staking connected to on-chain covenants -- pure in-memory state machine
- [ ] Governance recorded on-chain -- pure in-memory state machine
- [ ] Fee collection wired to transfers -- not connected

### Oracle
- [x] `oracle-pool/src/oracle.rs` -- Simulated price feeds + aggregation
- [x] `oracle-pool/src/attestation.rs` -- Ed25519 multisig (2-of-3)
- [x] `fetch_coingecko_price()` -- real HTTP request to CoinGecko API
- [x] `get_live_aggregated_price()` -- combines 1 live + 2 simulated sources
- [ ] On-chain attestation committed via `state-verity.sil` -- never done
- [x] 12 unit tests passing

### State Sync
- [x] `sync/src/state_sync.rs` -- State transition state machine
- [x] `check_and_transition()` -- correctly detects DKG/oracle/compliance changes
- [ ] `run()` loop -- empty. Logs "Polling DKG..." and sleeps. No HTTP request made. (lines 215-226)
- [x] `run_polling()` -- genuinely functional compliance sync loop (lines 233-298). Polls `/merkle-root` via `reqwest::Client`, detects root changes, triggers `check_and_transition()`. Handles API errors with retry.
- [~] `run_polling()` is not yet wired into application startup -- nobody calls it
- [x] 9 unit tests passing, including `test_merkle_root_polling_transition` and `test_no_state_set_errors`

## M4: Frontend + E2E

- [x] Next.js 15 dashboard (`apps/dashboard-fe/`)
- [x] 8 pages: Dashboard, Mint, Transfer, Clawback, Assets, Reserves, ASTM, Settings
- [x] Regulatory disclaimer banner
- [x] Zustand wallet store + API client
- [x] No Hedera/EVM dependencies
- [x] Transfer page calls real compliance API (`api.evaluateTransfer()`, `api.complianceTransfer()`)
- [~] Transfer page has mock fallback when API is offline (line 59: mock result returned)
- [~] Mint page: Step 2 (DKG) simulated, Step 3 (ZK) real API call, Step 4 (Covenant) shows pre-deployed contracts, Step 5 (KRC-20) preview only
- [x] E2E integration test (`tests/e2e_cycle.rs`) -- 1 test, 8-step cycle

## M5: Security + Verification + Live Deployment + Docs

### Security Testing
- [x] Property-based testing with proptest -- 8 tests in `tests/proptest_compliance.rs`
- [x] Security audit report (`docs/SECURITY-AUDIT.md`) -- STRIDE threat model doc
- [x] Criterion benchmarks (`benches/compliance_bench.rs`)
- [~] Formal verification -- property specs for all 7 contracts (`security/formal-specs/covenant-properties.md`, 18KB) + STRIDE threat model with 12 concrete threats (`security/audit-reports/security-audit.md`, 17KB) exist with real code line references. Not TLA+/Coq/model checking.

### Live Deployment
- [x] Real on-chain TN12 transactions: 12 confirmed (3 transfers + 2 wallet funding + 7 contract deploys)
- [x] All 7 SilverScript contracts deployed on TN12 with TX hashes
- [x] Mempool-aware UTXO selection (filters mempool-spent outpoints)
- [x] Storage mass limit protection (MAX_INPUTS=84)
- [x] `kaspa-adapter/src/tx_builder.rs` -- full UTXO transaction construction
- [x] `kaspa-adapter/src/script.rs` -- silverc JSON loading + P2SH derivation
- [x] `kaspa-adapter/src/client.rs` -- Schnorr sign + broadcast + deploy_contract

### Features with Honest Status
- [~] Threshold Schnorr institutional custody -- DEMO ONLY. Uses XOR key aggregation (`wallet.rs` line 275) and XOR signature combination (line 406). Not real MuSig2. 5 tests pass but test the XOR scheme.
- [~] Recursive ZK compliance history -- DEMO ONLY. Boolean witness pattern, not in-circuit verification. See `kyc_circuit.rs` line 209.
- [~] Live CoinGecko oracle -- `fetch_coingecko_price()` is real, but result is mixed with 2 simulated sources. No on-chain attestation.
- [ ] ASTM KRC-20 inscription broadcast -- `deploy_astm.rs` exists but OP_RETURN rejected by Kaspa
- [~] DKG Edge Node -- startup script + TypeScript client methods exist, node never started or connected

### Documentation
- [x] Architecture documentation (`docs/ARCHITECTURE.md`)
- [x] Security audit doc (`docs/SECURITY-AUDIT.md`)
- [x] This rubric (`ROLLS-ROYCE-RUBRIC.md`)
- [x] Functionality report (`FUNCTIONALITY-REPORT.md`)

---

## Key Technical Decisions

| Decision | Rationale |
|----------|-----------|
| UTXO over Account model | No reentrancy, parallel validation, Kaspa-native |
| Groth16 over Plonk | Smaller proofs, faster verification, ark-groth16 maturity |
| BN254 over BLS12-381 | Smaller field, faster pairing, sufficient for demo |
| SQLite over Postgres | Embedded, zero-config, sufficient for testnet |
| Axum over Actix | Tower ecosystem, async-native, better ergonomics |
| KRC-20 over native token | Kaspa doesn't support custom native tokens |
| Ed25519 for claims, Schnorr for UTXOs | Ed25519 simpler for claim signing; Kaspa uses Schnorr for UTXO signatures |
| XOR for threshold demo | Simpler than full MuSig2; clearly labeled as demo |
| Boolean witness for recursive ZK | Demonstrates the pattern without the complexity of in-circuit verification |

---

## File Tree (key files)

```
ASSETMINT/
├── contracts/silverscript/
│   ├── {rwa-core,clawback,state-verity,zkkyc-verifier,reserves}.sil   # Source
│   ├── {rwa-core,clawback,state-verity,zkkyc-verifier,reserves}.json  # Compiled
│   ├── htlc.sil, dividend.sil                                         # Compiled AND deployed
│   └── *-args.json                                                     # Constructor args
├── packages/kaspa-adapter/src/
│   ├── client.rs        # wRPC + Schnorr sign + broadcast + deploy_contract
│   ├── wallet.rs        # secp256k1 keypair + threshold (XOR demo) + kaspatest: address
│   ├── tx_builder.rs    # UTXO selection + Transaction construction + mass limit
│   └── script.rs        # silverc JSON loading + P2SH address derivation
├── packages/kaspa-adapter/tests/
│   ├── live_transfer.rs   # Real Alice/Bob transfers on TN12
│   ├── deploy_single.rs   # Individual contract deployment tests
│   ├── deploy_contracts.rs # Batch deploy with retry logic
│   └── deploy_astm.rs    # ASTM inscription test (blocked by Kasplex requirement)
├── services/assetmint-core/src/
│   ├── identity.rs      # SQLite DID registry
│   ├── claims.rs        # Ed25519 claims + W3C VCs
│   ├── rules.rs         # Composable compliance rules
│   ├── merkle.rs        # SHA-256 Merkle tree
│   ├── zk_prover.rs     # Groth16 proof generation (real + recursive demo)
│   ├── zk_verifier.rs   # Groth16 proof verification
│   └── api.rs           # Axum REST API
├── services/oracle-pool/src/
│   ├── oracle.rs        # Price feeds (1 real CoinGecko + 2 simulated)
│   └── attestation.rs   # Ed25519 multisig (2-of-3)
├── services/sync/src/
│   └── state_sync.rs    # State machine works; run() loop is empty
├── tokenomics/src/
│   ├── token.rs         # KRC-20 inscription JSON (format only, cannot broadcast)
│   ├── staking.rs       # Staking math (in-memory only, not on-chain)
│   ├── governance.rs    # Governance voting (in-memory only, not on-chain)
│   └── fee_model.rs     # Fee calculation (not wired to transfers)
├── zk-circuits/src/
│   ├── kyc_circuit.rs   # KycCircuit (real) + RecursiveKycCircuit (demo)
│   └── setup.rs         # Trusted setup (deterministic seed for testnet)
├── apps/dashboard-fe/src/
│   ├── app/transfer/    # Real API calls to compliance backend
│   ├── app/mint/        # Steps 2,5 simulated; step 3 real API; step 4 shows deployed contracts
│   └── lib/api.ts       # API client with mock fallbacks
├── docs/
│   ├── ARCHITECTURE.md
│   └── SECURITY-AUDIT.md
├── FUNCTIONALITY-REPORT.md   # Honest assessment (7.3/10)
└── ROLLS-ROYCE-RUBRIC.md     # This file
```
