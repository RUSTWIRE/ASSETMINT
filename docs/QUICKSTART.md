<!-- DISCLAIMER: Technical demo code -->
<!-- SPDX-License-Identifier: MIT -->

# AssetMint Quick Start

Get the demo running in under five minutes.

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Workspace build (compliance engine, ZK circuits, tokenomics) |
| Node.js | 18+ | Dashboard frontend (Next.js) |
| kaspad | TN12 | Local Kaspa Testnet-12 node for live blockchain queries |

Optional: Docker (for the Sovereign Metadata Service on port 8900).

## Quick Start

```bash
# Clone and enter
git clone <repo-url> && cd ASSETMINT

# One-command setup and demo launch
make setup && make demo
```

`make setup` installs Rust dependencies, builds the workspace, installs frontend packages, and runs a ZK trusted setup.

`make demo` starts the compliance API on port 3001 and the frontend on port 3000.

## Service Ports

| Service | Port | URL |
|---------|------|-----|
| Dashboard Frontend | 3000 | `http://localhost:3000` |
| Compliance API | 3001 | `http://localhost:3001` |
| Sovereign Metadata | 8900 | `http://localhost:8900` |
| kaspad RPC | 17210 | `ws://127.0.0.1:17210` |

## First Verification

Confirm the compliance API is running:

```bash
curl http://localhost:3001/health
```

Expected response:

```json
{
  "status": "ok",
  "service": "assetmint-core",
  "kaspa_connected": false
}
```

`kaspa_connected` will be `true` if a local kaspad instance is running on TN12.

## Next Steps

See [DEMO-WALKTHROUGH.md](DEMO-WALKTHROUGH.md) for a full guided tour of the minting wizard, compliance checks, and covenant contracts.
