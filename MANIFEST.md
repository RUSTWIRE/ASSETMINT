# AssetMint Module Manifest

> DISCLAIMER: Technical demo code — legal wrappers required in production.
> SPDX-License-Identifier: MIT

## Module Inventory

| Module | Language | Status | Description |
|--------|----------|--------|-------------|
| `contracts/silverscript/` | SilverScript (.sil) | M1 | Covenant contracts: rwa-core, clawback, state-verity, zkkyc-verifier, reserves |
| `contracts/schemas/` | JSON | M0 | AssetMint-1.0 asset definition schema |
| `packages/kaspa-adapter/` | Rust | M2 | rusty-kaspa WASM wrapper: client, wallet, tx-builder, script loader |
| `packages/dkg-bridge/` | TypeScript | M2 | Thin HTTP client for self-hosted DKG Edge Node |
| `services/assetmint-core/` | Rust | M2 | Full Polymesh compliance port: identity, claims, rules, ZK prover/verifier |
| `services/oracle-pool/` | Rust | M3 | Simulated centralized multisig oracle + IOraclePool upgrade stub |
| `services/sync/` | Rust | M3 | DKG state-verity sync loop |
| `tokenomics/` | Rust | M3 | ASTM KRC-20 token, staking, governance, fee model |
| `zk-circuits/` | Rust | M1 | Groth16 ZK-KYC circuits (ark-groth16 + ark-bn254) |
| `infrastructure/dkg-node/` | Docker | M0 | Self-hosted OriginTrail DKG Edge Node |
| `apps/dashboard-fe/` | TypeScript/Next.js | M4 | Forked Hedera RWA frontend adapted for Kaspa |
| `security/` | Markdown | M5 | Audit reports, formal specs, model checks |

## Vendor Dependencies

| Repo | Purpose |
|------|---------|
| `vendor/polymesh-sdk/` | Reference: compliance patterns |
| `vendor/polymesh-api/` | Rust crate: Polymesh API types |
| `vendor/hedera-accelerator-rwa-defi-fe/` | Fork source: frontend |
| `vendor/dkg.js/` | Reference: DKG API patterns |
| `vendor/edge-node-installer/` | Reference: DKG node setup |
| `vendor/silverscript/` | Build: SilverScript compiler |
| `vendor/rusty-kaspa/` | Reference: WASM bindings |
| `vendor/kips/` | Reference: KIP-10 opcode spec |
| `vendor/x402-KAS/` | Reference: covenant patterns |

## Milestone Tracking

- [x] M0: Scaffold + DKG Edge Node + Rust init
- [ ] M1: SilverScript contracts + Groth16 ZK prover
- [ ] M2: Full Rust Polymesh compliance port
- [ ] M3: ASTM token + simulated oracle + state-verity sync
- [ ] M4: Frontend swap + end-to-end cycle
- [ ] M5: Formal verification + security + whitepaper
