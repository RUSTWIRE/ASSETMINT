# AssetMint 10/10 Rolls-Royce Rubric

> DISCLAIMER: Technical demo code — legal wrappers required in production.
> SPDX-License-Identifier: MIT

## Scoring (fill in as milestones complete)

| # | Criterion | Target | Status | Evidence |
|---|-----------|--------|--------|----------|
| 1 | Rust compliance engine | Full Polymesh port with identity, claims, rules | Pending | |
| 2 | ZK-KYC proofs | Groth16 <200ms gen, <50ms verify | Pending | |
| 3 | Sovereign DKG | Self-hosted Edge Node, private-only, localhost:8900 | Pending | |
| 4 | SilverScript covenants | 5 contracts compiled + all paths tested on TN12 | Pending | |
| 5 | ASTM token | KRC-20 deployed + staking + governance on TN12 | Pending | |
| 6 | Oracle | Simulated multisig + IOraclePool upgrade stub | Pending | |
| 7 | Fees | ≤0.001 KAS per transfer | Pending | |
| 8 | Test coverage | 100% core paths (cargo test + integration) | Pending | |
| 9 | Security | Formal specs + audit report + property tests | Pending | |
| 10 | Documentation | Whitepaper + full RustDoc + architecture diagram | Pending | |

## Overall Score: __/10

## Milestone Evidence

### M0: Scaffold
- Folder structure: ✅
- Vendor repos cloned: ✅
- DKG Edge Node:
- Rust workspace:
- SilverScript compiler:

### M1: Contracts + ZK
- rwa-core.sil compiled:
- clawback.sil compiled:
- state-verity.sil compiled:
- zkkyc-verifier.sil compiled:
- reserves.sil compiled:
- ZK circuit setup:
- Proof gen benchmark:
- Proof verify benchmark:

### M2: Compliance
- Identity API:
- Claims API:
- Rules engine:
- ZK prover integration:
- Merkle tree:

### M3: Token + Oracle + Sync
- ASTM deploy tx:
- Staking covenant:
- Oracle attestation:
- State sync loop:

### M4: Frontend + E2E
- Frontend loads:
- Wallet connects:
- Mint tx hash:
- Transfer tx hash:
- Clawback tx hash:
- State-sync tx hash:

### M5: Security + Docs
- Formal verification report:
- Load test results:
- Security audit:
- Investor whitepaper:
