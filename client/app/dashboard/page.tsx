"use client";

import React, { useEffect } from "react";
import Link from "next/link";
import { Plus, LayoutDashboard } from "lucide-react";
import { Navbar } from "@/components/common/Navbar";
import { VaultOverview } from "@/components/vault/VaultOverview";
import { SessionManager } from "@/components/vault/SessionManager";
import { TradeHistory } from "@/components/trading/TradeHistory";
import { ActivityChart } from "@/components/trading/ActivityChart";
import { Button } from "@/components/common/Button";
import { useVault } from "@/contexts/VaultContext";
import { MOCK_VAULT, MOCK_TRADES } from "@/lib/mock";

export default function DashboardPage() {
  const { vault, walletConnected, isLoading } = useVault();

  // For demo: auto-load mock vault when visiting dashboard
  const { connectWallet } = useVault();

  if (!walletConnected) {
    return (
      <div className="min-h-screen grid-texture">
        <Navbar />
        <div className="max-w-7xl mx-auto px-6 py-24 flex flex-col items-center text-center gap-6">
          <div className="w-16 h-16 rounded-2xl bg-sol-purple/10 border border-sol-purple/30 flex items-center justify-center">
            <LayoutDashboard size={28} className="text-sol-purple" />
          </div>
          <h1 className="text-3xl font-bold">Connect Your Wallet</h1>
          <p className="text-vault-muted max-w-sm">
            Connect your Solana wallet to access the dashboard and manage your
            vault.
          </p>
          <Button onClick={connectWallet} size="lg">
            Connect Wallet
          </Button>
        </div>
      </div>
    );
  }

  if (!vault) {
    return (
      <div className="min-h-screen grid-texture">
        <Navbar />
        <div className="max-w-7xl mx-auto px-6 py-24 flex flex-col items-center text-center gap-6">
          <div className="w-16 h-16 rounded-2xl bg-vault-surface border border-vault-border flex items-center justify-center">
            <Plus size={28} className="text-vault-muted" />
          </div>
          <h1 className="text-3xl font-bold">No Vault Found</h1>
          <p className="text-vault-muted max-w-sm">
            You don't have an active vault yet. Create one to start delegating
            trading access.
          </p>
          <div className="flex gap-3">
            <Link href="/create">
              <Button size="lg">
                <Plus size={18} /> Create Vault
              </Button>
            </Link>
            <button
              onClick={() => document.getElementById("load-demo")?.click()}
              className="px-6 py-3 rounded-xl border border-vault-border text-vault-muted hover:text-white hover:border-sol-purple/40 transition-all text-sm"
            >
              Load Demo
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen grid-texture">
      <Navbar />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-8 space-y-5">
        {/* Top row */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
          <VaultOverview />
          <SessionManager />
        </div>

        {/* Chart */}
        <ActivityChart />

        {/* Trade history */}
        <TradeHistory />
      </main>
    </div>
  );
}
