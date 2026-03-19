<!-- DISCLAIMER: Technical demo code — legal wrappers required in production -->
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

## CLI Usage

AssetMint includes a command-line interface with 9 commands for interacting with the compliance API. The CLI communicates with the running Axum HTTP server.

### Build and Run

```bash
# Via make (recommended)
make cli ARGS="health"

# Or directly via cargo
cargo run -p assetmint-core --bin assetmint -- health
```

### Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `health` | Check API health status | `make cli ARGS="health"` |
| `network` | Display Kaspa network info | `make cli ARGS="network"` |
| `identity register` | Register a new DID | `make cli ARGS="identity register --did did:kaspa:alice --key 0xabc123"` |
| `identity get` | Look up an identity | `make cli ARGS="identity get --did did:kaspa:alice"` |
| `claim issue` | Issue a compliance claim | `make cli ARGS="claim issue --subject did:kaspa:alice --type KycVerified --expiry 0"` |
| `compliance check` | Evaluate transfer compliance | `make cli ARGS="compliance check --sender did:kaspa:alice --receiver did:kaspa:bob --asset KPROP-NYC-TEST --amount 1000"` |
| `balance` | Query address balance | `make cli ARGS="balance --address kaspatest:qq..."` |
| `transfer` | Execute a compliant transfer | `make cli ARGS="transfer --sender-did did:kaspa:alice --receiver-did did:kaspa:bob --receiver-address kaspatest:qq... --amount 100000000 --asset KPROP-NYC-TEST"` |
| `merkle-root` | Query current Merkle root | `make cli ARGS="merkle-root"` |

### Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--api-url` | Base URL of the AssetMint API | `http://localhost:3001` |
| `--api-key` | API key for write endpoints | None |

### Full Compliance Cycle via CLI

```bash
# 1. Register identities
make cli ARGS="identity register --did did:kaspa:alice --key 0xabc123"
make cli ARGS="identity register --did did:kaspa:bob --key 0xdef456"

# 2. Issue KYC claims
make cli ARGS="--api-key YOUR_KEY claim issue --subject did:kaspa:alice --type KycVerified --expiry 0"
make cli ARGS="--api-key YOUR_KEY claim issue --subject did:kaspa:bob --type KycVerified --expiry 0"

# 3. Check compliance
make cli ARGS="compliance check --sender did:kaspa:alice --receiver did:kaspa:bob --asset KPROP-NYC-TEST --amount 1000"

# 4. Query Merkle root
make cli ARGS="merkle-root"
```

## Security Configuration

AssetMint uses environment variables for security-sensitive configuration. In demo mode (no env vars set), the platform uses default test values with warnings.

| Variable | Purpose | Default | Required for Production |
|----------|---------|---------|----------------------|
| `CLAIM_ISSUER_KEY` | Ed25519 claim signing key (64 hex chars) | Test key `[42u8; 32]` with WARNING | Yes |
| `OPERATOR_PRIVATE_KEY` | Server-side Kaspa signing key (64 hex chars) | Alice test key with WARNING | Yes |
| `API_KEY` | API key for write endpoint authentication | None (auth skipped) | Yes |
| `CORS_ORIGIN` | Allowed CORS origin | `http://localhost:3000` | Yes |
| `IDENTITY_DB_PATH` | SQLite database path for identity persistence | In-memory (lost on restart) | Yes |
| `AUDIT_LOG_PATH` | File-based audit log path | stdout | Yes |
| `ZK_KEYS_DIR` | Directory for ZK proving/verification keys | `/tmp/assetmint_compliance_keys` | Yes |
| `ALLOW_UNSAFE_THRESHOLD` | Enable XOR threshold signing (testing only) | Disabled | No |

### Production Startup

```bash
export CLAIM_ISSUER_KEY=$(openssl rand -hex 32)
export OPERATOR_PRIVATE_KEY=$(openssl rand -hex 32)
export API_KEY=$(openssl rand -hex 16)
export CORS_ORIGIN=https://your-domain.com
export IDENTITY_DB_PATH=/var/lib/assetmint/identities.db
export AUDIT_LOG_PATH=/var/lib/assetmint/audit.log
export ZK_KEYS_DIR=/var/lib/assetmint/zk-keys
make demo
```

## Next Steps

See [DEMO-WALKTHROUGH.md](DEMO-WALKTHROUGH.md) for a full guided tour of the minting wizard, compliance checks, and covenant contracts.
