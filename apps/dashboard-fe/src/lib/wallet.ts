// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

export interface KaspaWallet {
  address: string;
  did: string;
  privateKey: string; // hex-encoded — NEVER expose in production
  balance: number; // sompis
  connected: boolean;
}

// Simulated testnet wallet — REPLACE_WITH_TESTNET_WALLET
export function createTestWallet(): KaspaWallet {
  console.log("[K-RWA] Creating simulated testnet wallet");
  return {
    address: "kaspatest:qr35ennsep3hxfe7lnz5ee7j5jgmkjswss2dxxl7n",
    did: "did:kaspa:qr35ennsep3hxfe7lnz5ee7j5jgmkjswss2dxxl7n",
    privateKey: "b7e151628aed2a6abf7158809cf4f3c762e7160f38b4da56a784d9045190cfef",
    balance: 100_000_000_000, // 1000 KAS
    connected: true,
  };
}

export function formatKAS(sompis: number): string {
  return (sompis / 100_000_000).toFixed(2);
}

export function truncateAddress(address: string): string {
  if (address.length <= 20) return address;
  return `${address.slice(0, 14)}...${address.slice(-6)}`;
}
