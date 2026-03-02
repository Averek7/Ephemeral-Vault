"use client";

import { useEffect, useState, useCallback } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletReadyState, WalletName } from "@solana/wallet-adapter-base";
import {
  Wallet,
  ChevronDown,
  LogOut,
  Copy,
  ExternalLink,
  RefreshCw,
  X,
} from "lucide-react";

// Names of errors that are safe to swallow — they all mean "user cancelled".
const IGNORED_WALLET_ERROR_NAMES = new Set([
  "WalletAccountError",
  "WalletNotSelectedError",
  "WalletWindowClosedError",
  "WalletConnectionError",
]);

function isIgnoredWalletError(err: unknown): boolean {
  return (
    err instanceof Error && IGNORED_WALLET_ERROR_NAMES.has(err.name)
  );
}

export function WalletButton() {
  const {
    wallets,
    select,
    connect,
    disconnect,
    connected,
    connecting,
    publicKey,
    wallet,
  } = useWallet();

  const [menuOpen, setMenuOpen] = useState(false);
  const [selectingWallet, setSelectingWallet] = useState(false);
  const [pendingWalletName, setPendingWalletName] = useState<WalletName | null>(
    null,
  );
  const isBusy = connecting || pendingWalletName !== null;

  const address = publicKey?.toBase58();

  // Deduplicate by adapter name — MetaMask and others can appear multiple times.
  const availableWallets = Array.from(
    new Map(
      wallets
        .filter((w) => w.readyState === WalletReadyState.Installed)
        .map((w) => [w.adapter.name, w]),
    ).values(),
  );

  const shortAddress = address
    ? `${address.slice(0, 4)}...${address.slice(-4)}`
    : "";

  const closeMenu = useCallback(() => {
    setMenuOpen(false);
    setSelectingWallet(false);
  }, []);

  const copyAddress = () => {
    if (!address) return;
    navigator.clipboard.writeText(address);
    closeMenu();
  };

  const viewExplorer = () => {
    if (!address) return;
    window.open(
      `https://explorer.solana.com/address/${address}?cluster=devnet`,
      "_blank",
    );
  };

  // When the adapter has finished switching to the pending wallet, connect.
  useEffect(() => {
    if (!pendingWalletName) return;
    if (wallet?.adapter.name !== pendingWalletName) return;

    let cancelled = false;

    const connectSelectedWallet = async () => {
      try {
        if (!connected) {
          await connect();
        }
        if (!cancelled) closeMenu();
      } catch (err) {
        if (!cancelled && !isIgnoredWalletError(err)) {
          console.error("Wallet connect failed:", err);
        }
      } finally {
        if (!cancelled) {
          setPendingWalletName(null);
        }
      }
    };

    connectSelectedWallet();

    return () => {
      cancelled = true;
    };
  }, [pendingWalletName, wallet?.adapter.name, connected, connect, closeMenu]);

  const handleSelectWallet = useCallback(
    async (walletName: WalletName) => {
      if (isBusy) return;

      // Already connected to the same wallet — just close.
      if (connected && wallet?.adapter.name === walletName) {
        closeMenu();
        return;
      }

      // Disconnect from current wallet first if switching.
      if (connected) {
        try {
          await disconnect();
        } catch {
          // ignore
        }
      }

      // If adapter is already set to this wallet, connect immediately.
      if (wallet?.adapter.name === walletName) {
        try {
          await connect();
          closeMenu();
        } catch (err) {
          if (!isIgnoredWalletError(err)) {
            console.error("Wallet connect failed:", err);
          }
        }
        return;
      }

      // Select first, then effect connects once adapter state updates.
      setPendingWalletName(walletName);
      select(walletName);
    },
    [isBusy, connected, wallet?.adapter.name, disconnect, connect, select, closeMenu],
  );

  /* ---------------- DISCONNECTED ---------------- */

  if (!connected) {
    return (
      <div className="relative">
        <button
          onClick={() => setMenuOpen(true)}
          disabled={isBusy || connecting}
          className="flex items-center gap-2 px-4 py-2.5 rounded-xl
          bg-gradient-to-r from-sol-purple to-sol-green
          text-white text-sm font-semibold
          shadow-lg shadow-sol-purple/30
          hover:scale-[1.02] transition
          disabled:opacity-60 disabled:cursor-not-allowed disabled:scale-100"
        >
          <Wallet size={16} />
          {isBusy || connecting ? "Connecting…" : "Connect Wallet"}
        </button>

        {menuOpen && (
          <>
            <div
              className="fixed inset-0 bg-black/40 backdrop-blur-sm z-40"
              onClick={closeMenu}
            />

            {/* Desktop Dropdown */}
            <div
              className="hidden sm:block absolute right-0 mt-3 w-64 z-50
              rounded-2xl bg-[#0f1115]/95 backdrop-blur-xl
              border border-white/10 shadow-2xl overflow-hidden"
            >
              {availableWallets.length === 0 ? (
                <div className="px-5 py-4 text-sm text-white/50">
                  No wallets detected. Install Phantom or Solflare.
                </div>
              ) : (
                availableWallets.map((w) => (
                  <button
                    key={w.adapter.name}
                    onClick={() => handleSelectWallet(w.adapter.name)}
                    className="wallet-item"
                  >
                    {w.adapter.name}
                  </button>
                ))
              )}
            </div>

            {/* Mobile Bottom Sheet */}
            <div
              className="sm:hidden fixed bottom-0 left-0 right-0 z-50
              rounded-t-3xl bg-[#0f1115]
              border-t border-white/10
              shadow-2xl p-6 space-y-3"
            >
              <div className="flex justify-between items-center mb-2">
                <span className="text-white font-semibold">Select Wallet</span>
                <X size={20} className="text-white/60" onClick={closeMenu} />
              </div>

              {availableWallets.length === 0 ? (
                <div className="text-sm text-white/50 text-center py-4">
                  No wallets detected. Install Phantom or Solflare.
                </div>
              ) : (
                availableWallets.map((w) => (
                  <button
                    key={w.adapter.name}
                    onClick={() => handleSelectWallet(w.adapter.name)}
                    className="wallet-item-large"
                  >
                    {w.adapter.name}
                  </button>
                ))
              )}
            </div>
          </>
        )}
      </div>
    );
  }

  /* ---------------- CONNECTED ---------------- */

  return (
    <div className="relative">
      <button
        onClick={() => setMenuOpen(true)}
        className="flex items-center gap-3 px-4 py-2.5 rounded-xl
        bg-white/5 backdrop-blur-md border border-white/10
        text-sm text-white hover:border-sol-purple/40 transition"
      >
        <div className="relative flex items-center justify-center w-4 h-4 shrink-0">
          <span className="absolute inline-flex h-4 w-4 rounded-full bg-emerald-400 opacity-30 animate-ping" />
          <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-emerald-400" />
        </div>

        <span className="mono text-xs tracking-wide">{shortAddress}</span>

        <ChevronDown size={16} />
      </button>

      {menuOpen && (
        <>
          <div
            className="fixed inset-0 bg-black/40 backdrop-blur-sm z-40"
            onClick={closeMenu}
          />

          {/* Desktop Dropdown — normal wallet options */}
          {!selectingWallet ? (
            <div
              className="hidden sm:block absolute right-0 mt-3 w-72 z-50
              rounded-2xl bg-[#0f1115]/95 backdrop-blur-xl
              border border-white/10 shadow-2xl overflow-hidden"
            >
              <div className="px-5 py-4 border-b border-white/10">
                <div className="text-xs text-white/50">Connected with</div>
                <div className="text-white font-semibold mt-1">
                  {wallet?.adapter.name}
                </div>
                <div className="text-xs text-white/40 mt-1 break-all">
                  {address}
                </div>
              </div>

              <button onClick={copyAddress} className="wallet-item">
                <Copy size={16} />
                Copy Address
              </button>

              <button onClick={viewExplorer} className="wallet-item">
                <ExternalLink size={16} />
                View on Explorer
              </button>

              <button
                onClick={() => setSelectingWallet(true)}
                className="wallet-item"
              >
                <RefreshCw size={16} />
                Switch Wallet
              </button>

              <div className="border-t border-white/10 my-1" />

              <button
                onClick={() => {
                  disconnect();
                  closeMenu();
                }}
                className="wallet-item text-red-400 hover:text-red-300"
              >
                <LogOut size={16} />
                Disconnect
              </button>
            </div>
          ) : (
            /* Desktop Dropdown — wallet picker for switching */
            <div
              className="hidden sm:block absolute right-0 mt-3 w-72 z-50
              rounded-2xl bg-[#0f1115]/95 backdrop-blur-xl
              border border-white/10 shadow-2xl overflow-hidden"
            >
              <div className="px-5 py-3 border-b border-white/10 flex items-center justify-between">
                <span className="text-white font-semibold text-sm">Switch Wallet</span>
                <button
                  onClick={() => setSelectingWallet(false)}
                  className="text-white/40 hover:text-white transition"
                >
                  <X size={16} />
                </button>
              </div>

              {availableWallets.map((w) => (
                <button
                  key={w.adapter.name}
                  onClick={() => handleSelectWallet(w.adapter.name)}
                  className={`wallet-item ${
                    w.adapter.name === wallet?.adapter.name
                      ? "text-sol-purple font-semibold"
                      : ""
                  }`}
                >
                  {w.adapter.name}
                  {w.adapter.name === wallet?.adapter.name && (
                    <span className="ml-auto text-xs text-white/40">current</span>
                  )}
                </button>
              ))}
            </div>
          )}

          {/* Mobile Bottom Sheet */}
          <div
            className="sm:hidden fixed bottom-0 left-0 right-0 z-50
            rounded-t-3xl bg-[#0f1115]
            border-t border-white/10
            shadow-2xl p-6 space-y-4"
          >
            {!selectingWallet ? (
              <>
                <div className="text-white font-semibold text-center">
                  Wallet Options
                </div>

                <div className="text-center text-xs text-white/50 break-all">
                  {shortAddress}
                </div>

                <button onClick={copyAddress} className="wallet-item-large">
                  Copy Address
                </button>

                <button onClick={viewExplorer} className="wallet-item-large">
                  View on Explorer
                </button>

                <button
                  onClick={() => setSelectingWallet(true)}
                  className="wallet-item-large"
                >
                  Switch Wallet
                </button>

                <button
                  onClick={() => {
                    disconnect();
                    closeMenu();
                  }}
                  className="wallet-item-large text-red-400"
                >
                  Disconnect
                </button>
              </>
            ) : (
              <>
                <div className="flex justify-between items-center">
                  <span className="text-white font-semibold">Switch Wallet</span>
                  <X
                    size={20}
                    className="text-white/60 cursor-pointer"
                    onClick={() => setSelectingWallet(false)}
                  />
                </div>

                {availableWallets.map((w) => (
                  <button
                    key={w.adapter.name}
                    onClick={() => handleSelectWallet(w.adapter.name)}
                    className={`wallet-item-large ${
                      w.adapter.name === wallet?.adapter.name
                        ? "border border-sol-purple/40 text-sol-purple"
                        : ""
                    }`}
                  >
                    {w.adapter.name}
                    {w.adapter.name === wallet?.adapter.name && (
                      <span className="ml-2 text-xs opacity-60">current</span>
                    )}
                  </button>
                ))}
              </>
            )}
          </div>
        </>
      )}
    </div>
  );
}
