'use client';

import React, { useState } from 'react';
import { ShieldOff } from 'lucide-react';
import { useRouter } from 'next/navigation';

import { Modal } from '@/components/common/Modal';
import { Button } from '@/components/common/Button';
import { useVault } from '@/contexts/VaultContext';
import { formatSOL } from '@/lib/mock';

interface RevokeModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function RevokeModal({ isOpen, onClose }: RevokeModalProps) {
  const { vault, revokeAccess, isLoading } = useVault();
  const [confirmed, setConfirmed] = useState(false);
  const router = useRouter();

  if (!vault) return null;

  const returnAmount = Math.max(0, vault.availableAmountSol);

  const handleRevoke = async () => {
    await revokeAccess();
    onClose();
    router.push('/');
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Revoke Delegate Access" icon="Lock">
      <div className="space-y-4">
        <div className="rounded-xl bg-red-500/5 border border-red-500/20 p-4 space-y-2 text-sm text-vault-muted">
          <p className="text-red-400 font-semibold flex items-center gap-2">
            <ShieldOff size={14} /> This will permanently:
          </p>
          <ul className="space-y-1 pl-5">
            <li>- Terminate the current trading session</li>
            <li>- Return all vault funds to your wallet</li>
            <li>- Deactivate the vault completely</li>
          </ul>
        </div>

        <div className="rounded-lg bg-vault-bg border border-vault-border p-4 space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-vault-muted">Vault balance</span>
            <span className="mono text-white">{formatSOL(vault.availableAmountSol)} SOL</span>
          </div>
          <div className="border-t border-vault-border pt-2 flex justify-between">
            <span className="text-vault-muted">Returned to wallet</span>
            <span className="mono text-sol-green font-bold">{formatSOL(returnAmount)} SOL</span>
          </div>
        </div>

        <label className="flex items-start gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={confirmed}
            onChange={(e) => setConfirmed(e.target.checked)}
            className="mt-0.5 accent-red-500"
          />
          <span className="text-xs text-vault-muted">
            I understand this action cannot be undone and will deactivate the vault.
          </span>
        </label>

        <div className="flex gap-3 pt-2">
          <Button variant="ghost" className="flex-1" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="danger"
            className="flex-1"
            onClick={handleRevoke}
            loading={isLoading}
            disabled={!confirmed}
          >
            <ShieldOff size={14} /> Revoke and Return
          </Button>
        </div>
      </div>
    </Modal>
  );
}
