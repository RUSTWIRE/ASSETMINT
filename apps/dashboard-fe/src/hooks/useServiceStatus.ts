// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

import { useState, useEffect, useCallback } from "react";
import { COMPLIANCE_API, ORACLE_API } from "@/lib/api";

export interface ServiceStatus {
  name: string;
  status: "online" | "offline" | "checking";
  latency?: number;
  detail?: string;
}

const SERVICE_ENDPOINTS = [
  { url: `${COMPLIANCE_API}/health`, name: "Backend API" },
  { url: `${COMPLIANCE_API}/network`, name: "Kaspa TN12" },
  { url: "http://localhost:8900/health", name: "Sovereign Metadata" },
  { url: `${ORACLE_API}/health`, name: "Oracle" },
] as const;

const POLL_INTERVAL_MS = 30_000;
const TIMEOUT_MS = 5_000;

async function checkEndpoint(
  url: string,
  name: string
): Promise<ServiceStatus> {
  const start = Date.now();
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(TIMEOUT_MS) });
    const latency = Date.now() - start;
    if (res.ok) {
      const data = await res.json().catch(() => null);
      return {
        name,
        status: "online",
        latency,
        detail:
          data?.network_id ||
          data?.version ||
          data?.service ||
          `${latency}ms`,
      };
    }
    return { name, status: "offline", detail: `HTTP ${res.status}` };
  } catch {
    return { name, status: "offline", detail: "Connection refused" };
  }
}

export function useServiceStatus() {
  const [services, setServices] = useState<ServiceStatus[]>(
    SERVICE_ENDPOINTS.map(({ name }) => ({ name, status: "checking" as const }))
  );

  const checkAll = useCallback(async () => {
    const results = await Promise.allSettled(
      SERVICE_ENDPOINTS.map(({ url, name }) => checkEndpoint(url, name))
    );

    setServices(
      results.map((r, i) =>
        r.status === "fulfilled"
          ? r.value
          : {
              name: SERVICE_ENDPOINTS[i].name,
              status: "offline" as const,
              detail: "Unexpected error",
            }
      )
    );
  }, []);

  useEffect(() => {
    checkAll();
    const interval = setInterval(checkAll, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [checkAll]);

  return services;
}
