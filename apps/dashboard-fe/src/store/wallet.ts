// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

import { create } from "zustand";
import { KaspaWallet, createTestWallet } from "@/lib/wallet";

interface WalletStore {
  wallet: KaspaWallet | null;
  connect: () => void;
  disconnect: () => void;
}

export const useWalletStore = create<WalletStore>((set) => ({
  wallet: null,
  connect: () => {
    console.log("[K-RWA] Connecting simulated testnet wallet");
    set({ wallet: createTestWallet() });
  },
  disconnect: () => {
    console.log("[K-RWA] Disconnecting wallet");
    set({ wallet: null });
  },
}));
