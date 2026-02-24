"use client";

import React, { useState, useEffect, useMemo } from "react";
import { Timer, RefreshCw, ShieldOff, Activity } from "lucide-react";
import { Card, CardHeader } from "@/components/common/Card";
import { Button } from "@/components/common/Button";
import { useVault } from "@/contexts/VaultContext";
import {
  formatCountdown,
  getSecondsRemaining,
  getTimerColor,
  getTimerPercent,
  formatTimestamp,
} from "@/lib/mock";
import { RevokeModal } from "@/components/modals/RevokeModal";

export function SessionManager() {
  const { vault, renewSession, isLoading } = useVault();
  const [seconds, setSeconds] = useState(0);
  const [showRevoke, setShowRevoke] = useState(false);

  useEffect(() => {
    if (!vault) return;
    const tick = () => setSeconds(getSecondsRemaining(vault.sessionExpiry));
    tick();
    const interval = setInterval(tick, 1000);
    return () => clearInterval(interval);
  }, [vault?.sessionExpiry]);

  if (!vault) return null;

  const color = getTimerColor(seconds);
  const pct = getTimerPercent(60, seconds);
  const isExpiringSoon = seconds < 5 * 60;
  const lastActivityTime = useMemo(() => {
    return Date.now() - 2 * 60 * 1000;
  }, []);

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
                {formatTimestamp(lastActivityTime)}
              </span>
            </span>
          }
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-5">
          {/* Timer */}
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

          {/* Delegate info */}
          <div className="p-4 rounded-xl bg-vault-bg/50 border border-vault-border/50 space-y-3">
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Delegate Bot
              </p>
              <p className="text-sm font-bold mono text-white">
                {vault.delegate}
              </p>
            </div>
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Status
              </p>
              <span className="inline-flex items-center gap-1.5 text-xs badge-active px-2 py-1 rounded-full">
                <Activity size={10} className="pulse-dot" />
                Trading Active
              </span>
            </div>
            <div>
              <p className="text-xs text-vault-muted mb-1 uppercase tracking-wide">
                Expires
              </p>
              <p className="text-xs text-white/70 mono">
                {new Date(vault.sessionExpiry).toLocaleTimeString()}
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
