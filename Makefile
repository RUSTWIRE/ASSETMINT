# DISCLAIMER: Technical demo code — legal wrappers required in production
# SPDX-License-Identifier: MIT
#
# AssetMint — One-Command Demo Startup
# Usage: make demo

.PHONY: demo setup backend frontend test stop clean help

help: ## Show available commands
	@echo "AssetMint Demo Commands:"
	@echo "  make setup     Install dependencies (npm + cargo)"
	@echo "  make demo      Start backend API + frontend dashboard"
	@echo "  make backend   Start compliance API on :3001"
	@echo "  make frontend  Start Next.js dashboard on :3000"
	@echo "  make test      Run all workspace tests"
	@echo "  make stop      Stop all running services"
	@echo ""
	@echo "Prerequisites: Rust toolchain, Node.js 18+, kaspad on ws://127.0.0.1:17210"

setup: ## Install all dependencies
	@echo "[K-RWA] Installing frontend dependencies..."
	cd apps/dashboard-fe && npm install --silent
	@echo "[K-RWA] Building Rust workspace..."
	cargo build --workspace --quiet
	@echo "[K-RWA] Setup complete."

backend: ## Start AssetMint compliance API (port 3001)
	@echo "[K-RWA] Starting AssetMint Core API on :3001..."
	@cargo run -p assetmint-core --quiet &
	@sleep 3
	@echo "[K-RWA] Backend API ready: http://localhost:3001"
	@echo "[K-RWA] Health check: curl http://localhost:3001/health"

frontend: ## Start Next.js dashboard (port 3000)
	@echo "[K-RWA] Starting dashboard on :3000..."
	@cd apps/dashboard-fe && npx next dev --port 3000 &
	@sleep 2
	@echo "[K-RWA] Frontend ready: http://localhost:3000"

demo: backend frontend ## Start full demo (backend + frontend)
	@echo ""
	@echo "[K-RWA] ========================================"
	@echo "[K-RWA]  AssetMint Demo Running"
	@echo "[K-RWA]  Dashboard:  http://localhost:3000"
	@echo "[K-RWA]  API:        http://localhost:3001"
	@echo "[K-RWA]  Kaspad:     ws://127.0.0.1:17210"
	@echo "[K-RWA] ========================================"
	@echo "[K-RWA] Press Ctrl+C to stop all services"

test: ## Run all workspace tests
	cargo test --workspace --lib

stop: ## Stop all running services
	@echo "[K-RWA] Stopping services..."
	-@pkill -f "target.*assetmint" 2>/dev/null || true
	-@pkill -f "next dev" 2>/dev/null || true
	@echo "[K-RWA] Services stopped."

clean: ## Clean build artifacts
	cargo clean
	rm -rf apps/dashboard-fe/.next apps/dashboard-fe/node_modules
