import { ActivityChartData, Trade, VaultAccount } from './types';

const nowSeconds = Math.floor(Date.now() / 1000);

export const MOCK_VAULT: VaultAccount = {
  address: 'Ev8uMock4xPq',
  owner: '7hNmMock3kRt',
  delegate: '8xKdMockm9Pq',
  approvedAmountLamports: 10_000_000_000,
  availableAmountLamports: 2_500_000_000,
  usedAmountLamports: 1_250_000_000,
  totalDepositedLamports: 5_000_000_000,
  totalWithdrawnLamports: 2_500_000_000,
  approvedAmountSol: 10,
  availableAmountSol: 2.5,
  usedAmountSol: 1.25,
  totalDepositedSol: 5,
  totalWithdrawnSol: 2.5,
  tradeCount: 23,
  sessionExpiry: nowSeconds + 45 * 60,
  delegatedAt: nowSeconds - 15 * 60,
  createdAt: nowSeconds - 3 * 60 * 60,
  lastActivity: nowSeconds - 2 * 60,
  isActive: true,
  isPaused: false,
  sessionStatus: 'active',
  status: 'active',
  version: 1,
  bump: 255,
};

export const MOCK_TRADES: Trade[] = [
  { id: '1', type: 'Swap', amount: 0.5, fee: 0.001, status: 'success', timestamp: Date.now() - 2 * 60 * 1000, txHash: 'abc...123' },
  { id: '2', type: 'Swap', amount: 0.3, fee: 0.001, status: 'success', timestamp: Date.now() - 5 * 60 * 1000, txHash: 'def...456' },
  { id: '3', type: 'Swap', amount: 1.0, fee: 0.002, status: 'success', timestamp: Date.now() - 12 * 60 * 1000, txHash: 'ghi...789' },
  { id: '4', type: 'Buy', amount: 0.2, fee: 0.001, status: 'success', timestamp: Date.now() - 20 * 60 * 1000, txHash: 'jkl...012' },
  { id: '5', type: 'Swap', amount: 0.8, fee: 0.002, status: 'failed', timestamp: Date.now() - 35 * 60 * 1000, txHash: 'mno...345' },
  { id: '6', type: 'Sell', amount: 0.4, fee: 0.001, status: 'success', timestamp: Date.now() - 50 * 60 * 1000, txHash: 'pqr...678' },
  { id: '7', type: 'Swap', amount: 0.6, fee: 0.001, status: 'success', timestamp: Date.now() - 70 * 60 * 1000, txHash: 'stu...901' },
];

export function generateChartData(): ActivityChartData[] {
  const now = Date.now();
  const data: ActivityChartData[] = [];
  let cumulative = 0;
  for (let i = 23; i >= 0; i--) {
    const volume = Math.random() * 0.8 + (i > 18 || i < 6 ? 0 : 0.3);
    cumulative += volume;
    const ts = now - i * 60 * 60 * 1000;
    const d = new Date(ts);
    data.push({
      timestamp: ts,
      label: d.getHours().toString().padStart(2, '0') + ':00',
      volume: parseFloat(volume.toFixed(3)),
      cumulative: parseFloat(cumulative.toFixed(3)),
    });
  }
  return data;
}

export function formatTimestamp(ts: number): string {
  const diff = Date.now() - ts;
  const mins = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  if (hours < 24) return `${hours}h ago`;
  return new Date(ts).toLocaleDateString();
}

export function formatAddress(addr: string): string {
  if (addr.includes('...')) return addr;
  return addr.slice(0, 4) + '...' + addr.slice(-4);
}

export function formatSOL(amount: number): string {
  return amount.toFixed(3);
}

export function getSecondsRemaining(expiry: number): number {
  return Math.max(0, Math.floor((expiry - Date.now()) / 1000));
}

export function formatCountdown(seconds: number): string {
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = (seconds % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}

export function getTimerColor(seconds: number): string {
  if (seconds > 30 * 60) return '#14F195';
  if (seconds > 10 * 60) return '#F59E0B';
  return '#EF4444';
}

export function getTimerPercent(sessionDurationMinutes: number, secondsRemaining: number): number {
  const total = sessionDurationMinutes * 60;
  return Math.max(0, Math.min(100, (secondsRemaining / total) * 100));
}
