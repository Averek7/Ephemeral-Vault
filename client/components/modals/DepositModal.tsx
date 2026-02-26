"use client";

import React, { useState } from "react";
import { ArrowDownToLine, CheckCircle2 } from "lucide-react";
import { Modal } from "@/components/common/Modal";
import { Input } from "@/components/common/Input";
import { Button } from "@/components/common/Button";
import { useVault } from "@/contexts/VaultContext";
import { formatSOL } from "@/lib/mock";

interface DepositModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function DepositModal({ isOpen, onClose }: DepositModalProps) {
  const { vault, deposit, isLoading } = useVault();
  const [amount, setAmount] = useState("");
  const walletBalance = 3.0;

  if (!vault) return null;

  const amountNum = parseFloat(amount) || 0;
  const afterDeposit = vault.currentBalance + amountNum;
  const remainingApproved = vault.approvedAmount - vault.currentBalance;
  const withinLimit = amountNum <= remainingApproved;
  const withinBalance = amountNum <= walletBalance;
  const valid = amountNum > 0 && withinLimit && withinBalance;

  const handleSubmit = async () => {
    if (!valid) return;
    await deposit(amountNum);
    setAmount("");
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Deposit to Vault" icon="📥">
      <div className="space-y-4">
        <Input
          label="Amount"
          type="number"
          placeholder="0.000"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          suffixNode="SOL"
          hint={`Available wallet balance: ${walletBalance} SOL`}
          error={
            amount && !withinBalance
              ? "Insufficient wallet balance"
              : amount && !withinLimit
              ? "Exceeds approved limit"
              : undefined
          }
        />

        {amountNum > 0 && (
          <div className="rounded-lg bg-vault-bg border border-vault-border p-4 space-y-2 text-sm">
            <InfoRow
              label="Current Vault"
              value={`${formatSOL(vault.currentBalance)} SOL`}
            />
            <InfoRow
              label="Depositing"
              value={`+ ${formatSOL(amountNum)} SOL`}
              color="green"
            />
            <div className="border-t border-vault-border pt-2">
              <InfoRow
                label="After Deposit"
                value={`${formatSOL(afterDeposit)} SOL`}
                bold
              />
            </div>
            <InfoRow
              label="Approved Limit"
              value={`${formatSOL(vault.approvedAmount)} SOL`}
            />
            <InfoRow
              label="Remaining"
              value={`${formatSOL(
                Math.max(0, remainingApproved - amountNum),
              )} SOL`}
            />
          </div>
        )}

        {valid && (
          <div className="flex items-center gap-2 text-xs text-sol-green">
            <CheckCircle2 size={12} /> Within approved limit
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
            <ArrowDownToLine size={14} /> Deposit
          </Button>
        </div>
      </div>
    </Modal>
  );
}

function InfoRow({
  label,
  value,
  color,
  bold,
}: {
  label: string;
  value: string;
  color?: "green";
  bold?: boolean;
}) {
  return (
    <div className="flex justify-between">
      <span className="text-vault-muted">{label}</span>
      <span
        className={`mono text-sm ${
          color === "green" ? "text-sol-green" : "text-white"
        } ${bold ? "font-bold" : ""}`}
      >
        {value}
      </span>
    </div>
  );
}
