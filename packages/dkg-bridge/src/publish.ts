// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Publish private Knowledge Assets to self-hosted DKG Edge Node.

const LOG_PREFIX = '[K-RWA]';

/**
 * RWA asset metadata for DKG publication
 */
export interface AssetMetadata {
  /** Asset name (e.g., "NYC Commercial Property #42") */
  name: string;
  /** Asset type (e.g., "RealEstate", "FinancialProduct") */
  assetType: string;
  /** Asset description */
  description: string;
  /** Valuation in USD */
  valuation: number;
  /** Physical address (if applicable) */
  propertyAddress?: string;
  /** Kaspa P2SH covenant address */
  covenantAddress: string;
  /** Compliance Merkle root */
  complianceMerkleRoot: string;
  /** Document hashes (IPFS CIDs or SHA-256) */
  documentHashes: string[];
  /** AssetMint-1.0 ticker */
  ticker: string;
}

/**
 * Publish RWA metadata as a private Knowledge Asset on the self-hosted DKG Edge Node.
 *
 * @param endpoint - DKG Edge Node URL
 * @param metadata - Asset metadata to publish
 * @returns UAL (Universal Asset Locator) of the published Knowledge Asset
 */
export async function publishAsset(
  endpoint: string,
  metadata: AssetMetadata,
): Promise<string> {
  console.log(`${LOG_PREFIX} Publishing asset to DKG: ${metadata.name}`);

  // Publish to AssetMint Sovereign Metadata Service (localhost:8900)
  const response = await fetch(`${endpoint}/publish`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(metadata),
  });

  if (!response.ok) {
    throw new Error(`${LOG_PREFIX} Publish failed: ${response.statusText}`);
  }

  const result = await response.json() as { ual: string };
  console.log(`${LOG_PREFIX} Asset published: UAL=${result.ual} (sovereign, private)`);
  return result.ual;
}
