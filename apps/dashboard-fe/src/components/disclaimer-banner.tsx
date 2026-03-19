// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { useState } from "react";
import { X } from "lucide-react";

export function DisclaimerBanner() {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  return (
    <div className="fixed top-0 left-0 right-0 z-50 bg-amber-500 text-amber-950 px-4 py-2 text-xs font-medium flex items-center justify-between">
      <p className="flex-1 text-center">
        REGULATORY DISCLAIMER: This is a technical demo on Kaspa Testnet-12.
        Not financial advice. Not for production use. All assets are fictional
        (KPROP-NYC-TEST). Legal wrappers required for any production deployment.
      </p>
      <button
        onClick={() => {
          console.log("[K-RWA] Disclaimer banner dismissed");
          setDismissed(true);
        }}
        className="ml-4 p-1 rounded hover:bg-amber-600 transition-colors"
        aria-label="Dismiss disclaimer"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
