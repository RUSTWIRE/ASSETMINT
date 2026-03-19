// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useEffect, useState } from "react";
import {
  FileBox,
  ShieldCheck,
  Database,
  ExternalLink,
  TrendingUp,
  Send,
  FileCode2,
} from "lucide-react";
import Link from "next/link";
import { api, explorer } from "@/lib/api";
import { DEPLOYED_CONTRACTS } from "@/lib/contracts";
import { DemoBanner } from "@/components/demo-banner";

const CONTRACTS = Object.values(DEPLOYED_CONTRACTS);

export default function AssetsPage() {
  const [complianceOnline, setComplianceOnline] = useState<boolean | null>(
    null
  );

  useEffect(() => {
    console.log("[K-RWA] Assets page mounted");

    api
      .health()
      .then(() => setComplianceOnline(true))
      .catch(() => setComplianceOnline(false));
  }, []);

  return (
    <div className="space-y-8">
      <DemoBanner feature="Asset metadata shown is placeholder data." details="Connect to the sovereign metadata service on port 8900 to see real published assets." />
      <div>
        <h2 className="text-2xl font-bold text-white">Asset Detail</h2>
        <p className="text-gray-400 text-sm mt-1">
          KPROP-NYC-TEST -- Tokenized commercial property
        </p>
      </div>

      {/* Asset Card */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            <div className="w-14 h-14 rounded-xl bg-indigo-600/20 flex items-center justify-center">
              <FileBox className="w-7 h-7 text-indigo-400" />
            </div>
            <div>
              <h3 className="text-xl font-bold text-white">KPROP-NYC-TEST</h3>
              <p className="text-sm text-gray-400">
                Tokenized Commercial Property
              </p>
              <p className="text-xs text-gray-600 mt-1">
                350 5th Ave, New York, NY 10118
              </p>
            </div>
          </div>
          <div className="text-right space-y-2">
            <p className="text-sm text-gray-400">Token Status</p>
            <p className="text-lg font-bold text-gray-500">Not deployed</p>
            <p className="text-xs text-gray-600">KRC-20 not yet broadcast</p>
            <Link
              href="/transfer"
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors"
            >
              <Send className="w-3.5 h-3.5" />
              Transfer
            </Link>
          </div>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6 pt-6 border-t border-gray-800">
          <div>
            <p className="text-xs text-gray-500">Token Standard</p>
            <p className="text-sm text-white font-medium">KRC-20</p>
          </div>
          <div>
            <p className="text-xs text-gray-500">Network</p>
            <p className="text-sm text-white font-medium">Kaspa Testnet-12</p>
          </div>
          <div>
            <p className="text-xs text-gray-500">Total Supply</p>
            <p className="text-sm text-gray-500 font-medium">Not available</p>
          </div>
          <div>
            <p className="text-xs text-gray-500">Holders</p>
            <p className="text-sm text-gray-500 font-medium">Not available</p>
          </div>
        </div>
      </div>

      {/* Deployed Contracts */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <FileCode2 className="w-5 h-5 text-indigo-400" />
          Deployed SilverScript Contracts
        </h3>
        <p className="text-xs text-gray-500">
          {CONTRACTS.length} covenant contracts deployed on Testnet-12
        </p>
        <div className="space-y-3">
          {CONTRACTS.map((c) => (
            <div key={c.name} className="bg-gray-800 rounded-lg p-3">
              <div className="flex items-center justify-between mb-1">
                <p className="text-sm text-white font-medium">{c.name}</p>
                <span className="text-xs text-gray-500">{c.scriptSize} bytes</span>
              </div>
              <p className="text-xs text-gray-500 mb-1">P2SH Address</p>
              <a
                href={explorer.addressUrl(c.p2shAddress)}
                target="_blank"
                rel="noopener"
                className="text-xs text-indigo-400 hover:text-indigo-300 font-mono break-all"
              >
                {c.p2shAddress}
              </a>
              <div className="flex items-center gap-2 mt-2">
                <p className="text-xs text-gray-500">Entrypoints:</p>
                {c.entrypoints.map((ep) => (
                  <span
                    key={ep}
                    className="text-xs px-1.5 py-0.5 rounded bg-gray-700 text-gray-300 font-mono"
                  >
                    {ep}
                  </span>
                ))}
              </div>
              <a
                href={explorer.txUrl(c.txId)}
                target="_blank"
                rel="noopener"
                className="inline-flex items-center gap-1 text-xs text-blue-400 hover:text-blue-300 mt-2"
              >
                View deploy TX
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          ))}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Sovereign Metadata */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Database className="w-5 h-5 text-purple-400" />
            Sovereign Metadata
          </h3>
          <div className="space-y-3">
            <div className="bg-gray-800 rounded-lg p-3">
              <p className="text-xs text-gray-500">Asset Identifier</p>
              <p className="text-xs text-indigo-400 font-mono break-all mt-1">
                did:assetmint:sovereign/a3f7b2c1d4e5f6a7
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-3">
              <p className="text-xs text-gray-500">Metadata Hash (SHA-256)</p>
              <p className="text-xs text-gray-300 font-mono break-all mt-1">
                7f8a9b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-3">
              <p className="text-xs text-gray-500">Storage</p>
              <p className="text-xs text-emerald-400 font-medium mt-1">
                Private (self-hosted)
              </p>
            </div>
            <div className="bg-gray-800 rounded-lg p-3">
              <p className="text-xs text-gray-500">Service</p>
              <p className="text-xs text-gray-300 font-mono mt-1">http://localhost:8900</p>
            </div>
          </div>
        </div>

        {/* Compliance Status */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <ShieldCheck className="w-5 h-5 text-emerald-400" />
            Compliance Status
          </h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between py-2 border-b border-gray-800">
              <span className="text-sm text-gray-400">KYC Required</span>
              <span className="text-sm text-emerald-400">Yes</span>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-800">
              <span className="text-sm text-gray-400">Accredited Only</span>
              <span className="text-sm text-emerald-400">Yes</span>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-800">
              <span className="text-sm text-gray-400">Jurisdiction</span>
              <span className="text-sm text-gray-300">US (Reg D 506(c))</span>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-800">
              <span className="text-sm text-gray-400">Lock-up Period</span>
              <span className="text-sm text-gray-300">12 months</span>
            </div>
            <div className="flex items-center justify-between py-2">
              <span className="text-sm text-gray-400">Engine Status</span>
              <span
                className={`text-sm ${
                  complianceOnline === true
                    ? "text-emerald-400"
                    : complianceOnline === false
                      ? "text-red-400"
                      : "text-gray-500"
                }`}
              >
                {complianceOnline === true
                  ? "Online"
                  : complianceOnline === false
                    ? "Offline"
                    : "Checking..."}
              </span>
            </div>
          </div>
        </div>

        {/* Oracle Price Feed */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <TrendingUp className="w-5 h-5 text-amber-400" />
            Oracle Price Feed
          </h3>
          <div className="space-y-3">
            <div className="bg-gray-800 rounded-lg p-4 text-center">
              <p className="text-xs text-gray-500">KPROP-NYC-TEST / USD</p>
              <p className="text-3xl font-bold text-gray-500 mt-2">
                Not available
              </p>
              <p className="text-xs text-gray-600 mt-1">
                Oracle feed not yet configured for this asset
              </p>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-800">
              <span className="text-sm text-gray-400">Oracle Source</span>
              <span className="text-sm text-gray-300">oracle-pool:3002</span>
            </div>
            <div className="flex items-center justify-between py-2">
              <span className="text-sm text-gray-400">Status</span>
              <span className="text-sm text-gray-500">Pending deployment</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
