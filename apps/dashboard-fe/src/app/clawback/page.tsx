// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useState } from "react";
import { ShieldAlert, Loader2, AlertTriangle, ExternalLink } from "lucide-react";
import { DemoBanner } from "@/components/demo-banner";
import { useWalletStore } from "@/store/wallet";
import { DEPLOYED_CONTRACTS } from "@/lib/contracts";
import { explorer } from "@/lib/api";

const clawbackContract = DEPLOYED_CONTRACTS.clawback;

export default function ClawbackPage() {
  const { wallet } = useWalletStore();
  const [targetAddress, setTargetAddress] = useState("");
  const [reason, setReason] = useState("");
  const [executing, setExecuting] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  const handleClawback = async () => {
    console.log("[K-RWA] Attempting clawback covenant execution");
    console.log("[K-RWA] Target:", targetAddress);
    console.log("[K-RWA] Reason:", reason);
    setExecuting(true);
    await new Promise((resolve) => setTimeout(resolve, 1500));
    setExecuting(false);
    setSubmitted(true);
    console.log("[K-RWA] Clawback requires P2SH witness — not yet implemented");
  };

  if (!wallet) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-gray-400 text-lg">
            Connect your wallet to access clawback admin
          </p>
          <p className="text-gray-600 text-sm mt-2">
            Issuer-only functionality
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <DemoBanner feature="Clawback uses mock examples." details="The deployed Clawback covenant contract is real (TX verified on TN12), but the enforcement execution flow is not yet implemented." />
      <div>
        <h2 className="text-2xl font-bold text-white">Issuer Clawback</h2>
        <p className="text-gray-400 text-sm mt-1">
          Admin panel for regulatory clawback operations -- issuer only
        </p>
      </div>

      {/* Warning Banner */}
      <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-4 flex items-start gap-3">
        <AlertTriangle className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
        <div>
          <p className="text-sm text-red-400 font-medium">
            Restricted Operation
          </p>
          <p className="text-xs text-gray-400 mt-1">
            Clawback is an issuer-level action used for regulatory compliance
            (sanctions, court orders, expired KYC). All clawbacks are recorded
            on-chain via OP_RETURN. In production, this requires multi-sig
            authorization and legal review.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Clawback Form */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <ShieldAlert className="w-5 h-5 text-red-400" />
            Execute Clawback
          </h3>

          <div>
            <label
              htmlFor="target-address"
              className="block text-sm text-gray-400 mb-1"
            >
              Target Address
            </label>
            <input
              id="target-address"
              type="text"
              value={targetAddress}
              onChange={(e) => setTargetAddress(e.target.value)}
              placeholder="kaspatest:q..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm font-mono focus:outline-none focus:border-red-500 placeholder:text-gray-600"
            />
          </div>

          <div>
            <label
              htmlFor="clawback-reason"
              className="block text-sm text-gray-400 mb-1"
            >
              Reason (recorded in OP_RETURN)
            </label>
            <textarea
              id="clawback-reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="e.g., OFAC sanctions list match"
              rows={3}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-red-500 placeholder:text-gray-600 resize-none"
            />
          </div>

          {submitted && (
            <div className="bg-amber-500/10 border border-amber-500/20 rounded-lg p-3 space-y-2">
              <p className="text-sm text-amber-400 font-medium">
                P2SH Witness Required
              </p>
              <p className="text-xs text-gray-400">
                Clawback covenant execution requires P2SH witness construction
                — coming soon. The Clawback contract IS deployed on TN12 at:
              </p>
              <p className="text-xs">
                <a
                  href={explorer.addressUrl(clawbackContract.p2shAddress)}
                  target="_blank"
                  rel="noopener"
                  className="text-indigo-400 hover:text-indigo-300 font-mono inline-flex items-center gap-1"
                >
                  {clawbackContract.p2shAddress.slice(0, 24)}...
                  <ExternalLink className="w-3 h-3" />
                </a>
              </p>
              <p className="text-xs">
                TX:{" "}
                <a
                  href={explorer.txUrl(clawbackContract.txId)}
                  target="_blank"
                  rel="noopener"
                  className="text-blue-400 hover:text-blue-300 font-mono inline-flex items-center gap-1"
                >
                  {clawbackContract.txId.slice(0, 16)}...
                  <ExternalLink className="w-3 h-3" />
                </a>
              </p>
            </div>
          )}

          <button
            onClick={handleClawback}
            disabled={!targetAddress || !reason || executing || submitted}
            className="w-full flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-red-600 hover:bg-red-500 text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {executing ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Executing...
              </>
            ) : submitted ? (
              "Pending P2SH Implementation"
            ) : (
              <>
                <ShieldAlert className="w-4 h-4" />
                Execute Clawback
              </>
            )}
          </button>
        </div>

        {/* Contract Details (replaces mock history) */}
        <div className="lg:col-span-2 bg-gray-900 rounded-xl border border-gray-800">
          <div className="px-6 py-4 border-b border-gray-800">
            <h3 className="text-lg font-semibold text-white">
              Clawback Contract
            </h3>
            <p className="text-xs text-gray-500">
              Deployed covenant on Kaspa Testnet-12
            </p>
          </div>
          <div className="p-6 space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-gray-500 mb-1">Contract Name</p>
                <p className="text-sm text-white font-medium">
                  {clawbackContract.name}
                </p>
              </div>
              <div>
                <p className="text-xs text-gray-500 mb-1">Script Size</p>
                <p className="text-sm text-white font-mono">
                  {clawbackContract.scriptSize} bytes
                </p>
              </div>
              <div className="sm:col-span-2">
                <p className="text-xs text-gray-500 mb-1">P2SH Address</p>
                <a
                  href={explorer.addressUrl(clawbackContract.p2shAddress)}
                  target="_blank"
                  rel="noopener"
                  className="text-sm text-indigo-400 hover:text-indigo-300 font-mono flex items-center gap-1 break-all"
                >
                  {clawbackContract.p2shAddress}
                  <ExternalLink className="w-3 h-3 shrink-0" />
                </a>
              </div>
              <div className="sm:col-span-2">
                <p className="text-xs text-gray-500 mb-1">Deployment TX</p>
                <a
                  href={explorer.txUrl(clawbackContract.txId)}
                  target="_blank"
                  rel="noopener"
                  className="text-sm text-blue-400 hover:text-blue-300 font-mono flex items-center gap-1 break-all"
                >
                  {clawbackContract.txId}
                  <ExternalLink className="w-3 h-3 shrink-0" />
                </a>
              </div>
              <div className="sm:col-span-2">
                <p className="text-xs text-gray-500 mb-1">Entrypoints</p>
                <div className="flex gap-2 flex-wrap">
                  {clawbackContract.entrypoints.map((ep) => (
                    <span
                      key={ep}
                      className="text-xs px-2.5 py-1 rounded-full bg-gray-800 text-gray-300 font-mono"
                    >
                      {ep}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            <div className="border-t border-gray-800 pt-4">
              <p className="text-xs text-gray-500 mb-2">How Clawback Works</p>
              <ol className="text-xs text-gray-400 space-y-1.5 list-decimal list-inside">
                <li>
                  Issuer identifies a non-compliant holder (sanctions, expired KYC, court order)
                </li>
                <li>
                  Clawback covenant is invoked via the{" "}
                  <code className="text-amber-400 bg-gray-800 px-1 rounded">
                    issuerClawback
                  </code>{" "}
                  entrypoint
                </li>
                <li>
                  P2SH witness proves issuer authorization and records reason in OP_RETURN
                </li>
                <li>
                  Funds are returned to the issuer treasury address
                </li>
              </ol>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
