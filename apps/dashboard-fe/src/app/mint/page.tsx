// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useState } from "react";
import {
  CheckCircle2,
  Circle,
  Loader2,
  ArrowRight,
  ArrowLeft,
  AlertTriangle,
  ExternalLink,
} from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { DemoBanner } from "@/components/demo-banner";
import { cn } from "@/lib/cn";
import { COMPLIANCE_API, explorer } from "@/lib/api";
import { DEPLOYED_CONTRACTS } from "@/lib/contracts";

interface MintFormData {
  name: string;
  assetType: string;
  description: string;
  value: string;
}

interface StepResult {
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

const STEPS = [
  { id: 1, title: "Asset Details", description: "Enter asset metadata" },
  { id: 2, title: "DKG Publish", description: "Publish to sovereign metadata service" },
  { id: 3, title: "ZK-KYC Proof", description: "Generate compliance proof" },
  { id: 4, title: "Deploy Covenant", description: "UTXO covenant contracts" },
  { id: 5, title: "KRC-20 Mint", description: "Preview only — not broadcast" },
];

type StepStatus = "pending" | "active" | "processing" | "complete" | "error";

/** SHA-256 hash of a string, returned as hex */
async function sha256Hex(input: string): Promise<string> {
  const encoded = new TextEncoder().encode(input);
  const hashBuffer = await crypto.subtle.digest("SHA-256", encoded);
  return Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export default function MintPage() {
  const { wallet } = useWalletStore();
  const [currentStep, setCurrentStep] = useState(1);
  const [stepStatuses, setStepStatuses] = useState<Record<number, StepStatus>>({
    1: "active",
    2: "pending",
    3: "pending",
    4: "pending",
    5: "pending",
  });
  const [formData, setFormData] = useState<MintFormData>({
    name: "KPROP-NYC-TEST",
    assetType: "commercial_property",
    description: "Tokenized commercial property at 350 5th Ave, New York, NY",
    value: "1000000",
  });
  const [stepResults, setStepResults] = useState<Record<number, StepResult>>(
    {}
  );

  const completeStep = (step: number) => {
    setStepStatuses((prev) => ({
      ...prev,
      [step]: "complete",
      ...(step < 5 ? { [step + 1]: "active" } : {}),
    }));
    if (step < 5) setCurrentStep(step + 1);
  };

  const failStep = (step: number, error: string) => {
    setStepStatuses((prev) => ({ ...prev, [step]: "error" }));
    setStepResults((prev) => ({ ...prev, [step]: { success: false, error } }));
  };

  // ── Step 1: just validates and advances ──
  const executeStep1 = () => {
    if (!formData.name.trim() || !formData.value.trim()) {
      failStep(1, "Asset name and value are required.");
      return;
    }
    setStepResults((prev) => ({
      ...prev,
      [1]: { success: true, data: { ...formData } },
    }));
    completeStep(1);
  };

  // ── Step 2: DKG — try sovereign metadata service, fall back to local hash ──
  const executeStep2 = async () => {
    setStepStatuses((prev) => ({ ...prev, [2]: "processing" }));
    const metadata = {
      name: formData.name,
      type: formData.assetType,
      description: formData.description,
      valuation_usd: formData.value,
      owner: wallet?.did,
      timestamp: new Date().toISOString(),
    };
    const metadataJson = JSON.stringify(metadata, null, 2);

    try {
      // Try to publish to sovereign metadata service
      const response = await fetch("http://localhost:8900/publish", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: metadataJson,
      });

      if (response.ok) {
        const result = await response.json();
        setStepResults((prev) => ({
          ...prev,
          [2]: {
            success: true,
            data: {
              dkgConnected: true,
              metadataJson,
              assetReference: result.metadata_hash,
              ual: result.ual,
              sovereignMetadata: true,
            },
          },
        }));
        completeStep(2);
        return;
      }
    } catch {
      // Sovereign metadata service not available — fall back to local hash
    }

    // Fallback: local SHA-256 hash only
    const hash = await sha256Hex(metadataJson);
    setStepResults((prev) => ({
      ...prev,
      [2]: {
        success: true,
        data: {
          dkgConnected: false,
          metadataJson,
          assetReference: hash,
          sovereignMetadata: false,
        },
      },
    }));
    completeStep(2);
  };

  // ── Step 3: ZK-KYC — real API call ──
  const executeStep3 = async () => {
    if (!wallet) return;
    setStepStatuses((prev) => ({ ...prev, [3]: "processing" }));
    try {
      const response = await fetch(
        `${COMPLIANCE_API}/zk-proof/${wallet.address}`
      );
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const data = await response.json();
      setStepResults((prev) => ({
        ...prev,
        [3]: {
          success: true,
          data: {
            proof: data.proof,
            publicInputs: data.public_inputs,
            proofHash: data.proof_hash,
            merkleRoot: data.merkle_root,
          },
        },
      }));
      completeStep(3);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Unknown error";
      failStep(
        3,
        `Compliance API not running at ${COMPLIANCE_API} — ${message}`
      );
    }
  };

  // ── Step 4: Show real deployed covenant contracts ──
  const executeStep4 = () => {
    setStepStatuses((prev) => ({ ...prev, [4]: "processing" }));
    const contracts = Object.entries(DEPLOYED_CONTRACTS).map(
      ([key, contract]) => ({
        key,
        name: contract.name,
        txId: contract.txId,
        p2shAddress: contract.p2shAddress,
        scriptSize: contract.scriptSize,
        entrypoints: [...contract.entrypoints],
      })
    );
    setStepResults((prev) => ({
      ...prev,
      [4]: { success: true, data: { contracts } },
    }));
    completeStep(4);
  };

  // ── Step 5: KRC-20 inscription preview (honest) ──
  const executeStep5 = () => {
    setStepStatuses((prev) => ({ ...prev, [5]: "processing" }));
    const inscription = {
      p: "krc-20",
      op: "mint",
      tick: "ASTM",
      amt: formData.value,
      to: wallet?.address,
    };
    setStepResults((prev) => ({
      ...prev,
      [5]: {
        success: true,
        data: {
          inscriptionJson: JSON.stringify(inscription, null, 2),
          inscription,
        },
      },
    }));
    completeStep(5);
  };

  const executeStep = (step: number) => {
    switch (step) {
      case 1:
        executeStep1();
        break;
      case 2:
        executeStep2();
        break;
      case 3:
        executeStep3();
        break;
      case 4:
        executeStep4();
        break;
      case 5:
        executeStep5();
        break;
    }
  };

  const getStepIcon = (stepId: number) => {
    const status = stepStatuses[stepId];
    if (status === "complete")
      return <CheckCircle2 className="w-6 h-6 text-emerald-400" />;
    if (status === "error")
      return <AlertTriangle className="w-6 h-6 text-red-400" />;
    if (status === "processing")
      return <Loader2 className="w-6 h-6 text-indigo-400 animate-spin" />;
    if (status === "active")
      return <Circle className="w-6 h-6 text-indigo-400" />;
    return <Circle className="w-6 h-6 text-gray-600" />;
  };

  if (!wallet) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-gray-400 text-lg">Connect your wallet to mint</p>
          <p className="text-gray-600 text-sm mt-2">
            Use the wallet button in the header
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <DemoBanner feature="Steps 2 (DKG) and 5 (KRC-20) use the sovereign metadata service instead of OriginTrail." details="Metadata is stored privately via SHA-256 integrity hashes on port 8900. On-chain hash commitment available via POST /metadata/publish-and-commit." />
      <div>
        <h2 className="text-2xl font-bold text-white">Mint RWA Token</h2>
        <p className="text-gray-400 text-sm mt-1">
          Multi-step minting wizard — real APIs and deployed contracts
        </p>
      </div>

      {/* Step Progress */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        <div className="flex items-center justify-between">
          {STEPS.map((step, index) => (
            <div key={step.id} className="flex items-center flex-1">
              <div className="flex flex-col items-center">
                {getStepIcon(step.id)}
                <span
                  className={cn(
                    "text-xs mt-2 text-center",
                    stepStatuses[step.id] === "active" ||
                      stepStatuses[step.id] === "processing"
                      ? "text-indigo-400"
                      : stepStatuses[step.id] === "complete"
                        ? "text-emerald-400"
                        : stepStatuses[step.id] === "error"
                          ? "text-red-400"
                          : "text-gray-600"
                  )}
                >
                  {step.title}
                </span>
              </div>
              {index < STEPS.length - 1 && (
                <div
                  className={cn(
                    "flex-1 h-px mx-4 mt-[-1rem]",
                    stepStatuses[step.id] === "complete"
                      ? "bg-emerald-400"
                      : "bg-gray-700"
                  )}
                />
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Step Content */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        {/* Step 1: Asset Details */}
        {currentStep === 1 && (
          <div className="space-y-6">
            <h3 className="text-lg font-semibold text-white">Asset Details</h3>
            <div className="grid grid-cols-1 gap-4 max-w-lg">
              <div>
                <label
                  htmlFor="asset-name"
                  className="block text-sm text-gray-400 mb-1"
                >
                  Asset Name
                </label>
                <input
                  id="asset-name"
                  type="text"
                  value={formData.name}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, name: e.target.value }))
                  }
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label
                  htmlFor="asset-type"
                  className="block text-sm text-gray-400 mb-1"
                >
                  Asset Type
                </label>
                <select
                  id="asset-type"
                  value={formData.assetType}
                  onChange={(e) =>
                    setFormData((prev) => ({
                      ...prev,
                      assetType: e.target.value,
                    }))
                  }
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
                >
                  <option value="commercial_property">
                    Commercial Property
                  </option>
                  <option value="residential_property">
                    Residential Property
                  </option>
                  <option value="commodity">Commodity</option>
                  <option value="treasury_bond">Treasury Bond</option>
                </select>
              </div>
              <div>
                <label
                  htmlFor="asset-description"
                  className="block text-sm text-gray-400 mb-1"
                >
                  Description
                </label>
                <textarea
                  id="asset-description"
                  value={formData.description}
                  onChange={(e) =>
                    setFormData((prev) => ({
                      ...prev,
                      description: e.target.value,
                    }))
                  }
                  rows={3}
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500 resize-none"
                />
              </div>
              <div>
                <label
                  htmlFor="asset-value"
                  className="block text-sm text-gray-400 mb-1"
                >
                  Value (USD)
                </label>
                <input
                  id="asset-value"
                  type="number"
                  value={formData.value}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, value: e.target.value }))
                  }
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>
            {stepResults[1]?.error && (
              <p className="text-sm text-red-400">{stepResults[1].error}</p>
            )}
          </div>
        )}

        {/* Step 2: DKG Publish */}
        {currentStep === 2 && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white">
              Sovereign Metadata Publication
            </h3>
            {(!stepResults[2]?.data?.sovereignMetadata) && (
              <div className="bg-yellow-900/20 border border-yellow-700/40 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <AlertTriangle className="w-5 h-5 text-yellow-500 mt-0.5 shrink-0" />
                  <div>
                    <p className="text-sm text-yellow-400 font-medium">
                      Sovereign Metadata: Local Hash Only
                    </p>
                    <p className="text-xs text-yellow-600 mt-1">
                      Metadata is hashed locally in form state. To publish to the
                      sovereign metadata service, use POST http://localhost:8900/publish
                      or POST /metadata/publish-and-commit for atomic DAG commitment.
                    </p>
                  </div>
                </div>
              </div>
            )}
            {stepResults[2]?.success && stepResults[2].data && (
              <div className="space-y-3">
                {Boolean(stepResults[2].data.sovereignMetadata) && (
                  <div className="bg-emerald-900/20 border border-emerald-700/40 rounded-lg p-4">
                    <div className="flex items-start gap-3">
                      <CheckCircle2 className="w-5 h-5 text-emerald-500 mt-0.5 shrink-0" />
                      <div>
                        <p className="text-sm text-emerald-400 font-medium">
                          Published to Sovereign Metadata Service
                        </p>
                        <p className="text-xs text-emerald-600 mt-1">
                          Metadata stored at http://localhost:8900 with SHA-256 integrity hash.
                        </p>
                      </div>
                    </div>
                  </div>
                )}
                {Boolean(stepResults[2].data.ual) && (
                  <div className="bg-gray-800 rounded-lg p-4">
                    <p className="text-xs text-gray-500 mb-1">UAL (Universal Asset Locator)</p>
                    <p className="text-sm text-indigo-400 font-mono break-all">
                      {String(stepResults[2].data.ual)}
                    </p>
                  </div>
                )}
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500 mb-1">
                    {stepResults[2].data.sovereignMetadata
                      ? "Metadata Hash (from sovereign service)"
                      : "Asset Reference (local SHA-256)"}
                  </p>
                  <p className="text-sm text-emerald-400 font-mono break-all">
                    {String(stepResults[2].data.assetReference)}
                  </p>
                </div>
                <details className="bg-gray-800 rounded-lg p-4">
                  <summary className="text-xs text-gray-500 cursor-pointer">
                    Metadata JSON
                  </summary>
                  <pre className="text-xs text-gray-300 font-mono mt-2 whitespace-pre-wrap">
                    {String(stepResults[2].data.metadataJson)}
                  </pre>
                </details>
              </div>
            )}
          </div>
        )}

        {/* Step 3: ZK-KYC Proof */}
        {currentStep === 3 && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white">
              ZK-KYC Proof Generation
            </h3>
            <p className="text-sm text-gray-400">
              Calls{" "}
              <code className="text-xs bg-gray-800 px-1.5 py-0.5 rounded">
                GET /zk-proof/{wallet.address.slice(0, 20)}...
              </code>{" "}
              on the compliance API to generate a Groth16 proof.
            </p>
            {stepStatuses[3] === "processing" && (
              <div className="bg-gray-800 rounded-lg p-4">
                <p className="text-sm text-indigo-400">
                  Calling compliance API for Groth16 proof...
                </p>
              </div>
            )}
            {stepResults[3]?.error && (
              <div className="bg-red-900/20 border border-red-700/40 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <AlertTriangle className="w-5 h-5 text-red-400 mt-0.5 shrink-0" />
                  <div>
                    <p className="text-sm text-red-400 font-medium">
                      Compliance API not running
                    </p>
                    <p className="text-xs text-red-500 mt-1">
                      {stepResults[3].error}
                    </p>
                    <p className="text-xs text-gray-500 mt-2">
                      Start the compliance API:{" "}
                      <code className="bg-gray-800 px-1.5 py-0.5 rounded">
                        cargo run -p assetmint-core
                      </code>
                    </p>
                  </div>
                </div>
              </div>
            )}
            {stepResults[3]?.success && stepResults[3].data && (
              <div className="space-y-3">
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500 mb-1">Proof Hash</p>
                  <p className="text-sm text-emerald-400 font-mono break-all">
                    {String(stepResults[3].data.proofHash)}
                  </p>
                </div>
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500 mb-1">Merkle Root</p>
                  <p className="text-sm text-emerald-400 font-mono break-all">
                    {String(stepResults[3].data.merkleRoot)}
                  </p>
                </div>
                <details className="bg-gray-800 rounded-lg p-4">
                  <summary className="text-xs text-gray-500 cursor-pointer">
                    Public Inputs
                  </summary>
                  <pre className="text-xs text-gray-300 font-mono mt-2 whitespace-pre-wrap">
                    {JSON.stringify(stepResults[3].data.publicInputs, null, 2)}
                  </pre>
                </details>
              </div>
            )}
          </div>
        )}

        {/* Step 4: Covenant Deployment */}
        {currentStep === 4 && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white">
              Deployed Covenant Contracts
            </h3>
            <p className="text-sm text-gray-400">
              These SilverScript covenants are already deployed on Kaspa
              Testnet-12. No simulation — these are real P2SH addresses and TX
              hashes.
            </p>
            {stepResults[4]?.success &&
              stepResults[4].data &&
              Array.isArray(stepResults[4].data.contracts) && (
                <div className="space-y-3">
                  {(
                    stepResults[4].data.contracts as Array<{
                      key: string;
                      name: string;
                      txId: string;
                      p2shAddress: string;
                      scriptSize: number;
                      entrypoints: string[];
                    }>
                  ).map((contract) => (
                    <div
                      key={contract.key}
                      className="bg-gray-800 rounded-lg p-4 space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <p className="text-sm font-medium text-white">
                          {contract.name}
                        </p>
                        <span className="text-xs text-gray-500">
                          {contract.scriptSize} bytes
                        </span>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500">P2SH Address</p>
                        <p className="text-xs text-indigo-400 font-mono break-all">
                          {contract.p2shAddress}
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500">Deploy TX</p>
                        <a
                          href={explorer.txUrl(contract.txId)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-xs text-emerald-400 font-mono break-all hover:underline inline-flex items-center gap-1"
                        >
                          {contract.txId}
                          <ExternalLink className="w-3 h-3" />
                        </a>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500">Entrypoints</p>
                        <div className="flex gap-2 mt-1">
                          {contract.entrypoints.map((ep: string) => (
                            <span
                              key={ep}
                              className="text-xs bg-gray-700 text-gray-300 px-2 py-0.5 rounded"
                            >
                              {ep}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
          </div>
        )}

        {/* Step 5: KRC-20 Mint */}
        {currentStep === 5 && (
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white">
              KRC-20 Inscription
            </h3>
            <div className="bg-yellow-900/20 border border-yellow-700/40 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="w-5 h-5 text-yellow-500 mt-0.5 shrink-0" />
                <div>
                  <p className="text-sm text-yellow-400 font-medium">
                    KRC-20 inscription requires Kasplex protocol
                  </p>
                  <p className="text-xs text-yellow-600 mt-1">
                    Broadcasting KRC-20 inscriptions requires the Kasplex
                    commit-reveal protocol (not OP_RETURN). This demo shows the
                    inscription JSON that would be broadcast.
                  </p>
                  <p className="text-xs text-gray-500 mt-2">
                    Inscription logic:{" "}
                    <code className="bg-gray-800 px-1.5 py-0.5 rounded">
                      tokenomics/src/token.rs
                    </code>
                  </p>
                </div>
              </div>
            </div>
            {stepResults[5]?.success && stepResults[5].data && (
              <div className="space-y-3">
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500 mb-2">
                    Inscription JSON (would be broadcast)
                  </p>
                  <pre className="text-sm text-emerald-400 font-mono whitespace-pre-wrap">
                    {String(stepResults[5].data.inscriptionJson)}
                  </pre>
                </div>
                <div className="bg-gray-800 rounded-lg p-4">
                  <p className="text-xs text-gray-500">
                    Asset: {formData.name} | Value: $
                    {Number(formData.value).toLocaleString()} | Tokens:{" "}
                    {formData.value} ASTM
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Navigation Buttons */}
        <div className="flex items-center justify-between mt-8 pt-6 border-t border-gray-800">
          <button
            onClick={() => setCurrentStep(Math.max(1, currentStep - 1))}
            disabled={currentStep === 1}
            className="flex items-center gap-2 px-4 py-2 text-sm text-gray-400 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back
          </button>

          {stepStatuses[currentStep] === "complete" && currentStep < 5 ? (
            <button
              onClick={() => setCurrentStep(currentStep + 1)}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors"
            >
              Next
              <ArrowRight className="w-4 h-4" />
            </button>
          ) : stepStatuses[currentStep] === "error" ? (
            <button
              onClick={() => {
                setStepStatuses((prev) => ({
                  ...prev,
                  [currentStep]: "active",
                }));
                setStepResults((prev) => {
                  const next = { ...prev };
                  delete next[currentStep];
                  return next;
                });
              }}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-red-600 hover:bg-red-500 text-white text-sm font-medium transition-colors"
            >
              Retry
            </button>
          ) : (
            <button
              onClick={() => executeStep(currentStep)}
              disabled={
                stepStatuses[currentStep] === "processing" ||
                stepStatuses[currentStep] === "complete"
              }
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {stepStatuses[currentStep] === "processing" ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Processing...
                </>
              ) : stepStatuses[currentStep] === "complete" ? (
                "Complete"
              ) : (
                `Execute Step ${currentStep}`
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
