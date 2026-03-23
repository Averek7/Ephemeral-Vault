'use client';

import React, { useState } from 'react';
import { Vault, TrendingUp, ArrowDownToLine, ArrowUpFromLine, Pause, Play } from 'lucide-react';
import { Card, CardHeader } from '@/components/common/Card';
import { Button } from '@/components/common/Button';
import { useVault } from '@/contexts/VaultContext';
import { formatSOL } from '@/lib/mock';
import { DepositModal } from '@/components/modals/DepositModal';
import { WithdrawModal } from '@/components/modals/WithdrawModal';
import { PauseModal } from '@/components/modals/PauseModal';

export function VaultOverview() {
  const { vault } = useVault();
  const [showDeposit, setShowDeposit] = useState(false);
  const [showWithdraw, setShowWithdraw] = useState(false);
  const [showPause, setShowPause] = useState(false);

  if (!vault) return null;

  const utilizationPct = vault.approvedAmountSol > 0
    ? (vault.availableAmountSol / vault.approvedAmountSol) * 100
    : 0;

  const statusColor = vault.status === 'active' ? '#14F195'
    : vault.status === 'paused' ? '#F59E0B'
    : '#EF4444';

  const statusLabel = vault.status.charAt(0).toUpperCase() + vault.status.slice(1);

  return (
    <>
      <Card elevated glow="purple">
        <CardHeader
          title="Vault Overview"
          icon={<Vault size={16} />}
          badge={
            <span className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-mono`}
              style={{ background: statusColor + '15', border: `1px solid ${statusColor}40`, color: statusColor }}>
              <span className="w-1.5 h-1.5 rounded-full inline-block pulse-dot" style={{ background: statusColor }} />
              {statusLabel}
            </span>
          }
          action={
            <span className="text-xs text-vault-muted mono">{vault.address}</span>
          }
        />

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-5">
          <StatBox
            label="Available"
            value={formatSOL(vault.availableAmountSol) + ' SOL'}
            sub={`of ${formatSOL(vault.approvedAmountSol)} SOL`}
            color="green"
          />
          <StatBox
            label="Total Deposited"
            value={formatSOL(vault.totalDepositedSol) + ' SOL'}
            color="purple"
          />
          <StatBox
            label="Total Withdrawn"
            value={formatSOL(vault.totalWithdrawnSol) + ' SOL'}
            color="muted"
          />
          <StatBox
            label="Trades"
            value={vault.tradeCount.toString()}
            sub="executed"
            color="coral"
            icon={<TrendingUp size={12} />}
          />
        </div>

        {/* Utilization bar */}
        <div className="mb-5">
          <div className="flex justify-between text-xs text-vault-muted mb-1.5">
            <span>Vault utilization</span>
            <span className="mono">{utilizationPct.toFixed(1)}%</span>
          </div>
          <div className="h-1.5 rounded-full bg-vault-border overflow-hidden">
            <div
              className="h-full rounded-full bg-gradient-to-r from-sol-purple to-sol-green transition-all duration-700"
              style={{ width: `${Math.min(100, utilizationPct)}%` }}
            />
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline-green" onClick={() => setShowDeposit(true)}>
            <ArrowDownToLine size={14} /> Deposit
          </Button>
          <Button size="sm" variant="secondary" onClick={() => setShowWithdraw(true)}>
            <ArrowUpFromLine size={14} /> Withdraw
          </Button>
          <Button
            size="sm"
            variant={vault.status === 'paused' ? 'outline-green' : 'danger'}
            onClick={() => setShowPause(true)}
          >
            {vault.status === 'paused' ? <Play size={14} /> : <Pause size={14} />}
            {vault.status === 'paused' ? 'Resume' : 'Emergency Pause'}
          </Button>
        </div>
      </Card>

      <DepositModal isOpen={showDeposit} onClose={() => setShowDeposit(false)} />
      <WithdrawModal isOpen={showWithdraw} onClose={() => setShowWithdraw(false)} />
      <PauseModal isOpen={showPause} onClose={() => setShowPause(false)} />
    </>
  );
}

function StatBox({ label, value, sub, color, icon }: {
  label: string;
  value: string;
  sub?: string;
  color: 'green' | 'purple' | 'muted' | 'coral';
  icon?: React.ReactNode;
}) {
  const colors = {
    green: 'text-sol-green',
    purple: 'text-sol-purple',
    muted: 'text-vault-muted',
    coral: 'text-sol-coral',
  };
  return (
    <div className="p-3 rounded-lg bg-vault-bg/50 border border-vault-border/50">
      <p className="text-xs text-vault-muted mb-1 flex items-center gap-1">{icon}{label}</p>
      <p className={`text-base font-bold mono ${colors[color]}`}>{value}</p>
      {sub && <p className="text-xs text-vault-muted mt-0.5">{sub}</p>}
    </div>
  );
}
