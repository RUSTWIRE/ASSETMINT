// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT

"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Coins,
  ArrowLeftRight,
  ShieldAlert,
  FileBox,
  Vault,
  Gem,
  Settings,
} from "lucide-react";
import { cn } from "@/lib/cn";

const navItems = [
  { href: "/", label: "Dashboard", icon: LayoutDashboard },
  { href: "/mint", label: "Mint", icon: Coins },
  { href: "/transfer", label: "Transfer", icon: ArrowLeftRight },
  { href: "/clawback", label: "Clawback", icon: ShieldAlert },
  { href: "/assets", label: "Assets", icon: FileBox },
  { href: "/reserves", label: "Reserves", icon: Vault },
  { href: "/astm", label: "ASTM Token", icon: Gem },
  { href: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="fixed left-0 top-8 bottom-0 w-64 bg-gray-900 border-r border-gray-800 flex flex-col z-40">
      <div className="p-6 border-b border-gray-800">
        <h1 className="text-xl font-bold text-white tracking-tight">
          AssetMint
        </h1>
        <p className="text-xs text-gray-500 mt-1">
          RWA Tokenization on Kaspa
        </p>
      </div>

      <nav className="flex-1 p-4 space-y-1 overflow-y-auto" aria-label="Main navigation">
        {navItems.map((item) => {
          const isActive = pathname === item.href;
          const Icon = item.icon;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors",
                isActive
                  ? "bg-indigo-600 text-white"
                  : "text-gray-400 hover:text-white hover:bg-gray-800"
              )}
              aria-current={isActive ? "page" : undefined}
            >
              <Icon className="w-5 h-5 shrink-0" />
              {item.label}
            </Link>
          );
        })}
      </nav>

      <div className="p-4 border-t border-gray-800">
        <div className="text-xs text-gray-600">
          <p>Network: Testnet-12</p>
          <p className="truncate">ws://tn12-node.kaspa.com:17210</p>
        </div>
      </div>
    </aside>
  );
}
