# Security & Formal Verification

> DISCLAIMER: Technical demo code — legal wrappers required in production.
> SPDX-License-Identifier: MIT

## Overview

This directory contains security audit reports and formal verification specs
for AssetMint contracts and services.

## Structure

- `audit-reports/` — Security audit and formal verification reports
- `formal-specs/` — Property specifications for SilverScript contracts

## Audit Scope (Milestone 5)

1. **SilverScript Contracts** — All spending paths, unauthorized access, oracle manipulation
2. **Compliance Engine** — Rule bypass, claim forgery, Merkle collision
3. **DKG Integration** — Private data leakage, UAL spoofing
4. **Oracle Pool** — Price manipulation, signature forgery
5. **Tokenomics** — Staking exploits, governance attacks

## Formal Properties to Verify

- No unauthorized spend is possible on any covenant
- Covenant preservation holds for all valid transactions
- ZK proof verification is sound (no false acceptance)
- Compliance rules cannot be bypassed
- Oracle attestation requires threshold signatures
