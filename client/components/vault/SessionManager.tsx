"use client";

import React, { useEffect, useState } from "react";
import { Activity, RefreshCw, ShieldOff, Timer } from "lucide-react";

import { Card, CardHeader } from "@/components/common/Card";
import { Button } from "@/components/common/Button";
import { RevokeModal } from "@/components/modals/RevokeModal";
import { useVault } from "@/contexts/VaultContext";
import {
  formatCountdown,
  getSecondsRemaining,
  getTimerColor,
  getTimerPercent,
} from "@/lib/mock";

export function SessionManager() {
  const { vault, renewSession, isLoading } = useVault();
  const [seconds, setSeconds] = useState(0);
  const [showRevoke, setShowRevoke] = useState(false);
  const sessionExpiryMs = vault?.sessionExpiry ? vault.sessionExpiry * 1000 : null;

  useEffect(() => {
    if (!sessionExpiryMs) return;
    const tick = () => setSeconds(getSecondsRemaining(sessionExpiryMs));
    tick();
    const interval = setInterval(tick, 1000);
    return () => clearInterval(interval);
  }, [sessionExpiryMs]);

  if (!vault) return null;

  const color = getTimerColor(seconds);
  const pct = getTimerPercent(60, Math.max(0, seconds));
  const isExpiringSoon = vault.sessionStatus === "expiring_soon";
  const lastActivityMs = (vault.lastActivity || vault.createdAt) * 1000;

  return (
    <>
      <Card elevated>
        <CardHeader
          title="Session"
          icon={<Timer size={16} />}
          action={
            <span className="text-xs text-vault-muted">
              Last activity:{" "}
              <span className="text-white/70">
                {new Date(lastActivityMs).toLocaleString()}
              </span>
            </span>
          }
        />

        <div className="grid grid-cols-1 gap-4 mb-5 md:grid-cols-2">
          <div className="p-4 rounded-xl bg-vault-bg/50 border border-vault-border/50 flex flex-col items-center gap-2">
            <div className="relative w-24 h-24">
              <svg className="w-full h-full -rotate-90" viewBox="0 0 100 100">
                <circle
                  cx="50"
                  cy="50"
                  r="40"
                  fill="none"
                  stroke="#2D2D3D"
                  strokeWidth="6"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="40"
                  fill="none"
                  stroke={color}
                  strokeWidth="6"
                  strokeLinecap="round"
                  strokeDasharray={`${2 * Math.PI * 40}`}
                  strokeDashoffset={`${2 * Math.PI * 40 * (1 - pct / 100)}`}
                  style={{
                    transition: "stroke-dashoffset 1s linear, stroke 0.5s",
                  }}
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span
                  className={`text-xl font-bold mono ${
                    isExpiringSoon ? "animate-pulse" : ""
                  }`}
                  style={{ color }}
                >
                  {formatCountdown(seconds)}
                </span>
                <span className="text-xs text-vault-muted">remaining</span>
              </div>
            </div>
            {isExpiringSoon && (
              <p className="text-xs text-amber-400 animate-pulse">
                Session expiring soon!
              </p>
            )}
          </div>

          <div className="p-4 rounded-xl bg-vault-bg/50 border border-vault-border/50 space-y-3">
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Delegate Bot
              </p>
              <p className="text-sm font-bold mono text-white">
                {vault.delegate ?? "--"}
              </p>
            </div>
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Status
              </p>
              {vault.status === "active" ? (
                <span className="inline-flex items-center gap-1.5 text-xs badge-active px-2 py-1 rounded-full capitalize">
                  <Activity size={10} className="pulse-dot" />
                  {vault.sessionStatus.replaceAll("_", " ")}
                </span>
              ) : (
                <span className="inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-full border border-vault-border text-vault-muted capitalize">
                  {vault.status.replaceAll("_", " ")}
                </span>
              )}
            </div>
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Expires
              </p>
              <p className="text-xs text-white/70 mono">
                {sessionExpiryMs
                  ? new Date(sessionExpiryMs).toLocaleTimeString()
                  : "No session"}
              </p>
            </div>
          </div>
        </div>

        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline-green"
            onClick={renewSession}
            loading={isLoading}
            disabled={!vault.sessionExpiry}
          >
            <RefreshCw size={14} /> Renew Session
          </Button>
          <Button
            size="sm"
            variant="danger"
            onClick={() => setShowRevoke(true)}
          >
            <ShieldOff size={14} /> Revoke Access
          </Button>
        </div>
      </Card>

      <RevokeModal isOpen={showRevoke} onClose={() => setShowRevoke(false)} />
    </>
  );
}
