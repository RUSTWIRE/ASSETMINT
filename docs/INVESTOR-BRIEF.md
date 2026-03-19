<!-- DISCLAIMER: Technical demo code — legal wrappers required in production -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint: Institutional RWA Tokenization on Kaspa

> The first ZK-private, compliance-native Real-World Asset platform on the fastest pure Proof-of-Work Layer 1.

---

## The Problem

Over $16 trillion in real-world assets -- real estate, commodities, fine art, private credit -- remain locked in illiquid markets with no efficient secondary trading. Existing tokenization platforms face fundamental limitations:

- **High gas fees** on EVM chains make micro-transfers uneconomical
- **Slow finality** (minutes to hours) creates settlement risk for institutional participants
- **Account-model vulnerabilities** (reentrancy, front-running) require extensive audit overhead
- **Privacy gaps** force investors to expose personal identity data on-chain for compliance checks

Current solutions (Polymesh, Securitize, tZERO) bolt compliance onto chains that were not designed for it. The result is expensive, fragile, and privacy-invasive.

---

## The Solution

**AssetMint** delivers institutional-grade RWA tokenization on Kaspa -- the fastest pure PoW Layer 1, running at 10 blocks per second with sub-second probabilistic finality via PHANTOM/GHOSTDAG consensus.

### Four Key Innovations

**1. UTXO-Native Compliance (SilverScript Covenants)**
Transfer restrictions are enforced at the UTXO level, not in smart contract storage. Covenant preservation ensures tokens can never escape their compliance rules. The UTXO model eliminates reentrancy by design and enables parallel transaction validation.

**2. ZK-KYC Privacy (Groth16 Proofs)**
Investors prove they are in the set of approved addresses using zero-knowledge proofs. The chain verifies compliance without learning the investor's identity. Only a 32-byte proof hash is stored on-chain.

**3. Sovereign Provenance (Self-Hosted Metadata)**
Asset metadata -- title records, valuations, legal documentation -- is stored in a self-hosted Sovereign Metadata Service with SHA-256 integrity hashes and tamper detection. Metadata hashes are committed to the Kaspa DAG for immutable, verifiable provenance anchored to the asset token. All data stays on YOUR infrastructure -- private by default.

**4. ASTM Protocol Token (KRC-20)**
The platform's native token captures fees from every RWA transfer, distributes staking rewards, and enables on-chain governance for protocol upgrades.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                    Dashboard (Next.js 15)                        │
├──────────────┬────────────────┬───────────────────────────────────┤
│ Compliance   │  Oracle API    │  Sovereign Metadata              │
│ API (:3001)  │  (:3002)       │  (:8900)                         │
├──────────────┴────────────────┴───────────────────────────────────┤
│               Rust Workspace (6 crates)                          │
│  compliance  │  oracle  │  sync  │  tokenomics  │  zk-circuits   │
├──────────────────────────────────────────────────────────────────┤
│          SilverScript Covenants (8 contracts)                    │
├──────────────────────────────────────────────────────────────────┤
│       Kaspa Testnet-12  •  10 BPS  •  PHANTOM/GHOSTDAG          │
└──────────────────────────────────────────────────────────────────┘
```

The full system comprises 8 deployed contracts (7 SilverScript + 1 clawback covenant), 6 Rust crates, a sovereign metadata service, 2 REST APIs, a CLI with 9 commands, a Next.js dashboard with 8 pages, 115 passing tests, and GitHub Actions CI with 3 parallel jobs.

---

## Market Opportunity

| Metric | Value |
|--------|-------|
| Total addressable market (global illiquid assets) | $16T+ |
| Currently tokenized RWA | ~$10B |
| Kaspa market presence | Zero existing RWA platforms |
| Kaspa block throughput | 10 BPS (fastest pure PoW) |

AssetMint holds first-mover advantage as the only RWA tokenization platform on Kaspa. The chain's high throughput and sub-second finality make it uniquely suited for institutional settlement requirements, while its pure PoW consensus provides the security guarantees that regulated entities demand.

---

## Token Economics (ASTM)

ASTM is a KRC-20 inscription token on Kaspa that captures value from platform activity.

| Parameter | Value |
|-----------|-------|
| Ticker | ASTM |
| Standard | KRC-20 (Kaspa inscriptions) |
| Max supply | 1,000,000,000 |
| Decimals | 8 |
| Per-mint limit | 1,000 ASTM |

### Staking

| Parameter | Value |
|-----------|-------|
| Minimum stake | 100 ASTM |
| Minimum lock | 7 days |
| Base APY | 5% (500 bps) |
| Mechanism | Time-locked covenant UTXOs |

Longer lock durations increase yield. Staked tokens are locked in SilverScript covenant UTXOs that enforce the time-lock on-chain.

### Fee Capture

Every RWA transfer on the platform incurs a fee (flat 0.0005 KAS + 0.01% proportional, capped at 0.001 KAS). Fees are distributed:

| Destination | Share | Purpose |
|-------------|-------|---------|
| Burn | 30% | Deflationary supply reduction |
| Stakers | 50% | Distributed to staking pool |
| Treasury | 20% | Platform operations and development |

### Governance

ASTM stakers vote on protocol proposals with weight proportional to their staked amount. Proposals that reach the execution threshold are recorded on-chain via `OP_RETURN` encoding.

---

## Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| M0 | Done | Project scaffold, Rust workspace, sovereign metadata config, vendor repos |
| M1 | Done | 7 SilverScript contracts, Groth16 ZK-KYC circuit, trusted setup |
| M2 | Done | Polymesh compliance reimplementation, identity registry, rules engine, REST API |
| M3 | Done | ASTM token (KRC-20), staking, governance, oracle with Ed25519 attestations |
| M4 | Done | Next.js 15 dashboard (8 pages), E2E integration test, CLI with 9 commands |
| M5 | Done | Sovereign metadata, covenant builder, on-chain staking, 18 TN12 transactions, 115 tests, CI pipeline |
| M6 | In progress | Security hardening, formal verification, investor whitepaper |
| M7 | Planned | Mainnet preparation, deployment tooling, launch readiness |

---

## Team

*[Team section to be completed]*

---

## Disclaimer

This document describes a **technical demonstration** running on **Kaspa Testnet-12**. It is not financial advice, not an offer of securities, and not intended for production use. All assets referenced (e.g., KPROP-NYC-TEST) are fictional. The ASTM token exists only on testnet and has no monetary value.

Legal, regulatory, and compliance wrappers are required before any production deployment. Consult qualified legal counsel before proceeding with any real-world asset tokenization.
