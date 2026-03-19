<!-- DISCLAIMER: Technical demo code — legal wrappers required in production -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint

> **DISCLAIMER: This is technical demo code -- legal wrappers required in production.**
> All transactions target Kaspa Testnet-12 ONLY. No real assets. No mainnet.

The definitive institutional-grade, ZK-private, sovereign Real-World Asset (RWA) tokenization platform on Kaspa.

## Architecture

- **Compliance Engine** (`services/assetmint-core/`) -- Full Rust reimplementation of Polymesh SDK patterns: identity registry, claims, transfer rules, Groth16 ZK-KYC proofs (115 tests, 18 confirmed TN12 transactions)
- **SilverScript Covenants** (`contracts/silverscript/`) -- 8 deployed contracts (7 SilverScript + 1 clawback covenant) with 3 proven covenant executions on TN12
- **Sovereign Metadata Service** (`infrastructure/dkg-node/sovereign-metadata/`) -- Self-hosted private metadata store with SHA-256 integrity hashes and tamper detection, running on port 8900
- **Oracle Pool** (`services/oracle-pool/`) -- Simulated centralized multisig oracle (upgrade stub for future miner-attested)
- **ASTM Token** (`tokenomics/`) -- KRC-20 protocol token with staking and governance
- **Frontend** (`apps/dashboard-fe/`) -- Next.js 15 dashboard with live service detection
- **CLI** (`assetmint` binary) -- 9 commands for compliance API interaction

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
| [docs/QUICKSTART.md](docs/QUICKSTART.md) | Prerequisites, setup, CLI usage, and first verification |
| [docs/DEMO-WALKTHROUGH.md](docs/DEMO-WALKTHROUGH.md) | Guided tour of the minting wizard and compliance flow |
| [docs/API-REFERENCE.md](docs/API-REFERENCE.md) | All 12 compliance API endpoints with request/response examples |
| [docs/CONTRACTS.md](docs/CONTRACTS.md) | 8 deployed contracts with TX hashes and P2SH addresses |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, component overview, and CI pipeline |
| [docs/SECURITY-AUDIT.md](docs/SECURITY-AUDIT.md) | Security considerations and audit notes |
| [INVESTOR-BRIEF.md](docs/INVESTOR-BRIEF.md) | Investor-facing project summary |
| [FUNCTIONALITY-REPORT.md](FUNCTIONALITY-REPORT.md) | Honest scoring and gap analysis (~8.2/10) |

## Network Configuration

| Service | Endpoint |
|---------|----------|
| Kaspa Testnet-12 | `ws://tn12-node.kaspa.com:17210` |
| Sovereign Metadata | `http://localhost:8900` |
| Compliance API | `http://localhost:3001` |
| Oracle API | `http://localhost:3002` |

## Configuration

| Variable | Purpose | Default |
|----------|---------|---------|
| `CLAIM_ISSUER_KEY` | Claim signing key (hex) | Test key (WARNING) |
| `OPERATOR_PRIVATE_KEY` | Server-side Kaspa signing key (hex) | Test key (WARNING) |
| `API_KEY` | Write endpoint auth | None (demo mode) |
| `CORS_ORIGIN` | Allowed origin | `http://localhost:3000` |
| `IDENTITY_DB_PATH` | SQLite path | In-memory |
| `AUDIT_LOG_PATH` | File-based audit log | stdout |
| `ZK_KEYS_DIR` | ZK proving/verification keys | `/tmp/assetmint_compliance_keys` |
| `ALLOW_UNSAFE_THRESHOLD` | Enable XOR threshold signing (testing only) | Disabled |

See [docs/QUICKSTART.md](docs/QUICKSTART.md) for full security configuration.

## CI/CD

GitHub Actions CI runs on every push and pull request with 3 parallel jobs:
- **Build** -- workspace compilation
- **Test** -- 115 lib tests across 6 crates
- **Lint** -- `cargo clippy` and `cargo fmt --check`

## Test Assets

All assets are **fictional** and for testing only:
- Ticker: `KPROP-NYC-TEST`
- Protocol token: `ASTM` (KRC-20)

## License

MIT -- See LICENSE file.
