// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { Wallet, LogOut } from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { formatKAS, truncateAddress } from "@/lib/wallet";

export function WalletButton() {
  const { wallet, connect, disconnect } = useWalletStore();

  if (wallet?.connected) {
    return (
      <div className="flex items-center gap-3">
        <div className="text-right">
          <p className="text-sm font-medium text-white">
            {formatKAS(wallet.balance)} KAS
          </p>
          <p className="text-xs text-gray-400">
            {truncateAddress(wallet.address)}
          </p>
        </div>
        <button
          onClick={disconnect}
          className="p-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
          aria-label="Disconnect wallet"
        >
          <LogOut className="w-4 h-4" />
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={connect}
      className="flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors"
    >
      <Wallet className="w-4 h-4" />
      Connect Wallet
    </button>
  );
}
