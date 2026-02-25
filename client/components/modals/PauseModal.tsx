'use client';

import React from 'react';
import { AlertTriangle } from 'lucide-react';
import { Modal } from '@/components/common/Modal';
import { Button } from '@/components/common/Button';
import { useVault } from '@/contexts/VaultContext';

interface PauseModalProps { isOpen: boolean; onClose: () => void; }

export function PauseModal({ isOpen, onClose }: PauseModalProps) {
  const { vault, pauseVault, unpauseVault, isLoading } = useVault();
  const isPaused = vault?.status === 'paused';

  const handleAction = async () => {
    if (isPaused) await unpauseVault();
    else await pauseVault();
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={isPaused ? 'Resume Vault' : 'Emergency Pause'}
      icon={isPaused ? '▶️' : '⚠️'}
    >
      <div className="space-y-4">
        <div className="rounded-xl bg-amber-500/5 border border-amber-500/20 p-4">
          <div className="flex items-start gap-3">
            <AlertTriangle size={18} className="text-amber-400 flex-shrink-0 mt-0.5" />
            <div className="text-sm">
              <p className="font-semibold text-amber-400 mb-2">
                {isPaused ? 'Resume trading activity' : 'This will immediately:'}
              </p>
              {isPaused ? (
                <p className="text-vault-muted">The vault will resume normal operation and the delegate bot will be able to execute trades again.</p>
              ) : (
                <ul className="text-vault-muted space-y-1">
                  <li>• Stop all trading activity</li>
                  <li>• Block new deposits from delegate</li>
                  <li>• Prevent trade execution</li>
                </ul>
              )}
            </div>
          </div>
        </div>

        <p className="text-sm text-vault-muted">
          {isPaused ? 'Your vault is currently paused. Resume to allow trading.' : 'Your funds remain safe in the vault. You can unpause anytime.'}
        </p>

        <div className="flex gap-3 pt-2">
          <Button variant="ghost" className="flex-1" onClick={onClose}>Cancel</Button>
          <Button
            className="flex-1"
            variant={isPaused ? undefined : 'danger'}
            onClick={handleAction}
            loading={isLoading}
          >
            {isPaused ? 'Resume Vault' : 'Pause Vault'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
