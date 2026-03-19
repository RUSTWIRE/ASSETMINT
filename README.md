# AssetMint

> **DISCLAIMER: This is technical demo code — legal wrappers required in production.**
> All transactions target Kaspa Testnet-12 ONLY. No real assets. No mainnet.
> SPDX-License-Identifier: MIT

The definitive institutional-grade, ZK-private, sovereign Real-World Asset (RWA) tokenization platform on Kaspa.

## Architecture

- **Compliance Engine** (`services/compliance-rust/`) — Full Rust port of Polymesh SDK patterns: identity registry, claims, transfer rules, Groth16 ZK-KYC proofs
- **SilverScript Covenants** (`contracts/silverscript/`) — On-chain enforcement via KIP-10 introspection opcodes
- **DKG Edge Node** (`infrastructure/dkg-node/`) — Self-hosted OriginTrail Knowledge Graph for private asset metadata
- **Oracle Pool** (`services/oracle-pool/`) — Simulated centralized multisig oracle (upgrade stub for future miner-attested)
- **ASTM Token** (`tokenomics/`) — KRC-20 protocol token with staking and governance
- **Frontend** (`apps/dashboard-fe/`) — Forked Hedera RWA DeFi accelerator, adapted for Kaspa

## Quick Start

```bash
# Clone and enter
git clone <repo-url> && cd ASSETMINT

# One-command setup and demo launch
make setup && make demo
```

This starts the compliance API on port 3001 and the dashboard on port 3000. See [docs/QUICKSTART.md](docs/QUICKSTART.md) for prerequisites and verification steps.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/QUICKSTART.md](docs/QUICKSTART.md) | Prerequisites, setup, and first verification |
| [docs/DEMO-WALKTHROUGH.md](docs/DEMO-WALKTHROUGH.md) | Guided tour of the minting wizard and compliance flow |
| [docs/API-REFERENCE.md](docs/API-REFERENCE.md) | All 12 compliance API endpoints with request/response examples |
| [docs/CONTRACTS.md](docs/CONTRACTS.md) | 7 deployed SilverScript covenants with TX hashes and P2SH addresses |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture and component overview |
| [docs/INVESTOR-BRIEF.md](docs/INVESTOR-BRIEF.md) | Investor-facing project summary |
| [docs/SECURITY-AUDIT.md](docs/SECURITY-AUDIT.md) | Security considerations and audit notes |

## Network Configuration

| Service | Endpoint |
|---------|----------|
| Kaspa Testnet-12 | `ws://tn12-node.kaspa.com:17210` |
| DKG Edge Node | `http://localhost:8900` |
| Compliance API | `http://localhost:3001` |
| Oracle API | `http://localhost:3002` |

## Test Assets

All assets are **fictional** and for testing only:
- Ticker: `KPROP-NYC-TEST`
- Protocol token: `ASTM` (KRC-20)

## License

MIT — See LICENSE file.
