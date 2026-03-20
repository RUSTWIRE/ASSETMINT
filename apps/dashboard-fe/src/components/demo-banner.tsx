// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import { AlertTriangle } from "lucide-react";

interface DemoBannerProps {
  feature: string;
  details?: string;
}

export function DemoBanner({ feature, details }: DemoBannerProps) {
  return (
    <div className="bg-amber-900/20 border border-amber-700/40 rounded-lg p-3 mb-4 flex items-start gap-2">
      <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0" />
      <div>
        <p className="text-amber-400 text-sm">
          <strong>Demo Mode:</strong> {feature}
        </p>
        {details && (
          <p className="text-amber-500/80 text-xs mt-1">{details}</p>
        )}
      </div>
    </div>
  );
}
