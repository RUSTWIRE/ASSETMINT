# AssetMint Development Context

> This file preserves critical context across sessions. DO NOT DELETE.

## Project Status: ALL 5 MILESTONES COMPLETE

### Completed
- **M0**: Full scaffold, 9 vendor repos cloned, Rust workspace (6 crates) compiles, SilverScript compiler built, DKG Edge Node config, git branches created
- **M1**: 5 SilverScript contracts written & compiled, Groth16 ZK-KYC circuit implemented, trusted setup, prover, verifier all working.
- **M2**: Full Rust Polymesh compliance port — identity registry, Ed25519 claims, composable rules engine, Axum REST API. 23 tests pass.
- **M3**: ASTM token (KRC-20 inscriptions), staking, governance, fee model, simulated multisig oracle with Ed25519 attestations, DKG state-verity sync. 47 tests pass.
- **M4**: Next.js 15 dashboard (8 pages), Kaspa wallet/API layer, E2E integration test. 75 tests pass across workspace.
- **M5**: Property-based testing (8 proptest), security audit, Criterion benchmarks, architecture docs, investor brief, rubric. 83 tests pass.
- **LIVE KASPA**: kaspa-adapter wired to real rusty-kaspa RPC (kaspa-wrpc-client @ c6819f3). Compliance API connects to local kaspad (127.0.0.1:17210) on startup. Live endpoints: `/network`, `/balance`, `/health` (with kaspa_connected). Frontend fetches real block count, DAA score, difficulty from kaspad. **85 tests pass** (incl. 2 live TN12 integration tests).

### Running the Stack
```bash
# Terminal 1: kaspad (already running)
kaspad --testnet --netsuffix=12 --rpclisten-borsh=0.0.0.0:17210 --utxoindex

# Terminal 2: Compliance API (port 3001, auto-connects to kaspad)
cd /Users/rory/ASSETMINT && cargo run -p compliance-rust

# Terminal 3: Frontend dashboard (port 3000)
cd /Users/rory/ASSETMINT/apps/dashboard-fe && npm run dev
```

### See ROLLS-ROYCE-RUBRIC.md for full completion checklist.

## M5 Deliverables

### Property-Based Testing (proptest — 8 tests)
| Test | Property |
|------|----------|
| prop_max_amount_always_enforced | MaxTransferAmount allows iff amount <= max |
| prop_empty_engine_always_allows | Empty engine always allows any transfer |
| prop_kyc_required_denies_without_claim | No claims → always denied |
| prop_kyc_required_allows_with_claim | KYC claim (non-expiring) → always allowed |
| prop_hold_period_monotonic | Once allowed, stays allowed for all later times |
| prop_violations_count_matches_denied_rules | allowed=true ↔ violations empty |
| prop_jurisdiction_block_always_blocks | Blocked jurisdiction → always denied |
| prop_rules_evaluated_equals_rule_count | rules_evaluated == number of rules added |

### Documentation
| Document | Path | Description |
|----------|------|-------------|
| Security Audit | `docs/SECURITY-AUDIT.md` | STRIDE threat model, 2 critical + 4 high findings, 20 remediation items |
| Architecture | `docs/ARCHITECTURE.md` | 10-section system architecture with diagrams |
| Investor Brief | `docs/INVESTOR-BRIEF.md` | 2-page overview: problem, solution, market, tokenomics, roadmap |
| Completion Rubric | `ROLLS-ROYCE-RUBRIC.md` | Full M0-M5 checklist with key decisions |

### Benchmarks
| Benchmark | File |
|-----------|------|
| compliance_evaluation | `benches/compliance_bench.rs` |
| merkle_tree_build | `benches/compliance_bench.rs` |
| merkle_proof_verify | `benches/compliance_bench.rs` |

### Total Workspace Tests: 83
| Crate | Unit | Integration | Proptest |
|-------|------|-------------|---------|
| compliance-rust | 23 | 1 (E2E) | 8 |
| zk-circuits | 4 | — | — |
| tokenomics | 30 | — | — |
| oracle-pool | 10 | — | — |
| sync | 7 | — | — |

## M1 Deliverables

### SilverScript Contracts (all compile with `silverc`)

| Contract | File | Entrypoints | Script Size | Purpose |
|----------|------|------------|-------------|---------|
| RwaCore | `contracts/silverscript/rwa-core.sil` | zkTransfer, adminUpdate | 395 bytes | ZK-KYC transfer guard + covenant preservation |
| Clawback | `contracts/silverscript/clawback.sil` | ownerSpend, issuerClawback | 161 bytes | Issuer clawback with OP_RETURN reason |
| StateVerity | `contracts/silverscript/state-verity.sil` | updateState, managerReclaim | 316 bytes | Oracle attestation + DKG state transition |
| ZkKycVerifier | `contracts/silverscript/zkkyc-verifier.sil` | verifyProof, updateVerifierKey | 396 bytes | On-chain ZK proof verification stub |
| Reserves | `contracts/silverscript/reserves.sil` | withdraw, deposit, custodianReclaim | 372 bytes | Proof-of-reserves with oracle attestation |

Constructor args: `contracts/silverscript/args/*.json` (uses `{"kind":"array","data":[{"kind":"byte","data":N},...]}` format)

### ZK Circuits (all tests pass)

| Component | File | Tests | Description |
|-----------|------|-------|-------------|
| KycCircuit | `zk-circuits/src/kyc_circuit.rs` | 3 pass | MiMC-based Merkle inclusion + nullifier binding |
| Trusted Setup | `zk-circuits/src/setup.rs` | 1 pass | Groth16 key generation (deterministic for testnet) |
| ZkProver | `services/compliance-rust/src/zk_prover.rs` | 1 pass | Full proof generation with ZkWitness |
| ZkVerifier | `services/compliance-rust/src/zk_verifier.rs` | 1 pass | Full proof verification + VK hash |

Key implementation notes:
- Uses simplified MiMC-like hash (not cryptographically secure — demo only)
- `H(a,b) = (a+b)^5 + a*b + 7` — keeps constraint count low while demonstrating the pattern
- Production would use Poseidon hash inside the circuit
- `build_merkle_tree()` and `native_*` helpers in kyc_circuit.rs for witness construction
- Proof generation and verification use `ark-snark::SNARK` trait
- Keys serialized with `ark-serialize` compressed format

### validateOutputState Pattern
Contracts using `validateOutputState()` must:
1. Declare constructor params with `init` prefix (e.g., `byte[32] initMerkleRoot`)
2. Declare contract fields initialized from params (e.g., `byte[32] merkleRoot = initMerkleRoot;`)
3. Include ALL fields in every `validateOutputState()` call

## M2 Deliverables

### Compliance Modules (all tests pass — 23 total)

| Module | File | Tests | Description |
|--------|------|-------|-------------|
| Identity Registry | `services/compliance-rust/src/identity.rs` | 5 pass | SQLite-backed DID registry with claims loading |
| Claims | `services/compliance-rust/src/claims.rs` | 4 pass | Ed25519-signed claims with expiry verification |
| Rules Engine | `services/compliance-rust/src/rules.rs` | 5 pass | Composable AND/OR rules with HoldPeriod |
| REST API | `services/compliance-rust/src/api.rs` | 5 pass | Axum endpoints: identity, claims, evaluate, merkle-root |
| ZK Prover | `services/compliance-rust/src/zk_prover.rs` | 1 pass | Groth16 proof generation |
| ZK Verifier | `services/compliance-rust/src/zk_verifier.rs` | 1 pass | Groth16 proof verification |
| Merkle Tree | `services/compliance-rust/src/merkle.rs` | 2 pass | SHA-256 Merkle tree for approved addresses |

### API Endpoints (port 3001)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/identity` | Register a new DID identity |
| POST | `/claim` | Issue a signed claim (KycVerified, AccreditedInvestor, JurisdictionAllowed, AmlClear, ExemptedEntity) |
| GET | `/compliance/evaluate` | Evaluate transfer compliance (query params: sender_did, receiver_did, asset_id, amount, mint_timestamp) |
| GET | `/merkle-root` | Current Merkle root of approved (non-revoked) addresses |
| GET | `/health` | Health check |

Key implementation details:
- `AppState` holds `IdentityRegistry`, `Mutex<ComplianceEngine>`, and `ClaimIssuer`
- Claims are Ed25519-signed with SHA-256 canonical claim data hashing
- Rule evaluation checks claim expiry: `c.expiry == 0 || c.expiry > now`
- `RequirementGroup::All` = AND logic, `RequirementGroup::Any` = OR logic
- HoldPeriod rule checks `now < mint_timestamp + period`
- Merkle root returns zero hash when no addresses registered

## M3 Deliverables

### Tokenomics (30 tests pass)

| Module | File | Tests | Description |
|--------|------|-------|-------------|
| ASTM Token | `tokenomics/src/token.rs` | 7 pass | KRC-20 inscription builder (deploy/mint/transfer), validation, commitment hashing |
| Staking | `tokenomics/src/staking.rs` | 8 pass | Time-locked covenant staking, 5% APY rewards, min stake/lock enforcement |
| Governance | `tokenomics/src/governance.rs` | 8 pass | Proposal creation, weighted voting, execution threshold, OP_RETURN encoding |
| Fee Model | `tokenomics/src/fee_model.rs` | 7 pass | Flat + proportional fee (capped ≤0.001 KAS), 30/50/20 burn/staker/treasury split |

### Oracle Pool (10 tests pass)

| Module | File | Tests | Description |
|--------|------|-------|-------------|
| Price Feed | `services/oracle-pool/src/oracle.rs` | 5 pass | Simulated 3-source aggregation with outlier rejection (20% threshold) |
| Attestation | `services/oracle-pool/src/attestation.rs` | 5 pass | Ed25519 2-of-3 multisig signing, full signature verification, tamper detection |
| IOraclePool | `services/oracle-pool/src/interfaces/oracle_pool.rs` | — | Upgrade trait stub for future miner-attested oracle |

### State Sync (7 tests pass)

| Module | File | Tests | Description |
|--------|------|-------|-------------|
| State Sync | `services/sync/src/state_sync.rs` | 7 pass | DKG poll + state transitions (DKG/oracle/compliance changes), OP_RETURN encoding |

Key implementation details:
- Oracle attestations use real Ed25519 signatures (not placeholders) — testnet seeds `[1u8;32]`, `[2u8;32]`, `[3u8;32]`
- Tampered attestations correctly fail verification
- State transitions track `ChangeType` (DKG, Oracle, Compliance, Combined)
- KRC-20 inscriptions enforce per-mint limit (1000 ASTM max)
- Fee model: flat 50k sompis + 1 bps proportional, capped at 100k sompis (0.001 KAS)
- Staking rewards: 500 bps (5% APY), min 100 ASTM stake, min 7-day lock

## M4 Deliverables

### Frontend Dashboard (Next.js 15 + Tailwind v4)

| Page | Route | Description |
|------|-------|-------------|
| Dashboard | `/` | Portfolio overview: KAS/ASTM balances, recent transactions, compliance health, staking stats |
| Mint Wizard | `/mint` | 5-step flow: asset details → DKG publish → ZK-KYC proof → covenant deploy → KRC-20 mint |
| Transfer | `/transfer` | ZK-KYC gated transfer with compliance evaluation (calls `/compliance/evaluate`) |
| Clawback | `/clawback` | Issuer admin panel: target address, OP_RETURN reason, history |
| Asset Detail | `/assets` | KPROP-NYC-TEST: DKG metadata, UTXO info, compliance, oracle price |
| Reserves | `/reserves` | Proof-of-reserves: collateral ratio charts, oracle attestation, reserve breakdown |
| ASTM Token | `/astm` | Staking panel, governance proposals with voting, fee model info |
| Settings | `/settings` | API endpoints, wallet info, network (Testnet-12) |

Key components:
- `disclaimer-banner.tsx` — Regulatory disclaimer on every page (amber, dismissible)
- `sidebar.tsx` — Navigation with Lucide icons
- `wallet-button.tsx` — Simulated Kaspa testnet wallet (connect/disconnect)
- `src/lib/api.ts` — Fetch client for compliance-rust (port 3001) and oracle-pool (port 3002)
- `src/store/wallet.ts` — Zustand wallet state store

No Hedera/EVM deps. Pure Kaspa-native frontend.

### E2E Integration Test (1 test)

| Test | File | Steps | Description |
|------|------|-------|-------------|
| Full RWA Cycle | `services/compliance-rust/tests/e2e_cycle.rs` | 8 steps | Register → KYC → transfer (allow/deny) → MaxAmount → Merkle → ZK proof → revoke |

### Total Workspace Tests: 75

| Crate | Tests |
|-------|-------|
| compliance-rust (unit) | 23 |
| compliance-rust (E2E) | 1 |
| zk-circuits | 4 |
| tokenomics | 30 |
| oracle-pool | 10 |
| sync | 7 |

## Architecture Quick Reference

| Component | Location | Language | Status |
|-----------|----------|----------|--------|
| SilverScript contracts | `contracts/silverscript/*.sil` | SilverScript | ✅ M1 complete |
| Kaspa adapter | `packages/kaspa-adapter/` | Rust | Stubs done |
| DKG bridge | `packages/dkg-bridge/` | TypeScript | Stubs done |
| Compliance engine | `services/compliance-rust/` | Rust | ✅ M2 complete (23 tests) |
| Oracle pool | `services/oracle-pool/` | Rust | ✅ M3 complete (10 tests) |
| State sync | `services/sync/` | Rust | ✅ M3 complete (7 tests) |
| Tokenomics | `tokenomics/` | Rust | ✅ M3 complete (30 tests) |
| ZK circuits | `zk-circuits/` | Rust | ✅ M1 complete |
| Frontend | `apps/dashboard-fe/` | Next.js/TS | ✅ M4 complete (8 pages) |
| DKG Edge Node | `infrastructure/dkg-node/` | Docker | Config done |

## Key Paths

| What | Path |
|------|------|
| SilverScript compiler | `vendor/silverscript/target/release/silverc` |
| SilverScript tutorial | `vendor/silverscript/docs/TUTORIAL.md` |
| Compiled contract artifacts | `contracts/silverscript/*.json` |
| Constructor arg files | `contracts/silverscript/args/*.json` |
| ZK proving/verification keys | Written to disk by `setup::run_trusted_setup()` |
| Polymesh SDK source | `vendor/polymesh-sdk/` |
| Polymesh API Rust crate | `vendor/polymesh-api/` |

## SilverScript Key Patterns

### Compiler Usage
```bash
# No constructor args:
vendor/silverscript/target/release/silverc contract.sil -o output.json

# With constructor args:
vendor/silverscript/target/release/silverc contract.sil --constructor-args args.json -o output.json
```

### Constructor Args JSON Format
```json
[
  {"kind": "array", "data": [{"kind": "byte", "data": 1}, ...]},
  {"kind": "int", "data": 42}
]
```
- byte[32]/pubkey: `{"kind": "array", "data": [{"kind":"byte","data":N} x 32]}`
- int: `{"kind": "int", "data": N}`
- NOT "pubkey" or "byte[32]" as kind — must use AST expression types

### validateOutputState Requirements
- Contract must have declared fields (not just constructor params)
- `validateOutputState(outputIndex, { field1: val1, field2: val2, ... })` must include ALL fields

## Rust Workspace Dependencies

| Crate | Key Dependencies |
|-------|-----------------|
| kaspa-adapter | sha2, hex, thiserror, tracing |
| compliance-rust | axum 0.8, rusqlite 0.32, ark-groth16 0.5, ark-bn254 0.5, ark-snark 0.5, ed25519-dalek 2, zk-circuits |
| oracle-pool | axum 0.8, reqwest 0.12, chrono 0.4, async-trait 0.1 |
| sync | reqwest 0.12 |
| tokenomics | (workspace deps only) |
| zk-circuits | ark-groth16 0.5, ark-bn254 0.5, ark-r1cs-std 0.5, ark-snark 0.5, ark-crypto-primitives 0.5 |

## Guardrails (every file, every function)
- `[K-RWA]` log prefix
- `// DISCLAIMER: Technical demo code — legal wrappers required in production`
- `// SPDX-License-Identifier: MIT`
- Testnet-12 ONLY: `ws://tn12-node.kaspa.com:17210`
- No hardcoded keys: `REPLACE_WITH_TESTNET_WALLET`
- Fictional assets: `KPROP-NYC-TEST`
- Oracle = centralized multisig (NOT miner-attested, per Ori Newman)
- vProgs = NOT ready, stub only

## Git State
- Initial commit: `d749514` on `master`
- Branches: feature/covenants, feature/compliance, feature/frontend, feature/dkg-sync, feature/tokenomics, feature/zk-circuits
- M1 work not yet committed

## Milestone Plan
- **M0** ✅ Scaffold + DKG + Rust init (Week 1)
- **M1** ✅ SilverScript contracts + Groth16 ZK prover (Weeks 2-3)
- **M2** ✅ Full Rust Polymesh compliance port (Weeks 4-5)
- **M3** ✅ ASTM token + simulated oracle + state-verity sync (Weeks 6-7)
- **M4** ✅ Frontend swap + E2E cycle (Weeks 8-9)
- **M5** Pending: Formal verification + security + whitepaper (Weeks 10-12)
