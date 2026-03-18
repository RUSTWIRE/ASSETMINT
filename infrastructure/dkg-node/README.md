# DKG Edge Node — Self-Hosted

> DISCLAIMER: Technical demo code — legal wrappers required in production.
> SPDX-License-Identifier: MIT

## Overview

Self-hosted OriginTrail DKG Edge Node for AssetMint.
All Knowledge Assets are **private by default**.

## Requirements

- Docker & Docker Compose
- 8 GB RAM, 4 CPU cores, 20+ GB storage
- An OpenAI API key (required by DKG Edge Node for Knowledge Mining)

## Quick Start

```bash
cp .env.example .env
# Edit .env with your API keys and wallet keys
docker-compose up -d
```

## Verify

```bash
curl http://localhost:8900
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Node status |
| `/assets` | POST | Publish Knowledge Asset |
| `/assets/:ual` | GET | Retrieve Knowledge Asset |

## Private Mode

This node runs in **private mode**. Knowledge Assets published here are NOT
visible on the public OriginTrail DKG network. This ensures sovereign data
ownership for AssetMint RWA metadata.
