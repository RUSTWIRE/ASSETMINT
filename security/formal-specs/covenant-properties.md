# AssetMint SilverScript Covenant Property Specifications

Generated from source analysis of the 7 SilverScript contracts in `contracts/silverscript/`.

---

## 1. RwaCore (`rwa-core.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `zkTransfer` (line 30) | senderPk, senderSig, proofHash, newMerkleRoot, recipientPk | ZK-KYC guarded transfer of RWA tokens |
| `adminUpdate` (line 70) | issuerPk, issuerSig, updatedMerkleRoot | Issuer updates the set of approved addresses |

### Safety Properties

**S1 — No unauthorized spend.** Every spend requires a valid Schnorr signature verified via `checkSig(senderSig, senderPk)` (line 38) or `checkSig(issuerSig, issuerPk)` (line 77). The `adminUpdate` path additionally requires `blake2b(issuerPk) == issuerKeyHash` (line 76), binding the admin pubkey to the committed hash at contract creation. An attacker without the private key corresponding to `senderPk` or `issuerPk` cannot produce a valid signature.

**S2 — ZK proof binding prevents proof substitution.** The `zkTransfer` path computes `proofBinding = sha256(proofHash + zkVerifierKeyHash + merkleRoot + senderPkHash)` (line 45) and requires it to be non-zero (line 46). This binds the proof to the specific circuit (via `zkVerifierKeyHash`), the current approved set (via `merkleRoot`), and the sender identity (via `senderPkHash`). A proof generated for a different circuit, Merkle root, or sender cannot pass this check.

**S3 — Transfer amount is positive.** `require(transferAmount > 0)` at line 56 prevents zero-value transfers.

### Liveness Properties

**L1 — Funds are always recoverable.** The `adminUpdate` path (line 70) allows the issuer to update the Merkle root unconditionally (given valid signature). If all approved addresses are lost, the issuer can update the root to include new addresses. The full UTXO value minus fee is preserved (line 81), so funds remain accessible.

**L2 — No deadlock.** Both paths require only a single signer: `zkTransfer` needs any sender with a valid ZK proof; `adminUpdate` needs the issuer. Neither path depends on the other, so the contract cannot enter a state where no path is executable (assuming key availability).

### Covenant Preservation

**CP1 — Change output recreates contract state.** In `zkTransfer`, if `changeValue > MINER_FEE`, the second output must pass `validateOutputState(1, ...)` (line 64), which verifies the output script matches the contract bytecode with state `{merkleRoot: newMerkleRoot, zkVerifierKeyHash, issuerKeyHash}`. The `issuerKeyHash` and `zkVerifierKeyHash` are carried forward unchanged.

**CP2 — Admin path preserves covenant.** `adminUpdate` calls `validateOutputState(0, ...)` (line 84) with the updated Merkle root but the same `zkVerifierKeyHash` and `issuerKeyHash`. The covenant is recreated at output index 0.

### Value Conservation

**VC1 — zkTransfer:** `changeValue = inputValue - transferAmount - MINER_FEE` (line 51). Output 0 receives `transferAmount`, output 1 (if any) receives `changeValue`. Total outputs = `transferAmount + changeValue = inputValue - MINER_FEE`.

**VC2 — adminUpdate:** `require(tx.outputs[0].value == inputValue - MINER_FEE)` (line 81). Exactly the input minus fee.

---

## 2. Clawback (`clawback.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `ownerSpend` (line 24) | ownerSig, recipientPk | Normal owner transfer |
| `issuerClawback` (line 40) | issuerPk, issuerSig, issuerReceivePk | Issuer reclaims asset unconditionally |

### Safety Properties

**S1 — Owner path requires owner signature.** `checkSig(ownerSig, owner)` (line 26) where `owner` is the pubkey committed at contract creation. Only the owner's private key can authorize a spend through this path.

**S2 — Clawback path requires issuer identity.** `blake2b(issuerPk) == issuerKeyHash` (line 46) and `checkSig(issuerSig, issuerPk)` (line 47). The issuer must know both the private key and the preimage of the hash commitment.

**S3 — Clawback includes audit trail.** `require(tx.outputs.length >= 2)` (line 60) ensures a second output exists for OP_RETURN metadata, making clawback events visible on-chain for regulatory audit.

### Liveness Properties

**L1 — Owner always has a spending path.** The `ownerSpend` entrypoint requires only the owner's signature. As long as the owner retains their key, funds are spendable.

**L2 — Issuer always has reclaim capability.** `issuerClawback` requires only the issuer's key. This is a deliberate regulatory escape hatch.

### Covenant Preservation

This contract does NOT preserve a covenant. Both paths send to P2PK outputs (lines 29-30, 50-51). Once spent, the clawback guard is removed from the output. This is intentional: clawback is a one-time wrapper, not a persistent covenant.

### Value Conservation

**VC1 — ownerSpend:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 34). At most the miner fee is deducted.

**VC2 — issuerClawback:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 55). Same guarantee applies to the clawback path.

---

## 3. StateVerity (`state-verity.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `updateState` (line 32) | managerPk, managerSig, newDkgUalHash, newOracleAttestationHash, newComplianceMerkleRoot | Oracle-attested state transition |
| `managerReclaim` (line 62) | managerPk, managerSig | State manager reclaims UTXO (retirement/migration) |

### Safety Properties

**S1 — Only the state manager can update state.** Both paths require `blake2b(managerPk) == stateManagerKeyHash` (lines 40, 63) and `checkSig(managerSig, managerPk)` (lines 41, 64). The manager key hash is committed at deployment.

**S2 — Oracle attestation must be non-zero.** `require(newOracleAttestationHash != 0x00...00)` (line 45) prevents empty attestations from being accepted. While the actual oracle signature verification happens off-chain in the oracle-pool service, this provides a minimal on-chain sanity check.

### Liveness Properties

**L1 — State can always be updated.** The `updateState` path is callable whenever the manager has a valid key and a non-zero oracle attestation. No external lock or time dependency prevents execution.

**L2 — State UTXO is reclaimable.** `managerReclaim` (line 62) has no constraints beyond manager authentication. The UTXO can always be spent (no time lock, no covenant requirement on the output).

### Covenant Preservation

**CP1 — updateState preserves covenant.** `validateOutputState(0, ...)` at lines 52-57 ensures the output recreates the StateVerity contract with updated state variables. The `stateManagerKeyHash` is carried forward unchanged, preventing manager key rotation without a reclaim-and-redeploy.

**CP2 — managerReclaim breaks covenant.** No `validateOutputState` call. The manager can spend the UTXO to any output. This is the escape hatch for retirement.

### Value Conservation

**VC1 — updateState:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 49). Value is preserved across state transitions.

**VC2 — managerReclaim:** No value constraint. The manager can claim the full UTXO value to any address.

---

## 4. ZkKycVerifier (`zkkyc-verifier.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `verifyProof` (line 34) | proverPk, proverSig, proofCommitment, nullifierHash | Verify a ZK-KYC proof hash commitment |
| `updateVerifierKey` (line 74) | adminPk, adminSig, newVerifierKeyHash, newApprovedMerkleRoot | Admin updates circuit parameters |

### Safety Properties

**S1 — Prover must sign.** `checkSig(proverSig, proverPk)` (line 41). Every proof verification requires the prover to authenticate.

**S2 — Proof binding prevents substitution.** `expectedBinding = sha256(proofCommitment + verifierKeyHash + approvedMerkleRoot + nullifierHash)` (lines 47-49). The binding is required non-zero (line 52). A proof generated for a different verifier key, Merkle root, or nullifier is rejected.

**S3 — Nullifier binding prevents replay.** `nullifierBinding = sha256(nullifierHash + proverPkHash)` (line 57), required non-zero (line 58). This ties the nullifier to the prover's identity, preventing cross-identity nullifier reuse.

**S4 — Admin key hash commitment.** `blake2b(adminPk) == adminKeyHash` (line 80) ensures only the committed admin can update the verifier key.

### Liveness Properties

**L1 — Verification service persists.** Both paths preserve the contract via `validateOutputState` (lines 65-69, 89-93). The verifier UTXO is never consumed, only cycled.

### Covenant Preservation

**CP1 — verifyProof preserves all state.** `validateOutputState(0, {verifierKeyHash, approvedMerkleRoot, adminKeyHash})` (lines 65-69). All parameters are carried forward unchanged.

**CP2 — updateVerifierKey updates selectively.** `validateOutputState(0, {newVerifierKeyHash, newApprovedMerkleRoot, adminKeyHash})` (lines 89-93). The `adminKeyHash` is preserved; only the verifier key and Merkle root are updated.

### Value Conservation

**VC1 — verifyProof:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 62).

**VC2 — updateVerifierKey:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 86).

---

## 5. Reserves (`reserves.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `withdraw` (line 32) | withdrawerPk, withdrawerSig, oraclePk, oracleAttestation, attestedReserveValue, attestedTokenSupplyValue, recipientPk | Oracle-attested reserve withdrawal |
| `deposit` (line 81) | (none) | Permissionless collateral deposit |
| `custodianReclaim` (line 97) | custodianPk, custodianSig | Emergency reclaim |

### Safety Properties

**S1 — Withdrawal requires dual authorization.** Both the withdrawer (`checkSig(withdrawerSig, withdrawerPk)` line 42) and the oracle (`blake2b(oraclePk) == oracleKeyHash` line 45, `checkSig(oracleAttestation, oraclePk)` line 46) must sign. Neither party alone can authorize a withdrawal.

**S2 — Reserve ratio enforcement.** Lines 56-61: `require(attestedTokenSupplyValue > 0)`, `require(attestedReserveValue >= withdrawAmount)`, and `require(remainingReserve * RATIO_DENOMINATOR >= minReserveRatio * attestedTokenSupplyValue)`. The contract mathematically enforces that the post-withdrawal reserve ratio remains above the minimum threshold set at deployment.

**S3 — Custodian key hash binding.** `blake2b(custodianPk) == custodianKeyHash` (line 98) for emergency reclaim.

### Liveness Properties

**L1 — Deposits are always possible.** The `deposit` path (line 81) requires no signature. Anyone can add collateral at any time, increasing the reserve.

**L2 — Custodian emergency exit.** `custodianReclaim` (line 97) provides an unconditional reclaim path with only custodian signature, usable for migration or emergency.

### Covenant Preservation

**CP1 — withdraw preserves covenant on change.** If `changeValue > MINER_FEE`, `validateOutputState(1, {oracleKeyHash, custodianKeyHash, minReserveRatio})` (lines 71-76). The reserve ratio threshold is immutable across withdrawals.

**CP2 — deposit preserves covenant.** `validateOutputState(0, {oracleKeyHash, custodianKeyHash, minReserveRatio})` (lines 88-92). The output value must be >= input value (line 85), enforcing that deposits can only increase the reserve.

**CP3 — custodianReclaim breaks covenant.** No `validateOutputState`. This is the exit path.

### Value Conservation

**VC1 — withdraw:** Output 0 receives `withdrawAmount`, output 1 (if change exists) receives `changeValue = inputValue - withdrawAmount - MINER_FEE` (line 51). The ratio check at lines 59-61 is the economic safety bound.

**VC2 — deposit:** `require(tx.outputs[0].value >= inputValue)` (line 85). Deposits can only increase value; no fee is charged.

**VC3 — custodianReclaim:** No value constraint. Full withdrawal to custodian.

---

## 6. HTLC (`htlc.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `claimWithPreimage` (line 30) | preimage, recipientPk, recipientSig | Recipient claims by revealing SHA-256 preimage |
| `refundAfterTimeout` (line 48) | senderPk, senderSig | Sender reclaims after timelock expiry |

### Safety Properties

**S1 — Hashlock binding.** `require(sha256(preimage) == hashLock)` (line 36). Only the holder of the 32-byte preimage can claim. The SHA-256 preimage resistance (2^128 security) prevents brute-force claims.

**S2 — Recipient identity verification.** `blake2b(recipientPk) == recipientKeyHash` (line 39) and `checkSig(recipientSig, recipientPk)` (line 40). Even with the preimage, only the designated recipient can claim. This prevents front-running by a miner who observes the preimage in the mempool.

**S3 — Sender identity for refund.** `blake2b(senderPk) == senderKeyHash` (line 53) and `checkSig(senderSig, senderPk)` (line 54).

### Liveness Properties

**L1 — Funds are always recoverable.** If the recipient claims before timeout, funds flow via `claimWithPreimage`. If not, the sender can reclaim via `refundAfterTimeout`. One path is always available.

**NOTE — Missing timelock enforcement.** The contract defines `timeLock` state (line 18) but neither path checks it against a block timestamp or DAA score. The `refundAfterTimeout` path is callable at any time. This is a specification gap: in production, `refundAfterTimeout` should include `require(tx.lockTime >= timeLock)` or equivalent.

### Covenant Preservation

This contract does NOT preserve a covenant. Both paths produce plain P2PK outputs. The HTLC is consumed upon use. This is correct behavior: HTLCs are one-shot atomic swap primitives.

### Value Conservation

**VC1 — claimWithPreimage:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 44).

**VC2 — refundAfterTimeout:** `require(tx.outputs[0].value >= inputValue - MINER_FEE)` (line 58).

---

## 7. Dividend (`dividend.sil`)

### Spending Paths

| Entrypoint | Parameters | Purpose |
|---|---|---|
| `claimDividend` (line 30) | holderPk, holderSig, holderShares, merkleProof0, merkleProof1 | Holder claims proportional share |
| `issuerTopUp` (line 70) | issuerPk, issuerSig | Issuer adds more distribution funds |
| `issuerReclaim` (line 86) | issuerPk, issuerSig | Issuer reclaims unclaimed funds |

### Safety Properties

**S1 — Merkle proof membership.** Lines 43-46: The contract computes `leaf = sha256(holderPk)`, then `intermediate = sha256(leaf + merkleProof0)`, then `computedRoot = sha256(intermediate + merkleProof1)`, and requires `computedRoot == holderMerkleRoot`. Only holders in the committed Merkle tree can claim. This is a 2-level proof (4 leaves max); production would need a deeper tree.

**S2 — Proportional claim amount.** `claimAmount = (inputValue * holderShares) / totalShares` (line 50). The claim is capped by the holder's declared share count, which must match the Merkle tree entry. `require(claimAmount > MINER_FEE)` (line 51) prevents dust claims.

**S3 — Issuer identity binding.** `blake2b(issuerPk) == issuerKeyHash` (lines 71, 87) and `checkSig(issuerSig, issuerPk)` (lines 72, 88) for both top-up and reclaim.

**NOTE — Double-claim vulnerability.** The Merkle root is preserved across claims (line 60-65). There is no nullifier mechanism; the same holder can claim repeatedly from the shrinking pool. In production, the Merkle root should be updated after each claim (removing the claimed leaf) or a separate nullifier set should be maintained.

### Liveness Properties

**L1 — Holders can claim any time before reclaim.** No deadline enforcement in `claimDividend`. The `deadline` state variable exists but is not checked.

**L2 — Issuer can top up.** `issuerTopUp` requires `tx.outputs[0].value >= inputValue` (line 76), allowing the issuer to add funds while preserving the covenant.

**L3 — Issuer can reclaim unclaimed funds.** `issuerReclaim` (line 86) is unconditional (no deadline check). In production, `require(tx.lockTime >= deadline)` should be enforced.

### Covenant Preservation

**CP1 — claimDividend preserves covenant on remainder.** If `remaining > MINER_FEE`, `validateOutputState(1, {holderMerkleRoot, issuerKeyHash, totalShares, deadline})` (lines 60-65). All state is preserved, including `totalShares` (which should arguably decrease after a claim).

**CP2 — issuerTopUp preserves covenant.** `validateOutputState(0, {holderMerkleRoot, issuerKeyHash, totalShares, deadline})` (lines 77-82).

**CP3 — issuerReclaim breaks covenant.** No `validateOutputState`. Exit path for the issuer.

### Value Conservation

**VC1 — claimDividend:** Output 0 receives `>= claimAmount - MINER_FEE` (line 54). Output 1 (if remainder exists) receives `>= remaining - MINER_FEE` (line 59). Total outputs approximate `inputValue - 2 * MINER_FEE` in the worst case.

**VC2 — issuerTopUp:** `require(tx.outputs[0].value >= inputValue)` (line 76). Value can only increase.

**VC3 — issuerReclaim:** No value constraint. Full reclaim.

---

## Cross-Contract Invariants

**INV1 — Covenant continuity.** All persistent contracts (RwaCore, StateVerity, ZkKycVerifier, Reserves, Dividend) use `validateOutputState` to enforce that the output script matches the contract bytecode with the appropriate state fields. One-shot contracts (Clawback, HTLC) intentionally do not.

**INV2 — Key hash commitment.** All admin/issuer/manager paths use `blake2b(pk) == keyHash` before `checkSig(sig, pk)`. This two-step verification prevents an attacker from substituting a different public key that could produce a valid signature for a different message.

**INV3 — MINER_FEE constant.** All contracts define `MINER_FEE = 1000` sompis. Value conservation checks use `>=` (not `==`), meaning the actual fee may be less than 1000 but the contract allows up to 1000 sompis to be consumed as fees.

**INV4 — No reentrancy.** SilverScript contracts execute atomically within a single transaction. There is no external call mechanism, so reentrancy is not possible. Each entrypoint checks the transaction structure and either succeeds or fails entirely.
