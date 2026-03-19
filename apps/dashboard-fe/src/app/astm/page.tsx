// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useState } from "react";
import { Gem, Lock, Vote, Clock, TrendingUp, Info, AlertTriangle, ExternalLink } from "lucide-react";
import { useWalletStore } from "@/store/wallet";
import { KTT_TOKENS, KTT_REGISTRY, KTT_TRANSFER_COUNT } from "@/lib/ktt";
import { explorer } from "@/lib/api";

const MOCK_STAKES = [
  {
    id: "stk-001",
    amount: 1500,
    lockDuration: "90 days",
    startDate: "2026-01-15",
    endDate: "2026-04-15",
    rewards: 30.75,
    status: "active",
  },
  {
    id: "stk-002",
    amount: 1000,
    lockDuration: "180 days",
    startDate: "2025-12-01",
    endDate: "2026-05-30",
    rewards: 45.2,
    status: "active",
  },
];

const MOCK_PROPOSALS = [
  {
    id: "prop-001",
    title: "Increase minimum collateral ratio to 120%",
    description:
      "Proposal to raise the minimum collateral ratio from 110% to 120% for enhanced security.",
    votesFor: 125000,
    votesAgainst: 45000,
    status: "active",
    endDate: "2026-03-25",
  },
  {
    id: "prop-002",
    title: "Add EUR jurisdiction support",
    description:
      "Extend compliance engine to support EU MiCA regulations for cross-border tokenization.",
    votesFor: 200000,
    votesAgainst: 12000,
    status: "active",
    endDate: "2026-03-28",
  },
  {
    id: "prop-003",
    title: "Reduce clawback grace period to 48h",
    description:
      "Shorten the grace period before clawback execution from 72h to 48h.",
    votesFor: 80000,
    votesAgainst: 95000,
    status: "closed",
    endDate: "2026-03-10",
  },
];

export default function ASTMPage() {
  const { wallet } = useWalletStore();
  const [stakeAmount, setStakeAmount] = useState("500");
  const [lockDuration, setLockDuration] = useState("90");
  const [staking, setStaking] = useState(false);
  const [staked, setStaked] = useState(false);

  const estimatedAPY =
    lockDuration === "30"
      ? 5.0
      : lockDuration === "90"
        ? 8.2
        : lockDuration === "180"
          ? 12.5
          : 18.0;

  const estimatedRewards =
    (Number(stakeAmount) * estimatedAPY * Number(lockDuration)) / 365 / 100;

  const handleStake = async () => {
    console.log(
      `[K-RWA] Staking ${stakeAmount} ASTM for ${lockDuration} days`
    );
    setStaking(true);
    await new Promise((resolve) => setTimeout(resolve, 2000));
    setStaking(false);
    setStaked(true);
    console.log("[K-RWA] Staking simulation complete");
  };

  const handleVote = (proposalId: string, vote: "for" | "against") => {
    console.log(`[K-RWA] Voting ${vote} on proposal ${proposalId}`);
  };

  if (!wallet) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-gray-400 text-lg">
            Connect your wallet to access ASTM
          </p>
          <p className="text-gray-600 text-sm mt-2">
            Staking and governance require a connected wallet
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* KTT Integration Banner */}
      <div className="bg-emerald-950/50 border border-emerald-700/50 rounded-lg p-4 mb-4">
        <div className="flex items-start gap-3">
          <Gem className="h-5 w-5 text-emerald-400 mt-0.5 shrink-0" />
          <div>
            <h3 className="text-emerald-300 font-semibold text-sm">KTT — Kaspa Trust Token (Covenant-Native)</h3>
            <p className="text-emerald-200/70 text-xs mt-1">
              AssetMint uses the KTT token standard instead of KRC-20. KTT enforces compliance at consensus level via SilverScript covenants — not an indexer.
              {KTT_TRANSFER_COUNT} confirmed token transfers on TN12.
            </p>
            <div className="mt-2 grid grid-cols-3 gap-2">
              {Object.values(KTT_TOKENS).map((token) => (
                <a
                  key={token.ticker}
                  href={explorer.txUrl(token.txId)}
                  target="_blank"
                  rel="noopener"
                  className="flex items-center gap-1 text-xs text-emerald-300 hover:text-emerald-200"
                >
                  <span className="font-mono">{token.ticker}</span>
                  <span className="text-emerald-500">({token.standard})</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              ))}
            </div>
            <a
              href={explorer.txUrl(KTT_REGISTRY.txId)}
              target="_blank"
              rel="noopener"
              className="inline-flex items-center gap-1 text-xs text-emerald-400 hover:text-emerald-300 mt-1"
            >
              Trust Registry TX <ExternalLink className="h-3 w-3" />
            </a>
          </div>
        </div>
      </div>

      <div className="bg-amber-50/5 border border-amber-500/20 rounded-lg p-3 mb-4">
        <div className="flex items-start gap-2">
          <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 shrink-0" />
          <p className="text-amber-200/70 text-sm">
            <strong className="text-amber-300">Demo Mode:</strong> ASTM staking and governance below use simulated data. Real token operations use the KTT standard shown above.
          </p>
        </div>
      </div>

      <div>
        <h2 className="text-2xl font-bold text-white">ASTM Token</h2>
        <p className="text-gray-400 text-sm mt-1">
          Protocol token staking and governance — simulated data
        </p>
      </div>

      <div className="bg-amber-50 border border-amber-200 rounded-lg p-3 mb-4 flex items-start gap-2">
        <AlertTriangle className="h-4 w-4 text-amber-600 mt-0.5 shrink-0" />
        <p className="text-amber-800 text-sm">
          <strong>Demo Mode:</strong> ASTM KRC-20 token is not deployed on Kaspa TN12. KRC-20 inscriptions require the Kasplex commit-reveal protocol. Token staking and governance logic works in-memory only.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Staking Panel */}
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Lock className="w-5 h-5 text-indigo-400" />
            Stake ASTM
          </h3>

          <div>
            <label
              htmlFor="stake-amount"
              className="block text-sm text-gray-400 mb-1"
            >
              Amount
            </label>
            <input
              id="stake-amount"
              type="number"
              value={stakeAmount}
              onChange={(e) => setStakeAmount(e.target.value)}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
            />
            <p className="text-xs text-gray-600 mt-1">
              Available: 5,000 ASTM (mock)
            </p>
          </div>

          <div>
            <label
              htmlFor="lock-duration"
              className="block text-sm text-gray-400 mb-1"
            >
              Lock Duration
            </label>
            <select
              id="lock-duration"
              value={lockDuration}
              onChange={(e) => setLockDuration(e.target.value)}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
            >
              <option value="30">30 days</option>
              <option value="90">90 days</option>
              <option value="180">180 days</option>
              <option value="365">365 days</option>
            </select>
          </div>

          <div className="bg-gray-800 rounded-lg p-4 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-500">Estimated APY</span>
              <span className="text-sm text-emerald-400 font-medium">
                {estimatedAPY}%
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-500">Estimated Rewards</span>
              <span className="text-sm text-white font-medium">
                {estimatedRewards.toFixed(2)} ASTM
              </span>
            </div>
          </div>

          {staked && (
            <div className="bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-3">
              <p className="text-sm text-emerald-400">
                Staked successfully (simulated)
              </p>
            </div>
          )}

          <button
            onClick={handleStake}
            disabled={staking || staked || !stakeAmount}
            className="w-full flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {staking ? "Staking..." : staked ? "Staked" : "Stake ASTM"}
          </button>
        </div>

        {/* Active Stakes */}
        <div className="lg:col-span-2 bg-gray-900 rounded-xl border border-gray-800">
          <div className="px-6 py-4 border-b border-gray-800">
            <h3 className="text-lg font-semibold text-white flex items-center gap-2">
              <Clock className="w-5 h-5 text-amber-400" />
              Active Stakes
            </h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full" role="table">
              <thead>
                <tr className="text-left text-xs text-gray-500 border-b border-gray-800">
                  <th className="px-6 py-3 font-medium">Amount</th>
                  <th className="px-6 py-3 font-medium">Duration</th>
                  <th className="px-6 py-3 font-medium">End Date</th>
                  <th className="px-6 py-3 font-medium">Rewards</th>
                  <th className="px-6 py-3 font-medium">Status</th>
                </tr>
              </thead>
              <tbody>
                {MOCK_STAKES.map((stake) => (
                  <tr
                    key={stake.id}
                    className="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors"
                  >
                    <td className="px-6 py-4 text-sm text-white font-medium">
                      {stake.amount.toLocaleString()} ASTM
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-300">
                      {stake.lockDuration}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-400">
                      {stake.endDate}
                    </td>
                    <td className="px-6 py-4 text-sm text-emerald-400">
                      +{stake.rewards} ASTM
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-xs px-2 py-1 rounded-full font-medium bg-indigo-500/10 text-indigo-400">
                        {stake.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Governance Proposals */}
      <div className="bg-gray-900 rounded-xl border border-gray-800">
        <div className="px-6 py-4 border-b border-gray-800">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Vote className="w-5 h-5 text-purple-400" />
            Governance Proposals
          </h3>
          <p className="text-xs text-gray-500 mt-1">
            Vote with staked ASTM tokens -- mock proposals
          </p>
        </div>
        <div className="divide-y divide-gray-800">
          {MOCK_PROPOSALS.map((proposal) => {
            const totalVotes = proposal.votesFor + proposal.votesAgainst;
            const forPercent =
              totalVotes > 0
                ? ((proposal.votesFor / totalVotes) * 100).toFixed(1)
                : "0";

            return (
              <div key={proposal.id} className="p-6">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <h4 className="text-sm font-semibold text-white">
                        {proposal.title}
                      </h4>
                      <span
                        className={`text-xs px-2 py-0.5 rounded-full font-medium ${
                          proposal.status === "active"
                            ? "bg-emerald-500/10 text-emerald-400"
                            : "bg-gray-700 text-gray-400"
                        }`}
                      >
                        {proposal.status}
                      </span>
                    </div>
                    <p className="text-xs text-gray-400 mt-1">
                      {proposal.description}
                    </p>

                    {/* Vote Bar */}
                    <div className="mt-3">
                      <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                        <span>
                          For: {proposal.votesFor.toLocaleString()} ({forPercent}
                          %)
                        </span>
                        <span>
                          Against: {proposal.votesAgainst.toLocaleString()}
                        </span>
                      </div>
                      <div className="w-full h-2 bg-gray-800 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-emerald-500 rounded-full"
                          style={{ width: `${forPercent}%` }}
                        />
                      </div>
                    </div>

                    <p className="text-xs text-gray-600 mt-2">
                      Ends: {proposal.endDate}
                    </p>
                  </div>

                  {proposal.status === "active" && (
                    <div className="flex gap-2 ml-4">
                      <button
                        onClick={() => handleVote(proposal.id, "for")}
                        className="px-3 py-1.5 text-xs rounded-lg bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/30 transition-colors"
                      >
                        Vote For
                      </button>
                      <button
                        onClick={() => handleVote(proposal.id, "against")}
                        className="px-3 py-1.5 text-xs rounded-lg bg-red-600/20 text-red-400 hover:bg-red-600/30 transition-colors"
                      >
                        Against
                      </button>
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Fee Model Info */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2 mb-4">
          <Info className="w-5 h-5 text-gray-400" />
          ASTM Fee Model
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <TrendingUp className="w-4 h-4 text-indigo-400" />
              <p className="text-sm text-white font-medium">Mint Fee</p>
            </div>
            <p className="text-2xl font-bold text-white">0.1%</p>
            <p className="text-xs text-gray-500 mt-1">
              Of total asset value at mint time
            </p>
          </div>
          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <TrendingUp className="w-4 h-4 text-emerald-400" />
              <p className="text-sm text-white font-medium">Transfer Fee</p>
            </div>
            <p className="text-2xl font-bold text-white">0.05%</p>
            <p className="text-xs text-gray-500 mt-1">
              Per transfer, paid in ASTM
            </p>
          </div>
          <div className="bg-gray-800 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <Gem className="w-4 h-4 text-purple-400" />
              <p className="text-sm text-white font-medium">Staker Share</p>
            </div>
            <p className="text-2xl font-bold text-white">60%</p>
            <p className="text-xs text-gray-500 mt-1">
              Of protocol fees distributed to stakers
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
