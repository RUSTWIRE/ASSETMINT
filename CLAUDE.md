# AssetMint — CLAUDE.md

## Build & Test
- `cargo test --workspace --lib` — run all 115 tests (42+10+12+9+35+7)
- `cargo check --workspace` — fast compile check
- `cargo fmt --all` — format all Rust code
- `cd apps/dashboard-fe && npx tsc --noEmit` — frontend type check
- `cd apps/dashboard-fe && npm run build` — full Next.js build
- `node -c infrastructure/dkg-node/sovereign-metadata/server.js` — metadata syntax check
- `make backend` — run API server with all env vars
- `make cli ARGS="health"` — run CLI commands

## Architecture
- Rust workspace: 6 crates (assetmint-core, kaspa-adapter, dkg-bridge, sync, tokenomics, zk-circuits)
- Frontend: Next.js 14 at apps/dashboard-fe/ (8 pages, dark theme, Tailwind + lucide-react)
- Metadata service: Node.js at infrastructure/dkg-node/sovereign-metadata/server.js (port 8900)
- API server: Axum on port 3001, read routes open, write routes need X-API-Key header
- Kaspa TN12 RPC: ws://127.0.0.1:17210 (10 BPS PHANTOM/GHOSTDAG)

## Gotchas
- NEVER use `compliance-rust` — old name, now `assetmint-core`
- NEVER hardcode private keys in source — use env vars
- Test parallelism: `std::env::set_var`/`remove_var` is racy — avoid env-dependent tests in parallel
- TN12 UTXO fragmentation: deploy covenants with ≤0.5 KAS to avoid storage mass errors
- KIP-9 storage mass: `C / output_value`, MAX_INPUTS=25 — too many small UTXOs will fail
- Frontend `ktt` package uses `file:` reference to `../../../KTT - KCR-20T` — won't resolve in CI
- Multiple agents editing api.rs simultaneously causes merge conflicts — assign one agent per file
- External evaluator outputs hallucinate CLI commands and inflate scores — always verify before implementing

## Conventions
- All files need DISCLAIMER header + SPDX-License-Identifier: MIT
- Log prefix: `[K-RWA]`
- Test assets: `KPROP-NYC-TEST`, test token: `ASTM`
- Honest scoring only (~8.2/10, never inflate)
- Dark theme colors: bg-gray-950/900/800, accent indigo, success emerald, danger red, warning amber-900/20

## Key Commands
- `cargo run -p assetmint-core --bin assetmint -- <cmd>` — CLI binary
- `cargo test -p kaspa-adapter --test <name> -- --nocapture` — run single TN12 integration test
- Alice test key: `ab08984d79824336161553b77e366abde831ebde78d78f0440e6833b2f2e2f92`
