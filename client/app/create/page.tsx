"use client";

import React, { useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import { PublicKey } from "@solana/web3.js";
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

const isValidSolanaAddress = (addr: string) => {
  try {
    new PublicKey(addr);
    return true;
  } catch {
    return false;
  }
};

export default function CreatePage() {
  const router = useRouter();
  const { createVault, isLoading, walletConnected, connectWallet } = useVault();

  const [step, setStep] = useState(0);
  const [currentTime] = useState(() => Date.now());

  const [approvedAmount, setApprovedAmount] = useState("10");
  const [delegate, setDelegate] = useState("");
  const [sessionDuration, setSessionDuration] = useState(60);
  const [initialDeposit, setInitialDeposit] = useState("");

  const balance = 5; // Mocked wallet balance, replace with actual balance from context or hook
  const walletBalance = balance ?? 0;

  const approvedNum = parseFloat(approvedAmount) || 0;
  const depositNum = parseFloat(initialDeposit) || 0;

  const expiryTime = useMemo(() => {
    return new Date(currentTime + sessionDuration * 60_000);
  }, [sessionDuration, currentTime]);

  const approvedError =
    approvedNum < 0.001
      ? "Minimum is 0.001 SOL"
      : approvedNum > 1000
      ? "Maximum is 1,000 SOL"
      : undefined;

  const delegateError =
    delegate && !isValidSolanaAddress(delegate)
      ? "Invalid Solana address"
      : undefined;

  const depositError =
    depositNum > walletBalance
      ? "Exceeds wallet balance"
      : depositNum > approvedNum
      ? "Exceeds approved limit"
      : depositNum <= 0 && initialDeposit
      ? "Deposit must be greater than 0"
      : undefined;

  const canNext = () => {
    if (step === 0) return !approvedError && approvedNum > 0;
    if (step === 1) return delegate && !delegateError;
    if (step === 2)
      return (
        depositNum > 0 &&
        depositNum <= walletBalance &&
        depositNum <= approvedNum
      );
    return false;
  };

  const handleNext = () => {
    if (step < 2 && canNext()) setStep((s) => s + 1);
  };

  const handleCreate = async () => {
    if (!canNext() || isLoading) return;

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
          <h1 className="text-2xl font-extrabold mb-1">Create ExecVault</h1>
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
                  className={`flex-1 h-0.5 mx-2 mb-4 ${
                    i < step ? "bg-sol-green" : "bg-vault-border"
                  }`}
                />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* Step Card */}
        <Card elevated glow="purple">
          {/* STEP 0 */}
          {step === 0 && (
            <div className="space-y-5">
              <Input
                label="Approved Amount"
                type="number"
                value={approvedAmount}
                onChange={(e) => setApprovedAmount(e.target.value)}
                hint="Min: 0.001 · Max: 1,000"
                suffixNode="SOL"
                error={approvedError}
              />

              <div className="space-y-2">
                <input
                  type="range"
                  min="0.001"
                  max="100"
                  step="0.1"
                  value={Math.min(approvedNum || 0, 100)}
                  onChange={(e) => setApprovedAmount(e.target.value)}
                  className="w-full accent-sol-purple"
                />
                <div className="flex justify-between text-xs text-vault-muted">
                  <span>0.001</span>
                  <span>50</span>
                  <span>100 SOL</span>
                </div>
              </div>
            </div>
          )}

          {/* STEP 1 */}
          {step === 1 && (
            <div className="space-y-5">
              <Input
                label="Delegate Wallet Address"
                type="text"
                value={delegate}
                onChange={(e) => setDelegate(e.target.value)}
                placeholder="Enter Solana wallet address..."
                hint="Only approve trusted bots"
                error={delegateError}
              />

              <div className="space-y-2">
                <label className="text-xs font-medium text-vault-muted uppercase tracking-wider">
                  Session Duration
                </label>

                {[30, 60, 240].map((val) => (
                  <button
                    key={val}
                    type="button"
                    onClick={() => setSessionDuration(val)}
                    className={`w-full text-left p-3 rounded-lg border transition ${
                      sessionDuration === val
                        ? "border-sol-purple bg-sol-purple/10"
                        : "border-vault-border"
                    }`}
                  >
                    {val === 60 ? "1 Hour (recommended)" : `${val} Minutes`}
                  </button>
                ))}
              </div>

              <div className="flex items-start gap-2 text-xs text-amber-400 bg-amber-400/5 border border-amber-400/20 rounded-lg p-3">
                <AlertTriangle size={14} className="mt-0.5" />
                Session expires at {expiryTime.toUTCString()}
              </div>
            </div>
          )}

          {/* STEP 2 */}
          {step === 2 && (
            <div className="space-y-5">
              <Input
                label="Deposit Amount"
                type="number"
                value={initialDeposit}
                onChange={(e) => setInitialDeposit(e.target.value)}
                hint={`Wallet balance: ${walletBalance} SOL`}
                suffixNode="SOL"
                error={depositError}
              />

              <button
                type="button"
                className="text-xs text-sol-purple hover:underline"
                onClick={() =>
                  setInitialDeposit(
                    Math.min(walletBalance, approvedNum).toString(),
                  )
                }
              >
                Use max: {Math.min(walletBalance, approvedNum)} SOL
              </button>

              <div className="rounded-xl bg-vault-bg border border-vault-border p-4 space-y-3 text-sm">
                <SumRow label="Approved Limit" value={`${approvedNum} SOL`} />
                <SumRow
                  label="Depositing"
                  value={depositNum ? `${depositNum} SOL` : "—"}
                  highlight={!!depositNum}
                />
                <SumRow label="Delegate" value={delegate || "—"} mono />
                <SumRow
                  label="Session Expiry"
                  value={expiryTime.toUTCString()}
                />
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
}: {
  label: string;
  value: string;
  highlight?: boolean;
  mono?: boolean;
}) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-vault-muted">{label}</span>
      <span
        className={`text-sm ${mono ? "font-mono" : ""} ${
          highlight ? "text-sol-green font-bold" : "text-white"
        }`}
      >
        {value}
      </span>
    </div>
  );
}
