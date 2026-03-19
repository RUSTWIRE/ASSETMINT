// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

const COMPLIANCE_API =
  process.env.NEXT_PUBLIC_COMPLIANCE_API || "http://localhost:3001";
const ORACLE_API =
  process.env.NEXT_PUBLIC_ORACLE_API || "http://localhost:3002";

export const api = {
  // Identity
  registerIdentity: (did: string, primaryKey: string) =>
    fetch(`${COMPLIANCE_API}/identity`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ did, primary_key: primaryKey }),
    }).then((r) => r.json()),

  // Claims
  issueClaim: (
    subjectDid: string,
    claimType: string,
    expiry: number,
    jurisdiction?: string
  ) =>
    fetch(`${COMPLIANCE_API}/claim`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        subject_did: subjectDid,
        claim_type: claimType,
        expiry,
        jurisdiction,
      }),
    }).then((r) => r.json()),

  // Compliance
  evaluateTransfer: (
    senderDid: string,
    receiverDid: string,
    assetId: string,
    amount: number,
    mintTimestamp?: number
  ) =>
    fetch(
      `${COMPLIANCE_API}/compliance/evaluate?sender_did=${senderDid}&receiver_did=${receiverDid}&asset_id=${assetId}&amount=${amount}&mint_timestamp=${mintTimestamp || 0}`
    ).then((r) => r.json()),

  // Merkle
  getMerkleRoot: () =>
    fetch(`${COMPLIANCE_API}/merkle-root`).then((r) => r.json()),

  // Health
  health: () => fetch(`${COMPLIANCE_API}/health`).then((r) => r.json()),

  // Kaspa Network (live from kaspad via compliance API)
  networkInfo: () =>
    fetch(`${COMPLIANCE_API}/network`).then((r) => r.json()),

  // Balance (live from kaspad)
  getBalance: (address: string) =>
    fetch(`${COMPLIANCE_API}/balance?address=${address}`).then((r) => r.json()),

  // Compliance-gated on-chain transfer
  complianceTransfer: (
    senderDid: string,
    receiverDid: string,
    senderPrivateKey: string,
    receiverAddress: string,
    amountSompis: number,
    assetId: string
  ) =>
    fetch(`${COMPLIANCE_API}/transfer`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sender_did: senderDid,
        receiver_did: receiverDid,
        sender_private_key: senderPrivateKey,
        receiver_address: receiverAddress,
        amount_sompis: amountSompis,
        asset_id: assetId,
      }),
    }).then((r) => r.json()),

  // Oracle
  oracleHealth: () => fetch(`${ORACLE_API}/health`).then((r) => r.json()),
};

// Kaspa Testnet-12 block explorer
const EXPLORER_BASE = "https://explorer-tn12.kaspa.org";

export const explorer = {
  txUrl: (txId: string) => `${EXPLORER_BASE}/txs/${txId}`,
  addressUrl: (address: string) => `${EXPLORER_BASE}/addresses/${address}`,
  blockUrl: (hash: string) => `${EXPLORER_BASE}/blocks/${hash}`,
};

export { COMPLIANCE_API, ORACLE_API, EXPLORER_BASE };
