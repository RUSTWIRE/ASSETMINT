// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  Wallet,
  ShieldCheck,
  TrendingUp,
  ArrowUpRight,
  ArrowDownLeft,
  Activity,
  FileCode2,
  ExternalLink,
  UserPlus,
  ClipboardCheck,
  FileCode,
  CheckCircle2,
  XCircle,
  Loader2,
  AlertTriangle,
  Send,
} from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { formatKAS } from "@/lib/wallet";
import { api, explorer } from "@/lib/api";
import { DEPLOYED_CONTRACTS } from "@/lib/contracts";

interface HealthStatus {
  status: string;
  service: string;
  kaspa_connected: boolean;
}

interface NetworkInfo {
  server_version: string;
  is_synced: boolean;
  virtual_daa_score: number;
  network_id: string;
  block_count: number;
  difficulty: number;
}

// Real confirmed transactions from Kaspa Testnet-12
// Deploy TX IDs sourced from DEPLOYED_CONTRACTS (full hashes for valid explorer links)
const REAL_TRANSACTIONS: Array<{
  id: string;
  type: "Transfer" | "Deploy";
  from?: string;
  to?: string;
  amount?: string;
  contract?: string;
  status: "confirmed";
}> = [
  { id: "a48b2c4bba2f4d0fcf54f7ffeed8f9e10bb4e24c25ef09ab09d60b8ab1e96b0b", type: "Transfer", from: "Alice", to: "Bob", amount: "0.1 KAS", status: "confirmed" },
  { id: "dfc0e9594eac42c9d32fca8a9e27b4d65d99d2de88768f96e1dc8a4f5ff63eb2", type: "Transfer", from: "Alice", to: "Bob", amount: "0.1 KAS", status: "confirmed" },
  { id: "f4489bd4e76c8c10137bb31c6c04d4c6f6d22c20e4ccaa08c72f4eb96ddc9e76", type: "Transfer", from: "Bob", to: "Alice", amount: "0.05 KAS", status: "confirmed" },
  { id: DEPLOYED_CONTRACTS.clawback.txId, type: "Deploy", contract: "Clawback", status: "confirmed" },
  { id: DEPLOYED_CONTRACTS.rwaCore.txId, type: "Deploy", contract: "RwaCore", status: "confirmed" },
  { id: DEPLOYED_CONTRACTS.stateVerity.txId, type: "Deploy", contract: "StateVerity", status: "confirmed" },
  { id: DEPLOYED_CONTRACTS.zkKycVerifier.txId, type: "Deploy", contract: "ZkKycVerifier", status: "confirmed" },
  { id: DEPLOYED_CONTRACTS.reserves.txId, type: "Deploy", contract: "Reserves", status: "confirmed" },
  { id: "5139f1fdda2ff841107730bce5ead83c29635b2484ddff33697898725e201e70", type: "Deploy", contract: "CHECKSIG Covenant", status: "confirmed" },
  { id: "ccfdab27756445ce5fa565cbf98efc34bb5487a3f07b4e529f63ee9955dc7775", type: "Transfer", from: "Covenant", to: "Recipient", amount: "0.9999 KAS", status: "confirmed" },
  { id: "6c1fee2b7387cadd777a5af8b62144f2c7dfc712a1eb463a9134594b6a2e429f", type: "Deploy", contract: "Compliance Covenant", status: "confirmed" },
  { id: "d0bcf48c8e879ee9d72a40ebe4424e671389848bcea5cd9c5ed1bafe0f392a56", type: "Transfer", from: "Compliance", to: "Recipient", amount: "0.9999 KAS", status: "confirmed" },
];

const CONTRACTS = Object.values(DEPLOYED_CONTRACTS);
const RULE_COUNT = 5; // KYC, accreditation, jurisdiction, lock-up, max-amount

export default function DashboardPage() {
  const { wallet } = useWalletStore();
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [healthError, setHealthError] = useState(false);
  const [network, setNetwork] = useState<NetworkInfo | null>(null);
  const [metadataStatus, setMetadataStatus] = useState<"checking" | "online" | "offline">("checking");
  const [oracleStatus, setOracleStatus] = useState<"checking" | "online" | "offline">("checking");

  useEffect(() => {
    console.log("[K-RWA] Dashboard mounted, fetching health + network status");
    api
      .health()
      .then((data) => {
        console.log("[K-RWA] Health response:", data);
        setHealth(data);
      })
      .catch(() => {
        console.log("[K-RWA] Compliance backend not reachable");
        setHealthError(true);
      });

    api
      .networkInfo()
      .then((data) => {
        console.log("[K-RWA] Network info:", data);
        setNetwork(data);
      })
      .catch(() => {
        console.log("[K-RWA] Network info not available");
      });

    fetch("http://localhost:8900/health")
      .then((r) => {
        if (r.ok) setMetadataStatus("online");
        else setMetadataStatus("offline");
      })
      .catch(() => setMetadataStatus("offline"));

    api
      .oracleHealth()
      .then(() => setOracleStatus("online"))
      .catch(() => setOracleStatus("offline"));
  }, []);

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-white">Portfolio Overview</h2>
        <p className="text-gray-400 text-sm mt-1">
          AssetMint dashboard -- Kaspa Testnet-12
        </p>
      </div>

      {/* System Status */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-4 flex items-center gap-3">
          {health ? (
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
          ) : healthError ? (
            <XCircle className="w-5 h-5 text-red-400 shrink-0" />
          ) : (
            <Loader2 className="w-5 h-5 text-gray-400 animate-spin shrink-0" />
          )}
          <div className="min-w-0">
            <p className="text-xs text-gray-500">Backend API</p>
            <p className={`text-sm font-medium truncate ${health ? "text-emerald-400" : healthError ? "text-red-400" : "text-gray-400"}`}>
              {health ? "Connected" : healthError ? "Offline" : "Checking..."}
            </p>
          </div>
        </div>

        <div className="bg-gray-900 rounded-xl border border-gray-800 p-4 flex items-center gap-3">
          {network?.is_synced ? (
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
          ) : network ? (
            <XCircle className="w-5 h-5 text-red-400 shrink-0" />
          ) : (
            <Loader2 className="w-5 h-5 text-gray-400 animate-spin shrink-0" />
          )}
          <div className="min-w-0">
            <p className="text-xs text-gray-500">Kaspa TN12</p>
            <p className={`text-sm font-medium truncate ${network?.is_synced ? "text-emerald-400" : network ? "text-red-400" : "text-gray-400"}`}>
              {network?.is_synced
                ? `Synced (${network.block_count.toLocaleString()} blocks)`
                : network
                  ? "Not synced"
                  : "Checking..."}
            </p>
          </div>
        </div>

        <div className="bg-gray-900 rounded-xl border border-gray-800 p-4 flex items-center gap-3">
          {metadataStatus === "online" ? (
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
          ) : metadataStatus === "offline" ? (
            <XCircle className="w-5 h-5 text-red-400 shrink-0" />
          ) : (
            <Loader2 className="w-5 h-5 text-gray-400 animate-spin shrink-0" />
          )}
          <div className="min-w-0">
            <p className="text-xs text-gray-500">Sovereign Metadata</p>
            <p className={`text-sm font-medium truncate ${metadataStatus === "online" ? "text-emerald-400" : metadataStatus === "offline" ? "text-red-400" : "text-gray-400"}`}>
              {metadataStatus === "online" ? "Online (port 8900)" : metadataStatus === "offline" ? "Offline" : "Checking..."}
            </p>
          </div>
        </div>

        <div className="bg-gray-900 rounded-xl border border-gray-800 p-4 flex items-center gap-3">
          <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0" />
          <div className="min-w-0">
            <p className="text-xs text-gray-500">Oracle</p>
            <p className="text-sm font-medium text-amber-400 truncate">
              CoinGecko (simulated)
            </p>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Link
          href="/transfer"
          className="flex items-center gap-2 bg-gray-900 border border-gray-800 rounded-lg px-4 py-3 text-sm text-gray-300 hover:border-indigo-500/50 hover:text-white transition-colors"
        >
          <UserPlus className="w-4 h-4 text-indigo-400" />
          Register Identity
        </Link>
        <Link
          href="/transfer"
          className="flex items-center gap-2 bg-gray-900 border border-gray-800 rounded-lg px-4 py-3 text-sm text-gray-300 hover:border-indigo-500/50 hover:text-white transition-colors"
        >
          <ClipboardCheck className="w-4 h-4 text-amber-400" />
          Issue KYC
        </Link>
        <Link
          href="/transfer"
          className="flex items-center gap-2 bg-gray-900 border border-gray-800 rounded-lg px-4 py-3 text-sm text-gray-300 hover:border-indigo-500/50 hover:text-white transition-colors"
        >
          <Send className="w-4 h-4 text-emerald-400" />
          Transfer Asset
        </Link>
        <a
          href="#contracts"
          className="flex items-center gap-2 bg-gray-900 border border-gray-800 rounded-lg px-4 py-3 text-sm text-gray-300 hover:border-indigo-500/50 hover:text-white transition-colors"
        >
          <FileCode className="w-4 h-4 text-purple-400" />
          View Contracts
        </a>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* Wallet Balance */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">KAS Balance</span>
            <Wallet className="w-5 h-5 text-indigo-400" />
          </div>
          <p className="text-2xl font-bold text-white">
            {wallet ? `${formatKAS(wallet.balance)} KAS` : "--"}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {wallet ? "Testnet tokens" : "Connect wallet"}
          </p>
        </div>

        {/* ASTM Balance */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">ASTM Balance</span>
            <TrendingUp className="w-5 h-5 text-gray-600" />
          </div>
          <p className="text-2xl font-bold text-gray-500">Not deployed</p>
          <p className="text-xs text-gray-600 mt-1">
            KRC-20 token not yet broadcast
          </p>
        </div>

        {/* Compliance Engine */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">Compliance Engine</span>
            <ShieldCheck className="w-5 h-5 text-amber-400" />
          </div>
          <p className="text-2xl font-bold text-white">
            {health ? `${RULE_COUNT} rules` : healthError ? "Offline" : "Checking..."}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {health
              ? `Kaspa: ${health.kaspa_connected ? "Connected" : "Disconnected"}`
              : healthError
                ? "Backend not reachable"
                : "Connecting..."}
          </p>
        </div>

        {/* Live Network Stats */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">Kaspa Network</span>
            <Activity className="w-5 h-5 text-purple-400" />
          </div>
          <p className="text-2xl font-bold text-white">
            {network ? network.network_id : "--"}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {network
              ? `v${network.server_version} | ${network.block_count.toLocaleString()} blocks | DAA ${network.virtual_daa_score.toLocaleString()}`
              : "Fetching..."}
          </p>
        </div>
      </div>

      {/* Deployed Contracts */}
      <div id="contracts" className="bg-gray-900 rounded-xl border border-gray-800">
        <div className="px-6 py-4 border-b border-gray-800">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <FileCode2 className="w-5 h-5 text-indigo-400" />
            Deployed Contracts
          </h3>
          <p className="text-xs text-gray-500">
            {CONTRACTS.length} SilverScript covenants on Testnet-12
          </p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full" role="table">
            <thead>
              <tr className="text-left text-xs text-gray-500 border-b border-gray-800">
                <th className="px-6 py-3 font-medium">Contract</th>
                <th className="px-6 py-3 font-medium">Script Size</th>
                <th className="px-6 py-3 font-medium">P2SH Address</th>
                <th className="px-6 py-3 font-medium">TX</th>
              </tr>
            </thead>
            <tbody>
              {CONTRACTS.map((c) => (
                <tr
                  key={c.name}
                  className="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="px-6 py-4 text-sm text-white font-medium">
                    {c.name}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-300 font-mono">
                    {c.scriptSize} bytes
                  </td>
                  <td className="px-6 py-4">
                    <a
                      href={explorer.addressUrl(c.p2shAddress)}
                      target="_blank"
                      rel="noopener"
                      className="text-xs text-indigo-400 hover:text-indigo-300 font-mono"
                    >
                      {c.p2shAddress.slice(0, 18)}...{c.p2shAddress.slice(-8)}
                    </a>
                  </td>
                  <td className="px-6 py-4">
                    <a
                      href={explorer.txUrl(c.txId)}
                      target="_blank"
                      rel="noopener"
                      className="text-xs text-blue-400 hover:text-blue-300 font-mono flex items-center gap-1"
                    >
                      {c.txId.slice(0, 10)}...
                      <ExternalLink className="w-3 h-3" />
                    </a>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Recent Transactions */}
      <div className="bg-gray-900 rounded-xl border border-gray-800">
        <div className="px-6 py-4 border-b border-gray-800">
          <h3 className="text-lg font-semibold text-white">
            Recent Transactions
          </h3>
          <p className="text-xs text-gray-500">Confirmed on Kaspa Testnet-12</p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full" role="table">
            <thead>
              <tr className="text-left text-xs text-gray-500 border-b border-gray-800">
                <th className="px-6 py-3 font-medium">Type</th>
                <th className="px-6 py-3 font-medium">Details</th>
                <th className="px-6 py-3 font-medium">Status</th>
                <th className="px-6 py-3 font-medium">Explorer</th>
              </tr>
            </thead>
            <tbody>
              {REAL_TRANSACTIONS.map((tx) => (
                <tr
                  key={tx.id}
                  className="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      {tx.type === "Deploy" ? (
                        <ArrowDownLeft className="w-4 h-4 text-emerald-400" />
                      ) : (
                        <ArrowUpRight className="w-4 h-4 text-blue-400" />
                      )}
                      <span className="text-sm text-gray-300">
                        {tx.type}
                      </span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-300">
                    {tx.type === "Transfer"
                      ? `${tx.from} → ${tx.to} (${tx.amount})`
                      : `Contract: ${tx.contract}`}
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-xs px-2 py-1 rounded-full font-medium bg-emerald-500/10 text-emerald-400">
                      {tx.status}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <a
                      href={explorer.txUrl(tx.id)}
                      target="_blank"
                      rel="noopener"
                      className="text-xs text-blue-400 hover:text-blue-300 font-mono flex items-center gap-1"
                    >
                      {tx.id.slice(0, 8)}...
                      <ExternalLink className="w-3 h-3" />
                    </a>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
