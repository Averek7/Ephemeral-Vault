'use client';

import React, { useState } from 'react';
import { Wallet, ChevronDown, LogOut, Copy, ExternalLink } from 'lucide-react';
import { useVault } from '@/contexts/VaultContext';

export function WalletButton() {
  const { walletConnected, walletAddress, connectWallet, disconnectWallet } = useVault();
  const [menuOpen, setMenuOpen] = useState(false);

  if (!walletConnected) {
    return (
      <button
        onClick={connectWallet}
        className="flex items-center gap-2 px-4 py-2 rounded-lg bg-sol-purple text-white text-sm font-semibold hover:bg-sol-purple/80 transition-all duration-150 shadow-lg shadow-sol-purple/30"
      >
        <Wallet size={15} />
        Connect Wallet
      </button>
    );
  }

  return (
    <div className="relative">
      <button
        onClick={() => setMenuOpen(v => !v)}
        className="flex items-center gap-2 px-3 py-2 rounded-lg bg-vault-surface border border-vault-border text-sm text-white hover:border-sol-purple/40 transition-all"
      >
        <div className="w-5 h-5 rounded-full bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center">
          <div className="w-2 h-2 rounded-full bg-sol-green pulse-dot" />
        </div>
        <span className="mono text-xs">{walletAddress}</span>
        <ChevronDown size={13} className={`text-vault-muted transition-transform ${menuOpen ? 'rotate-180' : ''}`} />
      </button>
      {menuOpen && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setMenuOpen(false)} />
          <div className="absolute right-0 top-full mt-2 z-20 w-44 rounded-xl border border-vault-border bg-vault-surface shadow-xl shadow-black/50 overflow-hidden">
            <button
              className="w-full flex items-center gap-2 px-4 py-2.5 text-xs text-vault-muted hover:text-white hover:bg-white/5 transition-colors"
              onClick={() => { navigator.clipboard.writeText(walletAddress); setMenuOpen(false); }}
            >
              <Copy size={13} /> Copy Address
            </button>
            <button
              className="w-full flex items-center gap-2 px-4 py-2.5 text-xs text-vault-muted hover:text-white hover:bg-white/5 transition-colors"
            >
              <ExternalLink size={13} /> View on Explorer
            </button>
            <div className="border-t border-vault-border" />
            <button
              className="w-full flex items-center gap-2 px-4 py-2.5 text-xs text-red-400 hover:text-red-300 hover:bg-red-500/5 transition-colors"
              onClick={() => { disconnectWallet(); setMenuOpen(false); }}
            >
              <LogOut size={13} /> Disconnect
            </button>
          </div>
        </>
      )}
    </div>
  );
}
