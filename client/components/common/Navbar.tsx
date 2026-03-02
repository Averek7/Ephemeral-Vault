"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Shield } from "lucide-react";
import { WalletButton } from "@/components/wallet/WalletButton";

export function Navbar() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-40 border-b border-vault-border bg-vault-bg/60 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 h-14 flex items-center justify-between">
        <Link
          href="/"
          className="flex items-center gap-2.5 group flex-shrink-0"
        >
          <div className="w-7 h-7 rounded-lg bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center group-hover:border-sol-purple/70 transition-colors">
            <Shield size={14} className="text-sol-purple" />
          </div>
          <span className="text-sm font-bold tracking-tight">
            Exec<span className="text-sol-purple">Vault</span>
          </span>
        </Link>

        <nav className="hidden md:flex items-center gap-1">
          {pathname === "/" ? (
            <>
              <a
                href="#how"
                className="px-3 py-1.5 rounded-lg text-xs text-vault-muted hover:text-white hover:bg-white/5 transition-colors"
              >
                How it works
              </a>
              <a
                href="#security"
                className="px-3 py-1.5 rounded-lg text-xs text-vault-muted hover:text-white hover:bg-white/5 transition-colors"
              >
                Security
              </a>
              <a
                href="#core"
                className="px-3 py-1.5 rounded-lg text-xs text-vault-muted hover:text-white hover:bg-white/5 transition-colors"
              >
                Core
              </a>
            </>
          ) : (
            <></>
          )}
        </nav>

        <WalletButton />
      </div>
    </header>
  );
}
