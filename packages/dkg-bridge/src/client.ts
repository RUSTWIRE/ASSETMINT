// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Thin HTTP client for self-hosted DKG Edge Node (http://localhost:8900)
// All Knowledge Assets are PRIVATE by default.

const LOG_PREFIX = '[K-RWA]';

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
}
