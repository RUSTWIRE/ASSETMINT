// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { DisclaimerBanner } from "@/components/disclaimer-banner";
import { Sidebar } from "@/components/sidebar";
import { WalletButton } from "@/components/wallet-button";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "AssetMint — RWA Tokenization Dashboard",
  description:
    "Institutional-grade RWA tokenization platform on Kaspa Testnet-12. Technical demo only.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} bg-gray-950 text-gray-100 antialiased`}>
        <DisclaimerBanner />
        <Sidebar />

        <div className="ml-64 pt-8 min-h-screen">
          <header className="sticky top-8 z-30 bg-gray-950/80 backdrop-blur-sm border-b border-gray-800 px-8 py-4 flex items-center justify-between">
            <div>
              <p className="text-xs text-gray-500 font-mono">
                Kaspa Testnet-12 | Demo Mode
              </p>
            </div>
            <WalletButton />
          </header>

          <main className="p-8">{children}</main>
        </div>
      </body>
    </html>
  );
}
