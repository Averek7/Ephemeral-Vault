"use client";

import React, { useState } from "react";
import { useRouter } from "next/navigation";
import {
  Shield,
  ChevronRight,
  ChevronLeft,
  Check,
  AlertTriangle,
} from "lucide-react";
import { Navbar } from "@/components/common/Navbar";
import { Card } from "@/components/common/Card";
import { Input } from "@/components/common/Input";
import { Button } from "@/components/common/Button";
import { useVault } from "@/contexts/VaultContext";

const STEPS = ["Spending Limit", "Delegate Bot", "Initial Deposit"];

export default function CreatePage() {
  const router = useRouter();
  const { createVault, isLoading, walletConnected, connectWallet } = useVault();
  const [step, setStep] = useState(0);

  const [approvedAmount, setApprovedAmount] = useState("10");
  const [delegate, setDelegate] = useState("");
  const [sessionDuration, setSessionDuration] = useState(60);
  const [customDuration, setCustomDuration] = useState("");
  const [initialDeposit, setInitialDeposit] = useState("");
  const walletBalance = 5.2;

  const approvedNum = parseFloat(approvedAmount) || 0;
  const depositNum = parseFloat(initialDeposit) || 0;

  const canNext = () => {
    if (step === 0) return approvedNum >= 0.001 && approvedNum <= 1000;
    if (step === 1) return delegate.length >= 10;
    if (step === 2) return depositNum > 0 && depositNum <= walletBalance;
    return false;
  };

  const handleNext = () => {
    if (step < 2) setStep((s) => s + 1);
  };

  const handleCreate = async () => {
    await createVault({
      approvedAmount: approvedNum,
      delegate,
      sessionDuration,
      initialDeposit: depositNum,
    });
    router.push("/dashboard");
  };

  if (!walletConnected) {
    return (
      <div className="min-h-screen grid-texture">
        <Navbar />
        <div className="max-w-md mx-auto px-6 py-24 text-center space-y-4">
          <h1 className="text-2xl font-bold">Connect wallet first</h1>
          <p className="text-vault-muted">
            You need to connect your wallet to create a vault.
          </p>
          <Button onClick={connectWallet}>Connect Wallet</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen grid-texture">
      <Navbar />
      <main className="max-w-lg mx-auto px-4 sm:px-6 py-12">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-sol-purple/20 border border-sol-purple/30 flex items-center justify-center mx-auto mb-4">
            <Shield size={22} className="text-sol-purple" />
          </div>
          <h1 className="text-2xl font-extrabold mb-1">
            Create ExecVault
          </h1>
          <p className="text-sm text-vault-muted">
            Secure temporary trading access on Solana
          </p>
        </div>

        {/* Progress */}
        <div className="flex items-center mb-8">
          {STEPS.map((s, i) => (
            <React.Fragment key={s}>
              <div className="flex flex-col items-center">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold transition-all ${
                    i < step
                      ? "bg-sol-green text-black"
                      : i === step
                      ? "bg-sol-purple text-white"
                      : "bg-vault-surface border border-vault-border text-vault-muted"
                  }`}
                >
                  {i < step ? <Check size={14} /> : i + 1}
                </div>
                <span
                  className={`text-xs mt-1 ${
                    i === step ? "text-white" : "text-vault-muted"
                  }`}
                >
                  {s}
                </span>
              </div>
              {i < STEPS.length - 1 && (
                <div
                  className={`flex-1 h-0.5 mx-2 mb-4 transition-all ${
                    i < step ? "bg-sol-green" : "bg-vault-border"
                  }`}
                />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* Step content */}
        <Card elevated glow="purple">
          {step === 0 && (
            <div className="space-y-5">
              <div>
                <h2 className="text-lg font-bold mb-1">Set Spending Limit</h2>
                <p className="text-sm text-vault-muted">
                  Maximum amount you authorize for automated trading.
                </p>
              </div>
              <Input
                label="Approved Amount"
                type="number"
                value={approvedAmount}
                onChange={(e) => setApprovedAmount(e.target.value)}
                suffix="SOL"
                hint="Min: 0.001 SOL · Max: 1,000 SOL"
              />
              {/* Slider */}
              <div className="space-y-2">
                <input
                  type="range"
                  min="0.001"
                  max="100"
                  step="0.1"
                  value={Math.min(parseFloat(approvedAmount) || 0, 100)}
                  onChange={(e) => setApprovedAmount(e.target.value)}
                  className="w-full accent-sol-purple"
                />
                <div className="flex justify-between text-xs text-vault-muted">
                  <span>0.001</span>
                  <span>50</span>
                  <span>100 SOL</span>
                </div>
              </div>
              {approvedNum > 0 && (
                <div className="rounded-lg bg-sol-purple/5 border border-sol-purple/20 p-3 text-xs text-sol-purple">
                  Authorizing {approvedNum} SOL ≈ $
                  {(approvedNum * 170).toFixed(2)} USD
                </div>
              )}
            </div>
          )}

          {step === 1 && (
            <div className="space-y-5">
              <div>
                <h2 className="text-lg font-bold mb-1">Delegate Trading Bot</h2>
                <p className="text-sm text-vault-muted">
                  The wallet address of the bot you're authorizing.
                </p>
              </div>
              <Input
                label="Delegate Wallet Address"
                type="text"
                value={delegate}
                onChange={(e) => setDelegate(e.target.value)}
                placeholder="Enter Solana wallet address..."
                hint="Paste the bot's public key"
              />
              <div className="space-y-2">
                <label className="text-xs font-medium text-vault-muted uppercase tracking-wider">
                  Session Duration
                </label>
                <div className="space-y-2">
                  {[
                    { label: "30 Minutes", value: 30 },
                    { label: "1 Hour (recommended)", value: 60 },
                    { label: "4 Hours", value: 240 },
                  ].map((opt) => (
                    <label
                      key={opt.value}
                      className="flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all"
                      style={{
                        borderColor:
                          sessionDuration === opt.value
                            ? "rgba(153,69,255,0.4)"
                            : "#2D2D3D",
                        background:
                          sessionDuration === opt.value
                            ? "rgba(153,69,255,0.05)"
                            : "transparent",
                      }}
                    >
                      <input
                        type="radio"
                        name="duration"
                        checked={sessionDuration === opt.value}
                        onChange={() => setSessionDuration(opt.value)}
                        className="accent-sol-purple"
                      />
                      <span className="text-sm">{opt.label}</span>
                      {opt.value === 60 && (
                        <span className="ml-auto text-xs text-sol-green badge-active px-2 py-0.5 rounded-full">
                          Recommended
                        </span>
                      )}
                    </label>
                  ))}
                </div>
              </div>
              <div className="flex items-start gap-2 text-xs text-amber-400 bg-amber-400/5 border border-amber-400/20 rounded-lg p-3">
                <AlertTriangle size={14} className="flex-shrink-0 mt-0.5" />
                Only approve wallets from trusted bots and services!
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-5">
              <div>
                <h2 className="text-lg font-bold mb-1">Initial Deposit</h2>
                <p className="text-sm text-vault-muted">
                  Fund your vault to get started.
                </p>
              </div>
              <Input
                label="Deposit Amount"
                type="number"
                value={initialDeposit}
                onChange={(e) => setInitialDeposit(e.target.value)}
                suffix="SOL"
                hint={`Wallet balance: ${walletBalance} SOL`}
                error={
                  depositNum > walletBalance
                    ? "Exceeds wallet balance"
                    : depositNum > approvedNum
                    ? "Exceeds approved limit"
                    : undefined
                }
              />
              <button
                className="text-xs text-sol-purple hover:underline"
                onClick={() =>
                  setInitialDeposit(
                    Math.min(walletBalance, approvedNum).toFixed(3),
                  )
                }
              >
                Use max: {Math.min(walletBalance, approvedNum).toFixed(3)} SOL
              </button>

              {/* Summary */}
              <div className="rounded-xl bg-vault-bg border border-vault-border p-4 space-y-3 text-sm">
                <h4 className="text-xs font-semibold text-vault-muted uppercase tracking-wide">
                  Summary
                </h4>
                <SumRow label="Approved Limit" value={`${approvedNum} SOL`} />
                <SumRow
                  label="Depositing"
                  value={depositNum ? `${depositNum} SOL` : "—"}
                  highlight={!!depositNum}
                />
                <SumRow label="Delegate" value={delegate || "—"} mono />
                <SumRow label="Session" value={sessionDuration + " min"} />
                <SumRow label="Network Fee" value="~0.001 SOL" muted />
              </div>
            </div>
          )}

          {/* Navigation */}
          <div className="flex gap-3 mt-6">
            {step > 0 ? (
              <Button
                variant="ghost"
                className="flex-1"
                onClick={() => setStep((s) => s - 1)}
              >
                <ChevronLeft size={16} /> Back
              </Button>
            ) : (
              <Button
                variant="ghost"
                className="flex-1"
                onClick={() => router.push("/")}
              >
                Cancel
              </Button>
            )}
            {step < 2 ? (
              <Button
                className="flex-1"
                onClick={handleNext}
                disabled={!canNext()}
              >
                Next <ChevronRight size={16} />
              </Button>
            ) : (
              <Button
                className="flex-1"
                onClick={handleCreate}
                loading={isLoading}
                disabled={!canNext()}
              >
                <Shield size={16} /> Create Vault
              </Button>
            )}
          </div>
        </Card>
      </main>
    </div>
  );
}

function SumRow({
  label,
  value,
  highlight,
  mono,
  muted,
}: {
  label: string;
  value: string;
  highlight?: boolean;
  mono?: boolean;
  muted?: boolean;
}) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-vault-muted">{label}</span>
      <span
        className={`text-sm ${mono ? "font-mono" : ""} ${
          highlight
            ? "text-sol-green font-bold"
            : muted
            ? "text-vault-muted"
            : "text-white"
        }`}
      >
        {value}
      </span>
    </div>
  );
}
