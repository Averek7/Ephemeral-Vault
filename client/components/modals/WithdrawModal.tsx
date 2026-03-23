'use client';

import React, { useState } from 'react';
import { ArrowUpFromLine } from 'lucide-react';

import { Modal } from '@/components/common/Modal';
import { Input } from '@/components/common/Input';
import { Button } from '@/components/common/Button';
import { useVault } from '@/contexts/VaultContext';
import { formatSOL } from '@/lib/mock';

interface WithdrawModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function WithdrawModal({ isOpen, onClose }: WithdrawModalProps) {
  const { vault, withdraw, isLoading } = useVault();
  const [amount, setAmount] = useState('');

  if (!vault) return null;

  const amountNum = parseFloat(amount) || 0;
  const valid = amountNum > 0 && amountNum <= vault.availableAmountSol;

  const handleSubmit = async () => {
    if (!valid) return;
    await withdraw(amountNum);
    setAmount('');
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Withdraw from Vault" icon="Withdraw">
      <div className="space-y-4">
        <Input
          label="Amount"
          type="number"
          placeholder="0.000"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          suffixNode="SOL"
          hint={`Available: ${formatSOL(vault.availableAmountSol)} SOL`}
          error={amount && !valid ? 'Insufficient vault balance' : undefined}
        />
        <button
          className="text-xs text-sol-purple hover:underline"
          onClick={() => setAmount(formatSOL(vault.availableAmountSol))}
        >
          Use max: {formatSOL(vault.availableAmountSol)} SOL
        </button>

        {amountNum > 0 && (
          <div className="rounded-lg bg-vault-bg border border-vault-border p-4 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-vault-muted">Withdrawing</span>
              <span className="mono text-white">{formatSOL(amountNum)} SOL</span>
            </div>
            <div className="border-t border-vault-border pt-2 flex justify-between">
              <span className="text-vault-muted">Available after</span>
              <span className="mono text-sol-green font-bold">
                {formatSOL(Math.max(0, vault.availableAmountSol - amountNum))} SOL
              </span>
            </div>
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <Button variant="ghost" className="flex-1" onClick={onClose}>
            Cancel
          </Button>
          <Button
            className="flex-1"
            onClick={handleSubmit}
            loading={isLoading}
            disabled={!valid}
          >
            <ArrowUpFromLine size={14} /> Withdraw
          </Button>
        </div>
      </div>
    </Modal>
  );
}
