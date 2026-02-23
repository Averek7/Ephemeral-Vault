export interface VaultAccount {
  address: string;
  owner: string;
  delegate: string;
  approvedAmount: number;
  currentBalance: number;
  totalDeposited: number;
  totalWithdrawn: number;
  tradesExecuted: number;
  sessionExpiry: number; // unix timestamp
  status: 'active' | 'paused' | 'revoked' | 'expired';
  createdAt: number;
}

export interface Trade {
  id: string;
  type: 'Swap' | 'Buy' | 'Sell';
  amount: number;
  fee: number;
  status: 'success' | 'pending' | 'failed';
  timestamp: number;
  txHash: string;
}

export interface SessionStatus {
  isActive: boolean;
  expiresAt: number;
  secondsRemaining: number;
  delegate: string;
  lastActivity: number;
}

export interface CreateVaultParams {
  approvedAmount: number;
  delegate: string;
  sessionDuration: number; // minutes
  initialDeposit: number;
}

export interface ActivityChartData {
  timestamp: number;
  label: string;
  volume: number;
  cumulative: number;
}

export type ToastType = 'success' | 'warning' | 'error' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}
