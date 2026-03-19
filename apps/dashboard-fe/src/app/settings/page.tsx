// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useEffect, useState } from "react";
import { Settings, Server, Wallet, Globe, Info } from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { COMPLIANCE_API, ORACLE_API, api } from "@/lib/api";

interface NetworkInfo {
  server_version: string;
  is_synced: boolean;
  virtual_daa_score: number;
  network_id: string;
  block_count: number;
  difficulty: number;
}

export default function SettingsPage() {
  const { wallet } = useWalletStore();
  const [network, setNetwork] = useState<NetworkInfo | null>(null);

  useEffect(() => {
    api.networkInfo().then(setNetwork).catch(() => {});
  }, []);

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-white">Settings</h2>
        <p className="text-gray-400 text-sm mt-1">
          Configuration and network information
        </p>
      </div>

      {/* API Endpoints */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Server className="w-5 h-5 text-indigo-400" />
          API Endpoints
        </h3>
        <div className="space-y-3">
          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-400">Compliance Engine (Rust)</p>
                <p className="text-sm text-white font-mono mt-1">
                  {COMPLIANCE_API}
                </p>
              </div>
              <span className="text-xs px-2 py-1 rounded-full bg-gray-700 text-gray-400">
                Port 3001
              </span>
            </div>
            <p className="text-xs text-gray-600 mt-2">
              Identity registry, claims, compliance evaluation, Merkle proofs
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-400">Oracle Pool</p>
                <p className="text-sm text-white font-mono mt-1">
                  {ORACLE_API}
                </p>
              </div>
              <span className="text-xs px-2 py-1 rounded-full bg-gray-700 text-gray-400">
                Port 3002
              </span>
            </div>
            <p className="text-xs text-gray-600 mt-2">
              Price feeds, reserve attestations, collateral ratio verification
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-400">Sovereign Metadata</p>
                <p className="text-sm text-white font-mono mt-1">
                  http://localhost:8900
                </p>
              </div>
              <span className="text-xs px-2 py-1 rounded-full bg-gray-700 text-gray-400">
                Port 8900
              </span>
            </div>
            <p className="text-xs text-gray-600 mt-2">
              Private metadata store with SHA-256 integrity hashes and tamper detection
            </p>
          </div>
        </div>
      </div>

      {/* Wallet Info */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Wallet className="w-5 h-5 text-emerald-400" />
          Wallet Information
        </h3>
        {wallet ? (
          <div className="space-y-3">
            <div className="bg-gray-800 rounded-lg p-4">
              <p className="text-xs text-gray-500">Address</p>
              <p className="text-sm text-white font-mono break-all mt-1">
                {wallet.address}
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-4">
              <p className="text-xs text-gray-500">DID</p>
              <p className="text-sm text-white font-mono break-all mt-1">
                {wallet.did}
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-4">
              <p className="text-xs text-gray-500">Balance</p>
              <p className="text-sm text-white mt-1">
                {(wallet.balance / 100_000_000).toFixed(2)} KAS (
                {wallet.balance.toLocaleString()} sompis)
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-4">
              <p className="text-xs text-gray-500">Status</p>
              <p className="text-sm text-emerald-400 mt-1">
                Connected (simulated)
              </p>
            </div>
          </div>
        ) : (
          <div className="bg-gray-800 rounded-lg p-4">
            <p className="text-sm text-gray-400">
              No wallet connected. Use the connect button in the header.
            </p>
          </div>
        )}
      </div>

      {/* Network Info */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Globe className="w-5 h-5 text-purple-400" />
          Network Information
        </h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Network</span>
            <span className="text-sm text-white font-medium">
              {network ? network.network_id : "Kaspa Testnet-12"}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">kaspad Version</span>
            <span className="text-sm text-emerald-400 font-mono">
              {network ? `v${network.server_version} (live)` : "Checking..."}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Block Count</span>
            <span className="text-sm text-white font-mono">
              {network ? network.block_count.toLocaleString() : "--"}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">DAA Score</span>
            <span className="text-sm text-white font-mono">
              {network ? network.virtual_daa_score.toLocaleString() : "--"}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Difficulty</span>
            <span className="text-sm text-white font-mono">
              {network ? network.difficulty.toFixed(2) : "--"}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Synced</span>
            <span className={`text-sm font-medium ${network?.is_synced ? "text-emerald-400" : "text-amber-400"}`}>
              {network ? (network.is_synced ? "Yes" : "Syncing...") : "--"}
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">wRPC Endpoint</span>
            <span className="text-sm text-white font-mono">
              ws://127.0.0.1:17210
            </span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Block Time</span>
            <span className="text-sm text-white">~1 second (BlockDAG)</span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Consensus</span>
            <span className="text-sm text-white">GHOSTDAG / PHANTOM</span>
          </div>
          <div className="flex items-center justify-between py-3 border-b border-gray-800">
            <span className="text-sm text-gray-400">Token Standard</span>
            <span className="text-sm text-white">KTT (Covenant-Native)</span>
          </div>
          <div className="flex items-center justify-between py-3">
            <span className="text-sm text-gray-400">Environment</span>
            <span className="text-sm text-amber-400">
              Demo / Testnet Only
            </span>
          </div>
        </div>
      </div>

      {/* About */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        <div className="flex items-start gap-3">
          <Info className="w-5 h-5 text-gray-500 shrink-0 mt-0.5" />
          <div>
            <h3 className="text-sm font-semibold text-white">
              About AssetMint
            </h3>
            <p className="text-xs text-gray-400 mt-2 leading-relaxed">
              AssetMint is an institutional-grade RWA tokenization platform
              built on Kaspa. It combines UTXO covenants, ZK-KYC compliance
              proofs, sovereign metadata, and KRC-20 tokens to enable
              compliant tokenization of real-world assets. This dashboard is a
              technical demo running on Testnet-12 with simulated data. All
              assets (KPROP-NYC-TEST) and tokens (ASTM) are fictional.
            </p>
            <p className="text-xs text-gray-600 mt-2">
              Version 0.1.0 | MIT License | Technical Demo
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
