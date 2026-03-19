// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Thin HTTP client for self-hosted DKG Edge Node (http://localhost:8900)
// All Knowledge Assets are PRIVATE by default.

import type { AssetMetadata } from './publish';

const LOG_PREFIX = '[K-RWA]';

/// DKG connection error — node unreachable or network failure
export class DkgConnectionError extends Error {
  constructor(message: string) {
    super(`${LOG_PREFIX} DKG connection failed: ${message}`);
    this.name = 'DkgConnectionError';
  }
}

/// DKG API error — node returned a non-OK response
export class DkgApiError extends Error {
  constructor(message: string) {
    super(`${LOG_PREFIX} DKG API error: ${message}`);
    this.name = 'DkgApiError';
  }
}

/**
 * DKG Edge Node client.
 * Connects to the self-hosted OriginTrail DKG Edge Node.
 */
export class DkgClient {
  private endpoint: string;

  /**
   * @param endpoint - DKG Edge Node URL (default: http://localhost:8900)
   */
  constructor(endpoint: string = 'http://localhost:8900') {
    this.endpoint = endpoint;
    console.log(`${LOG_PREFIX} DKG client initialized: ${endpoint}`);
  }

  /**
   * Check node health
   */
  async health(): Promise<boolean> {
    try {
      const response = await fetch(this.endpoint);
      console.log(`${LOG_PREFIX} DKG node health: ${response.status}`);
      return response.ok;
    } catch (error) {
      console.error(`${LOG_PREFIX} DKG node unreachable: ${error}`);
      return false;
    }
  }

  /**
   * Get the endpoint URL
   */
  getEndpoint(): string {
    return this.endpoint;
  }

  /**
   * Get node info including version, network, and health status.
   */
  async info(): Promise<{ version: string; network: string; healthy: boolean }> {
    try {
      const response = await fetch(`${this.endpoint}/info`);
      if (!response.ok) throw new DkgApiError(`Info failed: ${response.statusText}`);
      return response.json();
    } catch (error) {
      if (error instanceof DkgApiError) throw error;
      throw new DkgConnectionError(`${error}`);
    }
  }

  /**
   * Publish a private Knowledge Asset to the DKG Edge Node.
   *
   * @param metadata - RWA asset metadata to publish
   * @returns UAL (Universal Asset Locator) of the published Knowledge Asset
   */
  async publishKnowledgeAsset(metadata: AssetMetadata): Promise<string> {
    console.log(`${LOG_PREFIX} Publishing knowledge asset: ${metadata.name}`);
    try {
      const response = await fetch(`${this.endpoint}/publish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...metadata, private: true }),
      });
      if (!response.ok) throw new DkgApiError(`Publish failed: ${response.statusText}`);
      const result = await response.json();
      const ual = result.ual || `did:dkg:otp/${Date.now()}`;
      console.log(`${LOG_PREFIX} Knowledge asset published: UAL=${ual}`);
      return ual;
    } catch (error) {
      if (error instanceof DkgApiError) throw error;
      throw new DkgConnectionError(`${error}`);
    }
  }

  /**
   * Retrieve a Knowledge Asset by its UAL.
   *
   * @param ual - Universal Asset Locator
   * @returns Asset metadata or null if not found
   */
  async getAsset(ual: string): Promise<AssetMetadata | null> {
    console.log(`${LOG_PREFIX} Retrieving asset: ${ual}`);
    try {
      const response = await fetch(
        `${this.endpoint}/get?ual=${encodeURIComponent(ual)}`,
      );
      if (!response.ok) return null;
      return response.json();
    } catch (error) {
      console.error(`${LOG_PREFIX} Failed to retrieve asset: ${error}`);
      return null;
    }
  }
}
