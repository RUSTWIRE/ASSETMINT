#!/usr/bin/env bash
# DISCLAIMER: Technical demo code — legal wrappers required in production
# SPDX-License-Identifier: MIT
#
# DKG Edge Node startup script for AssetMint
# Starts OriginTrail DKG v9 with MySQL + Redis + Blazegraph
#
# Usage: chmod +x start.sh && ./start.sh

set -euo pipefail

LOG_PREFIX="[K-RWA]"

echo "${LOG_PREFIX} === DKG Edge Node Startup ==="

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "${LOG_PREFIX} ERROR: Docker not installed"; exit 1; }
command -v docker compose >/dev/null 2>&1 || { echo "${LOG_PREFIX} ERROR: Docker Compose not installed"; exit 1; }

# Navigate to script directory (where docker-compose.yml lives)
cd "$(dirname "$0")"

# Check if .env exists
if [ ! -f .env ]; then
    echo "${LOG_PREFIX} Creating .env from .env.example..."
    cp .env.example .env
fi

# Start services
echo "${LOG_PREFIX} Starting DKG Edge Node services..."
docker compose up -d

# Wait for health
echo "${LOG_PREFIX} Waiting for DKG node to become healthy..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:8900 > /dev/null 2>&1; then
        echo "${LOG_PREFIX} DKG Edge Node is HEALTHY on port 8900"
        echo "${LOG_PREFIX} ==================================="
        docker compose ps
        exit 0
    fi
    echo "${LOG_PREFIX}   Attempt $i/30 — waiting 2s..."
    sleep 2
done

echo "${LOG_PREFIX} ERROR: DKG node failed to start within 60s"
docker compose logs --tail=20
exit 1
