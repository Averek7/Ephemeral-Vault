'use client';

import React, { useState } from 'react';
import { History, CheckCircle2, XCircle, Clock, ExternalLink } from 'lucide-react';
import { Card, CardHeader } from '@/components/common/Card';
import { useVault } from '@/contexts/VaultContext';
import { formatTimestamp, formatSOL } from '@/lib/mock';
import { clsx } from 'clsx';
import { Trade } from '@/lib/types';

export function TradeHistory() {
  const { trades } = useVault();
  const [filter, setFilter] = useState<'all' | 'success' | 'failed'>('all');

  const filtered = trades.filter(t => filter === 'all' || t.status === filter);

  return (
    <Card>
      <CardHeader
        title="Recent Trades"
        icon={<History size={16} />}
        action={
          <div className="flex items-center gap-1 bg-vault-bg rounded-lg p-0.5 border border-vault-border">
            {(['all', 'success', 'failed'] as const).map(f => (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={clsx(
                  'px-2.5 py-1 rounded-md text-xs font-medium transition-all capitalize',
                  filter === f ? 'bg-sol-purple/20 text-sol-purple' : 'text-vault-muted hover:text-white'
                )}
              >
                {f}
              </button>
            ))}
          </div>
        }
      />

      {filtered.length === 0 ? (
        <div className="py-12 text-center text-vault-muted text-sm">
          No trades yet
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-vault-border">
                {['Time', 'Type', 'Amount', 'Fee', 'Status', 'Tx'].map(h => (
                  <th key={h} className="text-left pb-3 text-xs text-vault-muted uppercase tracking-wider font-medium pr-4 last:pr-0">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map((trade, i) => (
                <TradeRow key={trade.id} trade={trade} index={i} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </Card>
  );
}

function TradeRow({ trade, index }: { trade: Trade; index: number }) {
  const typeColors: Record<string, string> = {
    Swap: 'text-sol-purple bg-sol-purple/10 border-sol-purple/20',
    Buy: 'text-sol-green bg-sol-green/10 border-sol-green/20',
    Sell: 'text-sol-coral bg-sol-coral/10 border-sol-coral/20',
  };

  return (
    <tr
      className="border-b border-vault-border/40 hover:bg-white/2 transition-colors"
      style={{ animationDelay: `${index * 50}ms` }}
    >
      <td className="py-3 pr-4 text-vault-muted text-xs mono">{formatTimestamp(trade.timestamp)}</td>
      <td className="py-3 pr-4">
        <span className={clsx('px-2 py-0.5 rounded-md text-xs font-mono border', typeColors[trade.type] || 'text-vault-muted')}>
          {trade.type}
        </span>
      </td>
      <td className="py-3 pr-4 font-bold mono text-white">{formatSOL(trade.amount)} <span className="text-vault-muted font-normal">SOL</span></td>
      <td className="py-3 pr-4 text-vault-muted mono text-xs">{trade.fee}</td>
      <td className="py-3 pr-4">
        {trade.status === 'success' ? (
          <span className="flex items-center gap-1 text-sol-green text-xs">
            <CheckCircle2 size={12} /> Done
          </span>
        ) : trade.status === 'pending' ? (
          <span className="flex items-center gap-1 text-amber-400 text-xs">
            <Clock size={12} className="animate-spin" /> Pending
          </span>
        ) : (
          <span className="flex items-center gap-1 text-red-400 text-xs">
            <XCircle size={12} /> Failed
          </span>
        )}
      </td>
      <td className="py-3">
        <button className="text-vault-muted hover:text-sol-purple transition-colors">
          <ExternalLink size={12} />
        </button>
      </td>
    </tr>
  );
}
