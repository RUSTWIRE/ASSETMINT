// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useState } from "react";
import { ShieldCheck, Send, AlertTriangle, CheckCircle2 } from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { api } from "@/lib/api";

interface ComplianceResult {
  allowed: boolean;
  violations?: string[];
  merkle_root?: string;
  rules_evaluated?: number;
}

interface TransferResult {
  tx_hash?: string;
  error?: string;
  compliance_rejected?: boolean;
  violations?: string[];
}

export default function TransferPage() {
  const { wallet } = useWalletStore();
  const [senderDid, setSenderDid] = useState(wallet?.did || "");
  const [receiverDid, setReceiverDid] = useState("");
  const [receiverAddress, setReceiverAddress] = useState("");
  const [assetId, setAssetId] = useState("KPROP-NYC-TEST");
  const [amount, setAmount] = useState("10");
  const [complianceResult, setComplianceResult] =
    useState<ComplianceResult | null>(null);
  const [evaluating, setEvaluating] = useState(false);
  const [transferring, setTransferring] = useState(false);
  const [transferResult, setTransferResult] = useState<TransferResult | null>(
    null
  );
  const [error, setError] = useState("");

  const handleEvaluate = async () => {
    console.log("[K-RWA] Evaluating transfer compliance");
    setEvaluating(true);
    setError("");
    setComplianceResult(null);
    setTransferResult(null);

    try {
      const result = await api.evaluateTransfer(
        senderDid || wallet?.did || "",
        receiverDid,
        assetId,
        Number(amount)
      );
      console.log("[K-RWA] Compliance result:", result);
      setComplianceResult(result);
    } catch (err) {
      console.log("[K-RWA] Compliance evaluation failed, blocking transfer");
      setComplianceResult({
        allowed: false,
        violations: ["Compliance API unreachable — cannot evaluate transfer safety"],
        rules_evaluated: 0,
      });
      setError("Cannot connect to compliance API. Start it with: cargo run -p assetmint-core");
    } finally {
      setEvaluating(false);
    }
  };

  const handleTransfer = async () => {
    console.log("[K-RWA] Executing compliance-gated transfer via POST /transfer");
    setTransferring(true);
    setError("");
    setTransferResult(null);

    try {
      const result = await api.complianceTransfer(
        senderDid || wallet?.did || "",
        receiverDid,
        wallet?.privateKey || "",
        receiverAddress,
        Number(amount),
        assetId
      );

      console.log("[K-RWA] Transfer API response:", result);

      if (result.error || result.compliance_rejected) {
        const message =
          result.error ||
          (result.violations?.length
            ? `Compliance rejected: ${result.violations.join(", ")}`
            : "Transfer rejected by compliance engine");
        setError(message);
        setTransferResult(result);
      } else {
        setTransferResult(result);
      }
    } catch (err) {
      console.error("[K-RWA] Transfer API call failed:", err);
      const message =
        err instanceof Error ? err.message : "Transfer request failed";
      setError(message);
    } finally {
      setTransferring(false);
    }
  };

  if (!wallet) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-gray-400 text-lg">
            Connect your wallet to transfer
          </p>
          <p className="text-gray-600 text-sm mt-2">
            Use the wallet button in the header
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-white">
          ZK-KYC Gated Transfer
        </h2>
        <p className="text-gray-400 text-sm mt-1">
          All transfers require compliance evaluation before execution
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Transfer Form */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white">Transfer Details</h3>

          <div>
            <label
              htmlFor="sender-did"
              className="block text-sm text-gray-400 mb-1"
            >
              Sender DID
            </label>
            <input
              id="sender-did"
              type="text"
              value={senderDid || wallet.did}
              onChange={(e) => setSenderDid(e.target.value)}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm font-mono focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label
              htmlFor="receiver-did"
              className="block text-sm text-gray-400 mb-1"
            >
              Receiver DID
            </label>
            <input
              id="receiver-did"
              type="text"
              value={receiverDid}
              onChange={(e) => setReceiverDid(e.target.value)}
              placeholder="did:kaspa:..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm font-mono focus:outline-none focus:border-indigo-500 placeholder:text-gray-600"
            />
          </div>

          <div>
            <label
              htmlFor="receiver-address"
              className="block text-sm text-gray-400 mb-1"
            >
              Receiver Address
            </label>
            <input
              id="receiver-address"
              type="text"
              value={receiverAddress}
              onChange={(e) => setReceiverAddress(e.target.value)}
              placeholder="kaspatest:q..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm font-mono focus:outline-none focus:border-indigo-500 placeholder:text-gray-600"
            />
          </div>

          <div>
            <label
              htmlFor="asset-select"
              className="block text-sm text-gray-400 mb-1"
            >
              Asset
            </label>
            <select
              id="asset-select"
              value={assetId}
              onChange={(e) => setAssetId(e.target.value)}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
            >
              <option value="KPROP-NYC-TEST">KPROP-NYC-TEST</option>
              <option value="ASTM">ASTM</option>
            </select>
          </div>

          <div>
            <label
              htmlFor="transfer-amount"
              className="block text-sm text-gray-400 mb-1"
            >
              Amount
            </label>
            <input
              id="transfer-amount"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              onClick={handleEvaluate}
              disabled={evaluating || !receiverDid}
              className="flex items-center gap-2 px-5 py-2.5 rounded-lg bg-amber-600 hover:bg-amber-500 text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <ShieldCheck className="w-4 h-4" />
              {evaluating ? "Evaluating..." : "Evaluate Compliance"}
            </button>

            <button
              onClick={handleTransfer}
              disabled={
                !complianceResult?.allowed ||
                transferring ||
                !!transferResult?.tx_hash ||
                !receiverAddress
              }
              className="flex items-center gap-2 px-5 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <Send className="w-4 h-4" />
              {transferring
                ? "Transferring..."
                : transferResult?.tx_hash
                  ? "Complete"
                  : "Execute Transfer"}
            </button>
          </div>
        </div>

        {/* Compliance Result */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <h3 className="text-lg font-semibold text-white mb-4">
            Compliance Result
          </h3>

          {!complianceResult && !error && (
            <div className="flex items-center justify-center h-48 text-gray-600">
              <p className="text-sm">
                Run compliance evaluation to see results
              </p>
            </div>
          )}

          {error && (
            <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-red-400" />
                <p className="text-sm text-red-400">{error}</p>
              </div>
            </div>
          )}

          {complianceResult && (
            <div className="space-y-4">
              <div
                className={`flex items-center gap-3 p-4 rounded-lg ${
                  complianceResult.allowed
                    ? "bg-emerald-500/10 border border-emerald-500/20"
                    : "bg-red-500/10 border border-red-500/20"
                }`}
              >
                {complianceResult.allowed ? (
                  <CheckCircle2 className="w-6 h-6 text-emerald-400" />
                ) : (
                  <AlertTriangle className="w-6 h-6 text-red-400" />
                )}
                <div>
                  <p
                    className={`font-semibold ${
                      complianceResult.allowed
                        ? "text-emerald-400"
                        : "text-red-400"
                    }`}
                  >
                    {complianceResult.allowed
                      ? "Transfer Allowed"
                      : "Transfer Denied"}
                  </p>
                  <p className="text-xs text-gray-400 mt-1">
                    ZK-KYC compliance check{" "}
                    {complianceResult.allowed ? "passed" : "failed"}
                  </p>
                </div>
              </div>

              {complianceResult.violations &&
                complianceResult.violations.length > 0 && (
                  <div className="space-y-2">
                    <p className="text-sm text-gray-400">Violations:</p>
                    {complianceResult.violations.map((v, i) => (
                      <div
                        key={i}
                        className="flex items-center gap-2 text-sm text-red-400"
                      >
                        <AlertTriangle className="w-4 h-4 shrink-0" />
                        {v}
                      </div>
                    ))}
                  </div>
                )}

              {complianceResult.merkle_root && (
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500">Merkle Root</p>
                  <p className="text-xs text-gray-300 font-mono break-all mt-1">
                    {complianceResult.merkle_root}
                  </p>
                </div>
              )}

              {transferResult?.tx_hash && (
                <div className="bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-4 space-y-2">
                  <p className="text-sm text-emerald-400 font-medium">
                    Transfer executed successfully
                  </p>
                  <div>
                    <p className="text-xs text-gray-500">TX Hash</p>
                    <p className="text-xs text-emerald-300 font-mono break-all mt-1">
                      {transferResult.tx_hash}
                    </p>
                  </div>
                  <p className="text-xs text-gray-400">
                    Broadcast to Kaspa Testnet-12
                  </p>
                </div>
              )}

              {transferResult &&
                !transferResult.tx_hash &&
                transferResult.compliance_rejected && (
                  <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 space-y-2">
                    <p className="text-sm text-red-400 font-medium">
                      Transfer rejected by compliance engine
                    </p>
                    {transferResult.violations?.map((v, i) => (
                      <div
                        key={i}
                        className="flex items-center gap-2 text-xs text-red-400"
                      >
                        <AlertTriangle className="w-3 h-3 shrink-0" />
                        {v}
                      </div>
                    ))}
                  </div>
                )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
