# DISCLAIMER: Technical demo code — legal wrappers required in production
# SPDX-License-Identifier: MIT
#
# Multi-stage build for assetmint-core compliance API

FROM rust:1.82-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY packages/ packages/
COPY services/ services/
COPY tokenomics/ tokenomics/
COPY zk-circuits/ zk-circuits/
RUN cargo build --release -p assetmint-core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/assetmint-core /usr/local/bin/
EXPOSE 3001
CMD ["assetmint-core"]
