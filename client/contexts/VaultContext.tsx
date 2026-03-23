"use client";

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
  useRef,
} from "react";
import {
  BackendTradeRecord,
  CreateVaultParams,
  Trade,
  VaultAccount,
} from "@/lib/types";
import { useNotification } from "./NotificationContext";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { Transaction } from "@solana/web3.js";
import { apiGet, apiPost, ApiError } from "@/lib/api";

interface VaultContextType {
  vault: VaultAccount | null;
  trades: Trade[];
  walletConnected: boolean;
  walletAddress: string;
  isLoading: boolean;

  createVault: (params: CreateVaultParams) => Promise<void>;
  deposit: (amount: number) => Promise<void>;
  withdraw: (amount: number) => Promise<void>;
  pauseVault: () => Promise<void>;
  unpauseVault: () => Promise<void>;
  revokeAccess: () => Promise<void>;
  renewSession: () => Promise<void>;
  approveDelegate: (delegate: string, sessionDurationMinutes?: number) => Promise<void>;
  reactivateVault: () => Promise<void>;
  updateApprovedAmount: (newApprovedAmount: number) => Promise<void>;
}

const VaultContext = createContext<VaultContextType>({} as VaultContextType);

type TxResponse = { transactionBase64: string; vaultPda: string };

function lamportsFromSol(sol: number): number {
  if (!Number.isFinite(sol) || sol < 0) return 0;
  return Math.round(sol * 1_000_000_000);
}

function decodeBase64Tx(base64: string): Uint8Array {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

function normalizeTradeType(tradeType: string): Trade["type"] {
  const normalized = tradeType.trim().toLowerCase();
  if (normalized === "buy") return "Buy";
  if (normalized === "sell") return "Sell";
  return "Swap";
}

function toTrade(record: BackendTradeRecord): Trade {
  return {
    id: record.id,
    type: normalizeTradeType(record.trade_type),
    amount: record.amount_sol,
    fee: record.fee_sol,
    status: record.status,
    timestamp: new Date(record.created_at).getTime(),
    txHash: record.tx_hash,
  };
}

export function VaultProvider({ children }: { children: React.ReactNode }) {
  const [vault, setVault] = useState<VaultAccount | null>(null);
  const [trades, setTrades] = useState<Trade[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { addToast } = useNotification();
  const { connection } = useConnection();
  const { connected, publicKey, sendTransaction } = useWallet();
  const walletConnected = connected;
  const walletAddress = publicKey?.toBase58() ?? "";
  const hasSeenConnectionState = useRef(false);
  const effectiveVault = walletConnected ? vault : null;
  const effectiveTrades = walletConnected ? trades : [];

  const loadVault = useCallback(async () => {
    if (!walletConnected || !walletAddress) return;

    try {
      const v = await apiGet<VaultAccount>(`/vault/${walletAddress}`);
      setVault(v);
      try {
        const t = await apiGet<BackendTradeRecord[]>(
          `/trades/${v.address}?limit=50&offset=0`,
        );
        setTrades(t.map(toTrade));
      } catch (e) {
        setTrades([]);
      }
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) {
        setVault(null);
        setTrades([]);
        return;
      }
      throw e;
    }
  }, [walletConnected, walletAddress]);

  const sendBase64Tx = useCallback(
    async (resp: TxResponse) => {
      if (!walletConnected || !publicKey) {
        addToast("Connect your wallet first", "warning");
        return;
      }

      const tx = Transaction.from(decodeBase64Tx(resp.transactionBase64));
      const sig = await sendTransaction(tx, connection);
      await connection.confirmTransaction(sig, "confirmed");
      return sig;
    },
    [walletConnected, publicKey, sendTransaction, connection, addToast],
  );

  useEffect(() => {
    if (!hasSeenConnectionState.current) {
      hasSeenConnectionState.current = true;
      return;
    }

    if (walletConnected) {
      addToast("Wallet connected successfully", "success");
      return;
    }

    addToast("Wallet disconnected", "info");
    setVault(null);
    setTrades([]);
  }, [walletConnected, addToast]);

  useEffect(() => {
    if (!walletConnected || !walletAddress) return;
    loadVault().catch((e) => {
      addToast(e instanceof Error ? e.message : "Failed to load vault", "error");
    });
  }, [walletConnected, walletAddress, loadVault, addToast]);

  const createVault = useCallback(
    async (params: CreateVaultParams) => {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet to create a vault", "warning");
        return;
      }
      setIsLoading(true);
      try {
        const resp = await apiPost<TxResponse>("/tx/create_vault", {
          userPubkey: walletAddress,
          approvedAmountLamports: lamportsFromSol(params.approvedAmount),
          delegatePubkey: params.delegate?.trim() ? params.delegate.trim() : null,
          customDurationSeconds:
            params.sessionDuration && params.sessionDuration > 0
              ? Math.round(params.sessionDuration * 60)
              : null,
          initialDepositLamports:
            params.initialDeposit && params.initialDeposit > 0
              ? lamportsFromSol(params.initialDeposit)
              : null,
        });
        await sendBase64Tx(resp);
        await loadVault();
        addToast("Vault created successfully!", "success");
      } catch (e) {
        addToast(e instanceof Error ? e.message : "Create vault failed", "error");
      } finally {
        setIsLoading(false);
      }
    },
    [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault],
  );

  const deposit = useCallback(
    async (amount: number) => {
      setIsLoading(true);
      try {
        if (!walletConnected || !walletAddress) {
          addToast("Connect your wallet first", "warning");
          return;
        }
        const resp = await apiPost<TxResponse>("/tx/deposit", {
          userPubkey: walletAddress,
          amountLamports: lamportsFromSol(amount),
        });
        await sendBase64Tx(resp);
        await loadVault();
        addToast(`Deposited ${amount} SOL to vault`, "success");
      } catch (e) {
        addToast(e instanceof Error ? e.message : "Deposit failed", "error");
      } finally {
        setIsLoading(false);
      }
    },
    [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault],
  );

  const withdraw = useCallback(
    async (amount: number) => {
      setIsLoading(true);
      try {
        if (!walletConnected || !walletAddress) {
          addToast("Connect your wallet first", "warning");
          return;
        }
        const resp = await apiPost<TxResponse>("/tx/withdraw", {
          userPubkey: walletAddress,
          amountLamports: amount > 0 ? lamportsFromSol(amount) : 0,
        });
        await sendBase64Tx(resp);
        await loadVault();
        addToast(
          amount > 0 ? `Withdrew ${amount} SOL from vault` : "Withdrew all from vault",
          "success",
        );
      } catch (e) {
        addToast(e instanceof Error ? e.message : "Withdraw failed", "error");
      } finally {
        setIsLoading(false);
      }
    },
    [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault],
  );

  const pauseVault = useCallback(async () => {
    setIsLoading(true);
    try {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet first", "warning");
        return;
      }
      const resp = await apiPost<TxResponse>("/tx/pause", {
        userPubkey: walletAddress,
      });
      await sendBase64Tx(resp);
      await loadVault();
      addToast("Vault paused — all trading stopped", "warning");
    } catch (e) {
      addToast(e instanceof Error ? e.message : "Pause failed", "error");
    } finally {
      setIsLoading(false);
    }
  }, [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault]);

  const unpauseVault = useCallback(async () => {
    setIsLoading(true);
    try {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet first", "warning");
        return;
      }
      const resp = await apiPost<TxResponse>("/tx/unpause", {
        userPubkey: walletAddress,
      });
      await sendBase64Tx(resp);
      await loadVault();
      addToast("Vault resumed", "success");
    } catch (e) {
      addToast(e instanceof Error ? e.message : "Unpause failed", "error");
    } finally {
      setIsLoading(false);
    }
  }, [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault]);

  const revokeAccess = useCallback(async () => {
    setIsLoading(true);
    try {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet first", "warning");
        return;
      }
      const resp = await apiPost<TxResponse>("/tx/revoke", {
        userPubkey: walletAddress,
      });
      await sendBase64Tx(resp);
      await loadVault();
      addToast("Access revoked — funds returned to wallet", "info");
    } catch (e) {
      addToast(e instanceof Error ? e.message : "Revoke failed", "error");
    } finally {
      setIsLoading(false);
    }
  }, [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault]);

  const renewSession = useCallback(async () => {
    setIsLoading(true);
    try {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet first", "warning");
        return;
      }
      const resp = await apiPost<TxResponse>("/tx/renew_session", {
        userPubkey: walletAddress,
      });
      await sendBase64Tx(resp);
      await loadVault();
      addToast("Session renewed", "success");
    } catch (e) {
      addToast(e instanceof Error ? e.message : "Renew session failed", "error");
    } finally {
      setIsLoading(false);
    }
  }, [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault]);

  const approveDelegate = useCallback(
    async (delegate: string, sessionDurationMinutes?: number) => {
      setIsLoading(true);
      try {
        if (!walletConnected || !walletAddress) {
          addToast("Connect your wallet first", "warning");
          return;
        }
        const resp = await apiPost<TxResponse>("/tx/approve_delegate", {
          userPubkey: walletAddress,
          delegatePubkey: delegate,
          customDurationSeconds:
            sessionDurationMinutes && sessionDurationMinutes > 0
              ? Math.round(sessionDurationMinutes * 60)
              : null,
        });
        await sendBase64Tx(resp);
        await loadVault();
        addToast("Delegate approved", "success");
      } catch (e) {
        addToast(e instanceof Error ? e.message : "Approve delegate failed", "error");
      } finally {
        setIsLoading(false);
      }
    },
    [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault],
  );

  const reactivateVault = useCallback(async () => {
    setIsLoading(true);
    try {
      if (!walletConnected || !walletAddress) {
        addToast("Connect your wallet first", "warning");
        return;
      }
      const resp = await apiPost<TxResponse>("/tx/reactivate", {
        userPubkey: walletAddress,
      });
      await sendBase64Tx(resp);
      await loadVault();
      addToast("Vault reactivated", "success");
    } catch (e) {
      addToast(e instanceof Error ? e.message : "Reactivate failed", "error");
    } finally {
      setIsLoading(false);
    }
  }, [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault]);

  const updateApprovedAmount = useCallback(
    async (newApprovedAmount: number) => {
      setIsLoading(true);
      try {
        if (!walletConnected || !walletAddress) {
          addToast("Connect your wallet first", "warning");
          return;
        }
        const resp = await apiPost<TxResponse>("/tx/update_approved_amount", {
          userPubkey: walletAddress,
          newApprovedAmountLamports: lamportsFromSol(newApprovedAmount),
        });
        await sendBase64Tx(resp);
        await loadVault();
        addToast("Approved amount updated", "success");
      } catch (e) {
        addToast(
          e instanceof Error ? e.message : "Update approved amount failed",
          "error",
        );
      } finally {
        setIsLoading(false);
      }
    },
    [walletConnected, walletAddress, addToast, sendBase64Tx, loadVault],
  );

  return (
    <VaultContext.Provider
      value={{
        vault: effectiveVault,
        trades: effectiveTrades,
        walletConnected,
        walletAddress,
        isLoading,
        createVault,
        deposit,
        withdraw,
        pauseVault,
        unpauseVault,
        revokeAccess,
        renewSession,
        approveDelegate,
        reactivateVault,
        updateApprovedAmount,
      }}
    >
      {children}
    </VaultContext.Provider>
  );
}

export const useVault = () => useContext(VaultContext);
