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
# 1. Clone and enter
git clone <repo-url> && cd ASSETMINT

# 2. Clone vendor repos
./scripts/clone-vendors.sh

# 3. Start DKG Edge Node
cd infrastructure/dkg-node && docker-compose up -d

# 4. Build Rust workspace
cargo build

# 5. Build SilverScript compiler
cd vendor/silverscript && cargo build --release

# 6. Start compliance API
cd services/compliance-rust && cargo run

# 7. Start frontend
cd apps/dashboard-fe && npm install && npm run dev
```

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
