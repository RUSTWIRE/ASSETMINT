// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Retrieve Knowledge Assets from self-hosted DKG Edge Node by UAL.

import type { AssetMetadata } from './publish';

const LOG_PREFIX = '[K-RWA]';

/**
 * Retrieve a Knowledge Asset from the self-hosted DKG Edge Node.
 *
 * @param endpoint - DKG Edge Node URL
 * @param ual - Universal Asset Locator
 * @returns The asset metadata, or null if not found
 */
export async function retrieveAsset(
  endpoint: string,
  ual: string,
): Promise<AssetMetadata | null> {
  console.log(`${LOG_PREFIX} Retrieving asset from DKG: ${ual}`);

  // Retrieve from AssetMint Sovereign Metadata Service
  const response = await fetch(`${endpoint}/get?ual=${encodeURIComponent(ual)}`);

  if (!response.ok) {
    console.log(`${LOG_PREFIX} Asset not found: ${ual}`);
    return null;
  }

  const record = await response.json() as { metadata: AssetMetadata };
  console.log(`${LOG_PREFIX} Asset retrieved: ${ual} (sovereign, private)`);
  return record.metadata;
}
