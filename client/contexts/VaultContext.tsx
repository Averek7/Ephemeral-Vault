"use client";

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
} from "react";
import { VaultAccount, Trade, CreateVaultParams } from "@/lib/types";
import { MOCK_VAULT, MOCK_TRADES } from "@/lib/mock";
import { useNotification } from "./NotificationContext";

interface VaultContextType {
  vault: VaultAccount | null;
  trades: Trade[];
  walletConnected: boolean;
  walletAddress: string;
  isLoading: boolean;

  connectWallet: () => void;
  disconnectWallet: () => void;
  createVault: (params: CreateVaultParams) => Promise<void>;
  deposit: (amount: number) => Promise<void>;
  withdraw: (amount: number) => Promise<void>;
  pauseVault: () => Promise<void>;
  unpauseVault: () => Promise<void>;
  revokeAccess: () => Promise<void>;
  renewSession: () => Promise<void>;
}

const VaultContext = createContext<VaultContextType>({} as VaultContextType);

export function VaultProvider({ children }: { children: React.ReactNode }) {
  const [vault, setVault] = useState<VaultAccount | null>(null);
  const [trades, setTrades] = useState<Trade[]>([]);
  const [walletConnected, setWalletConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { addToast } = useNotification();

  // Simulate real-time trade updates
  useEffect(() => {
    if (!vault || vault.status !== "active") return;
    const interval = setInterval(() => {
      const newTrade: Trade = {
        id: Math.random().toString(36).slice(2),
        type: ["Swap", "Buy", "Sell"][
          Math.floor(Math.random() * 3)
        ] as Trade["type"],
        amount: parseFloat((Math.random() * 0.5 + 0.1).toFixed(3)),
        fee: 0.001,
        status: Math.random() > 0.1 ? "success" : "failed",
        timestamp: Date.now(),
        txHash:
          Math.random().toString(36).slice(2, 8) +
          "..." +
          Math.random().toString(36).slice(2, 6),
      };
      setTrades((prev) => [newTrade, ...prev.slice(0, 49)]);
      setVault((prev) =>
        prev
          ? {
              ...prev,
              tradesExecuted: prev.tradesExecuted + 1,
              currentBalance: parseFloat(
                Math.max(
                  0,
                  prev.currentBalance - newTrade.amount * 0.3,
                ).toFixed(3),
              ),
            }
          : prev,
      );
      addToast(
        `Trade executed: ${newTrade.amount} SOL ${newTrade.type.toLowerCase()}`,
        "success",
      );
    }, 15000);
    return () => clearInterval(interval);
  }, [vault?.status, addToast]);

  const connectWallet = useCallback(() => {
    setWalletConnected(true);
    setWalletAddress("7hNm...3kRt");
    addToast("Wallet connected successfully", "success");
  }, [addToast]);

  const disconnectWallet = useCallback(() => {
    setWalletConnected(false);
    setWalletAddress("");
    setVault(null);
    setTrades([]);
    addToast("Wallet disconnected", "info");
  }, [addToast]);

  const createVault = useCallback(
    async (params: CreateVaultParams) => {
      setIsLoading(true);
      await new Promise((r) => setTimeout(r, 2000));
      const newVault: VaultAccount = {
        address: "Ev8u...4xPq",
        owner: walletAddress,
        delegate: params.delegate,
        approvedAmount: params.approvedAmount,
        currentBalance: params.initialDeposit,
        totalDeposited: params.initialDeposit,
        totalWithdrawn: 0,
        tradesExecuted: 0,
        sessionExpiry: Date.now() + params.sessionDuration * 60 * 1000,
        status: "active",
        createdAt: Date.now(),
      };
      setVault(newVault);
      setTrades([]);
      setIsLoading(false);
      addToast("Vault created successfully!", "success");
    },
    [walletAddress, addToast],
  );

  const deposit = useCallback(
    async (amount: number) => {
      setIsLoading(true);
      await new Promise((r) => setTimeout(r, 1500));
      setVault((prev) =>
        prev
          ? {
              ...prev,
              currentBalance: parseFloat(
                (prev.currentBalance + amount).toFixed(3),
              ),
              totalDeposited: parseFloat(
                (prev.totalDeposited + amount).toFixed(3),
              ),
            }
          : prev,
      );
      setIsLoading(false);
      addToast(`Deposited ${amount} SOL to vault`, "success");
    },
    [addToast],
  );

  const withdraw = useCallback(
    async (amount: number) => {
      setIsLoading(true);
      await new Promise((r) => setTimeout(r, 1500));
      setVault((prev) =>
        prev
          ? {
              ...prev,
              currentBalance: parseFloat(
                Math.max(0, prev.currentBalance - amount).toFixed(3),
              ),
              totalWithdrawn: parseFloat(
                (prev.totalWithdrawn + amount).toFixed(3),
              ),
            }
          : prev,
      );
      setIsLoading(false);
      addToast(`Withdrew ${amount} SOL from vault`, "success");
    },
    [addToast],
  );

  const pauseVault = useCallback(async () => {
    setIsLoading(true);
    await new Promise((r) => setTimeout(r, 1000));
    setVault((prev) => (prev ? { ...prev, status: "paused" } : prev));
    setIsLoading(false);
    addToast("Vault paused — all trading stopped", "warning");
  }, [addToast]);

  const unpauseVault = useCallback(async () => {
    setIsLoading(true);
    await new Promise((r) => setTimeout(r, 1000));
    setVault((prev) => (prev ? { ...prev, status: "active" } : prev));
    setIsLoading(false);
    addToast("Vault resumed", "success");
  }, [addToast]);

  const revokeAccess = useCallback(async () => {
    setIsLoading(true);
    await new Promise((r) => setTimeout(r, 2000));
    setVault((prev) =>
      prev
        ? {
            ...prev,
            status: "revoked",
            currentBalance: 0,
            totalWithdrawn: prev.totalWithdrawn + prev.currentBalance,
          }
        : prev,
    );
    setIsLoading(false);
    addToast("Access revoked — funds returned to wallet", "info");
  }, [addToast]);

  const renewSession = useCallback(async () => {
    setIsLoading(true);
    await new Promise((r) => setTimeout(r, 1000));
    setVault((prev) =>
      prev ? { ...prev, sessionExpiry: Date.now() + 60 * 60 * 1000 } : prev,
    );
    setIsLoading(false);
    addToast("Session renewed for 1 hour", "success");
  }, [addToast]);

  // Load demo vault when wallet connects
  const loadDemo = useCallback(() => {
    setVault(MOCK_VAULT);
    setTrades(MOCK_TRADES);
  }, []);

  return (
    <VaultContext.Provider
      value={{
        vault,
        trades,
        walletConnected,
        walletAddress,
        isLoading,
        connectWallet,
        disconnectWallet,
        createVault,
        deposit,
        withdraw,
        pauseVault,
        unpauseVault,
        revokeAccess,
        renewSession,
      }}
    >
      {children}
      {/* Hidden demo loader */}
      {walletConnected && !vault && (
        <button onClick={loadDemo} className="hidden" id="load-demo" />
      )}
    </VaultContext.Provider>
  );
}

export const useVault = () => useContext(VaultContext);
