// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// KTT (Kaspa Trust Token) integration for AssetMint.
// Uses the KTT covenant-native token standard instead of KRC-20 (which uses OP_RETURN
// and is rejected by Kaspa nodes). KTT enforces compliance at consensus level via
// SilverScript covenants with KIP-10 transaction introspection.
//
// Live deployed tokens on TN12:
//   KDINO: e346df79...
//   KRPG:  4b0588cb...
//   KEUR:  cca572f1...
//
// Registry: 7f37d25f...

/** Deployed KTT tokens on Kaspa Testnet-12 */
export const KTT_TOKENS = {
  KDINO: {
    ticker: "KDINO",
    txId: "e346df796883e28981f9da4beaa0eaca07f1423179ba501aa0b254aaea1f3efc",
    standard: "KTT-UT", // Unitary Token
  },
  KRPG: {
    ticker: "KRPG",
    txId: "4b0588cb1d22e2ab607a5b7d88a1ce19bce6c8e583f35cb48513a819478f2f5e",
    standard: "KTT-MT", // Multi Token
  },
  KEUR: {
    ticker: "KEUR",
    txId: "cca572f10007adf2141d193d3ec96c4b007e635963778b4e458eca08bd51eda8",
    standard: "KTT-UT", // Unitary Token (stablecoin)
  },
} as const;

/** KTT Trust Registry on TN12 */
export const KTT_REGISTRY = {
  txId: "7f37d25f9b534d5c74e8ccf05a763397287f3c398ff650eda4c33fd379a2f280",
  scriptHex:
    "aa20a82e6780e63b2ec0dafab6c4b17cf018f35f45606ce6b02be89686c95645513787",
};

/** Number of confirmed KTT transfer transactions on TN12 */
export const KTT_TRANSFER_COUNT = 12;

export type KttTicker = keyof typeof KTT_TOKENS;
