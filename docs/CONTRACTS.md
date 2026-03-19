<!-- DISCLAIMER: Technical demo code -->
<!-- SPDX-License-Identifier: MIT -->

# Deployed SilverScript Covenant Contracts

All seven contracts are deployed on **Kaspa Testnet-12** as P2SH scripts compiled from SilverScript (`.sil`) source files. Each contract uses KIP-10 introspection opcodes for covenant enforcement.

Source: [`apps/dashboard-fe/src/lib/contracts.ts`](../apps/dashboard-fe/src/lib/contracts.ts)

---

## 1. Clawback

Wraps RWA token UTXOs with a regulatory clawback mechanism. The asset owner can spend normally, but the issuer can unconditionally reclaim the asset for regulatory freeze, court order, or AML action. Clawback transactions include OP_RETURN metadata with the reason.

| Field | Value |
|-------|-------|
| Script Size | 161 bytes |
| Deploy TX | `6080b47733e42d1cff8597cab14b2a412d8e423bed36add64d980c158f5c77eb` |
| P2SH Address | `kaspatest:ppztfhpzpxkqkxum37ymje2dehrj0l49t3c75a3k4pu3jzp37edn202ftet8y` |
| Entrypoints | `ownerSpend`, `issuerClawback` |
| Source | [`contracts/silverscript/clawback.sil`](../contracts/silverscript/clawback.sil) |

---

## 2. RwaCore

The primary transfer guard for RWA tokens. Transfers require a valid ZK-KYC proof (Groth16 proof hash committed on-chain). Uses a covenant pattern to ensure the output recreates the same contract, preserving transfer restrictions across all future spends.

| Field | Value |
|-------|-------|
| Script Size | 395 bytes |
| Deploy TX | `d7ed495882132765eb1c1dabd2cb378e3dbe5f39b1770c0313e54782e5a6baec` |
| P2SH Address | `kaspatest:prhl2h3vdsq32u8068dqtm6x3qazz4nz9jkv9lq2u4j80c8c0ldrwqt9z3d2t` |
| Entrypoints | `zkTransfer`, `adminUpdate` |
| Source | [`contracts/silverscript/rwa-core.sil`](../contracts/silverscript/rwa-core.sil) |

---

## 3. StateVerity

Manages on-chain state for an RWA asset. Holds references to the DKG Universal Asset Locator (UAL), the current oracle price attestation hash, and the compliance Merkle root. State transitions must be oracle-attested and signed by the state manager.

| Field | Value |
|-------|-------|
| Script Size | 316 bytes |
| Deploy TX | `94c50753b05e7d998af30fa51aad4d27f2e7fdd0e9ae48b655255b94d129fe5f` |
| P2SH Address | `kaspatest:pq6xyf8f4tzpeuz4s6yy8063j6g6dwv0a4lcerv4uc98m99shgpcsftdcl5d7` |
| Entrypoints | `updateState`, `managerReclaim` |
| Source | [`contracts/silverscript/state-verity.sil`](../contracts/silverscript/state-verity.sil) |

---

## 4. ZkKycVerifier

On-chain ZK proof verification via hash-commitment scheme. Since Kaspa script does not yet have native pairing operations, the prover commits the proof off-chain and the on-chain script verifies the hash binding between the proof, verification key, and public inputs. Full Groth16 verification (pairing checks) runs in the assetmint-core service.

| Field | Value |
|-------|-------|
| Script Size | 396 bytes |
| Deploy TX | `c29499adf3d1353ce914d8e61184357c31d479039ee91c41a09345953bf93c45` |
| P2SH Address | `kaspatest:pzhqgz42uftlpg2hpekn7sh48ddmmny9wrql8nczk2nuevsjgp7cz99szuyqs` |
| Entrypoints | `verifyProof`, `updateVerifierKey` |
| Source | [`contracts/silverscript/zkkyc-verifier.sil`](../contracts/silverscript/zkkyc-verifier.sil) |

---

## 5. Reserves

Enforces collateralization of RWA tokens. Withdrawals are only permitted if the oracle-attested reserve ratio stays above the minimum threshold. Prevents under-collateralized withdrawals and allows deposits to increase collateral.

| Field | Value |
|-------|-------|
| Script Size | 372 bytes |
| Deploy TX | `346fdbd30cf88fd6e1ba60444cb3ea892cf59bc807019106b7e6f8f18f012e1b` |
| P2SH Address | `kaspatest:prlsah5judppj9np80zzp4qyrf90ccjnvd3u9uvhx8gzf7pjej33vkl0ln4vg` |
| Entrypoints | `withdraw`, `deposit`, `custodianReclaim` |
| Source | [`contracts/silverscript/reserves.sil`](../contracts/silverscript/reserves.sil) |

---

## 6. HTLC

Hash Time-Locked Contract for cross-chain atomic swaps. Enables trustless RWA token transfers between Kaspa and other blockchains. The recipient reveals a preimage matching the hashlock to claim funds; if unclaimed by the deadline, the sender can reclaim.

| Field | Value |
|-------|-------|
| Script Size | 195 bytes |
| Deploy TX | `1347b397ff482c8ed1f8b914eab5102425c891111c38016008b98df6d3390528` |
| P2SH Address | `kaspatest:prrz0mrxc3020lajzm4zj9gtf9q0nwp7ku05sen9fz4rlldw4d9z5t2ftdll5` |
| Entrypoints | `claimWithPreimage`, `refundAfterTimeout` |
| Source | [`contracts/silverscript/htlc.sil`](../contracts/silverscript/htlc.sil) |

---

## 7. Dividend

Proportional dividend distribution via Merkle holder tree. The issuer funds the contract, then each token holder claims their proportional share by proving membership in the holder tree via Merkle proof.

| Field | Value |
|-------|-------|
| Script Size | 406 bytes |
| Deploy TX | `6ec163e1882bda2ac238626112e525d20d90c1bb569828f1fd279e7aea294c9c` |
| P2SH Address | `kaspatest:prrf9w05fgvpq8k40t24pdcst0r99504fq50uma0q233a2fh8kln2gxllvp6p` |
| Entrypoints | `claimDividend`, `issuerTopUp`, `issuerReclaim` |
| Source | [`contracts/silverscript/dividend.sil`](../contracts/silverscript/dividend.sil) |
