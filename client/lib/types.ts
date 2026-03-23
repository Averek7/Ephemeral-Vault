export type VaultStatus = "active" | "paused" | "inactive" | "expired";
export type VaultSessionStatus =
  | "no_session"
  | "active"
  | "expiring_soon"
  | "expired";

export interface VaultAccount {
  address: string;
  owner: string;
  delegate: string | null;
  approvedAmountLamports: number;
  availableAmountLamports: number;
  usedAmountLamports: number;
  totalDepositedLamports: number;
  totalWithdrawnLamports: number;
  approvedAmountSol: number;
  availableAmountSol: number;
  usedAmountSol: number;
  totalDepositedSol: number;
  totalWithdrawnSol: number;
  tradeCount: number;
  sessionExpiry: number | null;
  delegatedAt: number | null;
  createdAt: number;
  lastActivity: number;
  isActive: boolean;
  isPaused: boolean;
  sessionStatus: VaultSessionStatus;
  status: VaultStatus;
  version: number;
  bump: number;
}

export interface BackendTradeRecord {
  id: string;
  vault_address: string;
  tx_hash: string;
  trade_type: string;
  amount_sol: number;
  fee_sol: number;
  status: "success" | "pending" | "failed";
  slot: number | null;
  created_at: string;
}

export interface Trade {
  id: string;
  type: "Swap" | "Buy" | "Sell";
  amount: number;
  fee: number;
  status: "success" | "pending" | "failed";
  timestamp: number;
  txHash: string;
}

export interface CreateVaultParams {
  approvedAmount: number;
  delegate: string;
  sessionDuration: number;
  initialDeposit: number;
}

export interface ActivityChartData {
  timestamp: number;
  label: string;
  volume: number;
  cumulative: number;
}

export type ToastType = "success" | "warning" | "error" | "info";

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}
