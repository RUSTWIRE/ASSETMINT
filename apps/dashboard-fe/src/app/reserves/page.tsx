// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useEffect, useState } from "react";
import {
  Vault,
  ShieldCheck,
  TrendingUp,
  ExternalLink,
  Activity,
  RefreshCw,
} from "lucide-react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import { api, explorer } from "@/lib/api";
import { DEPLOYED_CONTRACTS } from "@/lib/contracts";
import { DemoBanner } from "@/components/demo-banner";

const reservesContract = DEPLOYED_CONTRACTS.reserves;

const COLLATERAL_DATA = [
  { month: "Oct", ratio: 105 },
  { month: "Nov", ratio: 108 },
  { month: "Dec", ratio: 112 },
  { month: "Jan", ratio: 110 },
  { month: "Feb", ratio: 115 },
  { month: "Mar", ratio: 118 },
];

const RESERVE_BREAKDOWN = [
  { name: "Real Estate (appraised)", value: 750000, color: "#6366f1" },
  { name: "Cash Escrow (USD)", value: 150000, color: "#10b981" },
  { name: "Insurance Bond", value: 80000, color: "#f59e0b" },
  { name: "KAS Collateral", value: 20000, color: "#8b5cf6" },
];

const ATTESTATION_HISTORY = [
  {
    id: "att-001",
    date: "2026-03-18",
    oracle: "oracle-pool:3002",
    ratio: "118%",
    status: "verified",
  },
  {
    id: "att-002",
    date: "2026-03-11",
    oracle: "oracle-pool:3002",
    ratio: "115%",
    status: "verified",
  },
  {
    id: "att-003",
    date: "2026-03-04",
    oracle: "oracle-pool:3002",
    ratio: "112%",
    status: "verified",
  },
  {
    id: "att-004",
    date: "2026-02-25",
    oracle: "oracle-pool:3002",
    ratio: "110%",
    status: "verified",
  },
];

interface OracleAttestation {
  asset_id: string;
  price_usd: number;
  timestamp: number;
  source: string;
  signature?: string;
}

export default function ReservesPage() {
  const [oracleOnline, setOracleOnline] = useState<boolean | null>(null);
  const [attestation, setAttestation] = useState<OracleAttestation | null>(null);
  const [attestationError, setAttestationError] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const fetchAttestation = () => {
    setRefreshing(true);
    setAttestationError(false);
    api
      .oracleAttestation("KAS")
      .then((data: OracleAttestation) => {
        setAttestation(data);
        setOracleOnline(true);
        setAttestationError(false);
      })
      .catch(() => {
        setAttestation(null);
        setAttestationError(true);
        setOracleOnline(false);
      })
      .finally(() => setRefreshing(false));
  };

  useEffect(() => {
    // Initial health check, then fetch attestation
    api
      .oracleHealth()
      .then(() => {
        setOracleOnline(true);
        fetchAttestation();
      })
      .catch(() => {
        setOracleOnline(false);
        setAttestationError(true);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalReserves = RESERVE_BREAKDOWN.reduce(
    (sum, item) => sum + item.value,
    0
  );

  return (
    <div className="space-y-8">
      <DemoBanner feature="Reserve data uses simulated values." details="Collateral ratios, reserve breakdown, and attestation history shown are mock data. The deployed Reserves covenant contract is real (TX verified on TN12)." />
      <div>
        <h2 className="text-2xl font-bold text-white">Proof of Reserves</h2>
        <p className="text-gray-400 text-sm mt-1">
          Collateralization and reserve transparency
        </p>
      </div>

      {/* Reserves Contract Info */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2 mb-4">
          <Vault className="w-5 h-5 text-indigo-400" />
          Reserves Contract
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <p className="text-xs text-gray-500">Contract</p>
            <p className="text-sm text-white font-medium">
              {reservesContract.name} ({reservesContract.scriptSize} bytes)
            </p>
          </div>
          <div>
            <p className="text-xs text-gray-500">P2SH Address</p>
            <a
              href={explorer.addressUrl(reservesContract.p2shAddress)}
              target="_blank"
              rel="noopener"
              className="text-xs text-indigo-400 hover:text-indigo-300 font-mono break-all flex items-center gap-1"
            >
              {reservesContract.p2shAddress.slice(0, 24)}...{reservesContract.p2shAddress.slice(-8)}
              <ExternalLink className="w-3 h-3 flex-shrink-0" />
            </a>
          </div>
          <div>
            <p className="text-xs text-gray-500">Deploy TX</p>
            <a
              href={explorer.txUrl(reservesContract.txId)}
              target="_blank"
              rel="noopener"
              className="text-xs text-blue-400 hover:text-blue-300 font-mono flex items-center gap-1"
            >
              {reservesContract.txId.slice(0, 16)}...
              <ExternalLink className="w-3 h-3 flex-shrink-0" />
            </a>
          </div>
        </div>
        <div className="flex items-center gap-2 mt-4 pt-4 border-t border-gray-800">
          <p className="text-xs text-gray-500">Entrypoints:</p>
          {reservesContract.entrypoints.map((ep) => (
            <span
              key={ep}
              className="text-xs px-1.5 py-0.5 rounded bg-gray-800 text-gray-300 font-mono"
            >
              {ep}
            </span>
          ))}
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">Total Reserves</span>
            <Vault className="w-5 h-5 text-indigo-400" />
          </div>
          <p className="text-2xl font-bold text-white">
            ${totalReserves.toLocaleString()}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            Backing KPROP-NYC-TEST tokens
          </p>
        </div>

        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <span className="text-sm text-gray-400">Collateral Ratio</span>
            <TrendingUp className="w-5 h-5 text-emerald-400" />
          </div>
          <p className="text-2xl font-bold text-emerald-400">
            {attestation ? `${Math.round((totalReserves / (attestation.price_usd * 1000)) * 100)}%` : "118%"}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {attestation
              ? "Above 100% minimum requirement"
              : "Above 100% minimum requirement (simulated - oracle offline)"}
          </p>
        </div>

        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-400">Oracle Status</span>
              <button
                onClick={fetchAttestation}
                disabled={refreshing}
                className="p-1 rounded hover:bg-gray-800 transition-colors disabled:opacity-50"
                title="Refresh Attestation"
                aria-label="Refresh oracle attestation"
              >
                <RefreshCw className={`w-3.5 h-3.5 text-gray-400 ${refreshing ? "animate-spin" : ""}`} />
              </button>
            </div>
            <Activity className="w-5 h-5 text-amber-400" />
          </div>
          <p className={`text-2xl font-bold ${
            oracleOnline === true
              ? "text-emerald-400"
              : oracleOnline === false
                ? "text-red-400"
                : "text-gray-500"
          }`}>
            {oracleOnline === true && attestation
              ? `$${attestation.price_usd.toFixed(4)}`
              : oracleOnline === true
                ? "Connected"
                : oracleOnline === false
                  ? "Oracle offline"
                  : "Checking..."}
          </p>
          <p className="text-xs text-gray-500 mt-1">
            {oracleOnline === true && attestation
              ? `KAS/USD via ${attestation.source || "oracle-pool:3002"}`
              : oracleOnline === true
                ? "CoinGecko feed active via oracle-pool:3002"
                : oracleOnline === false
                  ? "oracle-pool:3002 not reachable"
                  : "Connecting to oracle..."}
          </p>
          {attestationError && (
            <div className="flex items-center gap-1.5 mt-2">
              <div className="w-2 h-2 rounded-full bg-red-500" />
              <span className="text-xs text-red-400">Oracle offline</span>
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Collateral Ratio Chart */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <h3 className="text-lg font-semibold text-white mb-6">
            Collateral Ratio Trend
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={COLLATERAL_DATA}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="month" stroke="#9ca3af" fontSize={12} />
                <YAxis
                  stroke="#9ca3af"
                  fontSize={12}
                  domain={[95, 125]}
                  tickFormatter={(val: number) => `${val}%`}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1f2937",
                    border: "1px solid #374151",
                    borderRadius: "8px",
                    color: "#f3f4f6",
                  }}
                  formatter={(value: number) => [`${value}%`, "Ratio"]}
                />
                <Bar dataKey="ratio" fill="#6366f1" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Reserve Breakdown */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <h3 className="text-lg font-semibold text-white mb-6">
            Reserve Breakdown
          </h3>
          <div className="h-48 mb-6">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie
                  data={RESERVE_BREAKDOWN}
                  cx="50%"
                  cy="50%"
                  innerRadius={50}
                  outerRadius={80}
                  paddingAngle={4}
                  dataKey="value"
                >
                  {RESERVE_BREAKDOWN.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1f2937",
                    border: "1px solid #374151",
                    borderRadius: "8px",
                    color: "#f3f4f6",
                  }}
                  formatter={(value: number) => [
                    `$${value.toLocaleString()}`,
                    "Value",
                  ]}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
          <div className="space-y-2">
            {RESERVE_BREAKDOWN.map((item) => (
              <div
                key={item.name}
                className="flex items-center justify-between"
              >
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: item.color }}
                  />
                  <span className="text-sm text-gray-400">{item.name}</span>
                </div>
                <span className="text-sm text-white font-medium">
                  ${item.value.toLocaleString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Attestation History */}
      <div className="bg-gray-900 rounded-xl border border-gray-800">
        <div className="px-6 py-4 border-b border-gray-800">
          <h3 className="text-lg font-semibold text-white">
            Oracle Attestation History
          </h3>
          <p className="text-xs text-gray-500">
            Weekly reserve verification
          </p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full" role="table">
            <thead>
              <tr className="text-left text-xs text-gray-500 border-b border-gray-800">
                <th className="px-6 py-3 font-medium">Date</th>
                <th className="px-6 py-3 font-medium">Oracle</th>
                <th className="px-6 py-3 font-medium">Collateral Ratio</th>
                <th className="px-6 py-3 font-medium">Status</th>
              </tr>
            </thead>
            <tbody>
              {ATTESTATION_HISTORY.map((att) => (
                <tr
                  key={att.id}
                  className="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="px-6 py-4 text-sm text-gray-300">
                    {att.date}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-300 font-mono">
                    {att.oracle}
                  </td>
                  <td className="px-6 py-4 text-sm text-emerald-400 font-medium">
                    {att.ratio}
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-xs px-2 py-1 rounded-full font-medium bg-emerald-500/10 text-emerald-400">
                      {att.status}
                    </span>
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
