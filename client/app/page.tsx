"use client";

import React, { useEffect, useRef, useState } from "react";
import Link from "next/link";
import {
  Shield,
  Timer,
  Zap,
  Lock,
  TrendingUp,
  ChevronRight,
  ArrowRight,
  CheckCircle2,
  ArrowDownToLine,
  Bot,
  RotateCcw,
  ShieldOff,
  Github,
  BookOpen,
  FileCheck,
  Activity,
  Users,
  Cpu,
  GitBranch,
} from "lucide-react";
import { WalletButton } from "@/components/wallet/WalletButton";
import { useVault } from "@/contexts/VaultContext";

// ─── Live ticker feed ────────────────────────────────────────────────────────
const TICKER_EVENTS = [
  {
    type: "vault",
    msg: "Vault created · 5.0 SOL approved",
    addr: "9xPq...2mRt",
    time: "2s ago",
  },
  {
    type: "trade",
    msg: "Swap executed · 0.3 SOL",
    addr: "7hNm...3kRt",
    time: "8s ago",
  },
  {
    type: "revoke",
    msg: "Access revoked · 1.2 SOL returned",
    addr: "4cBz...9wLp",
    time: "14s ago",
  },
  {
    type: "trade",
    msg: "Swap executed · 0.8 SOL",
    addr: "2nXv...5rQd",
    time: "21s ago",
  },
  {
    type: "vault",
    msg: "Session renewed · 60 min",
    addr: "6mFj...1kSa",
    time: "35s ago",
  },
  {
    type: "trade",
    msg: "Buy executed · 0.15 SOL",
    addr: "8xKd...m9Pq",
    time: "47s ago",
  },
  {
    type: "trade",
    msg: "Swap executed · 1.1 SOL",
    addr: "3tYu...7hEw",
    time: "1m ago",
  },
  {
    type: "vault",
    msg: "Vault created · 20.0 SOL approved",
    addr: "5qAn...4cBx",
    time: "1m ago",
  },
];

function LiveTicker() {
  const [offset, setOffset] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setOffset((o) => o + 1);
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  const typeColor: Record<string, string> = {
    vault: "#9945FF",
    trade: "#14F195",
    revoke: "#FF6B9D",
  };

  const typeIcon: Record<string, React.ReactNode> = {
    vault: <Lock size={10} />,
    trade: <TrendingUp size={10} />,
    revoke: <ShieldOff size={10} />,
  };

  return (
    <div className="relative overflow-hidden h-8 flex items-center">
      <div
        className="absolute inset-y-0 left-0 w-12 z-10 pointer-events-none"
        style={{ background: "linear-gradient(90deg, #0A0A0F, transparent)" }}
      />
      <div
        className="absolute inset-y-0 right-0 w-12 z-10 pointer-events-none"
        style={{ background: "linear-gradient(-90deg, #0A0A0F, transparent)" }}
      />
      <div
        className="flex gap-8 transition-transform duration-700 ease-in-out"
        style={{
          transform: `translateX(-${offset * 280}px)`,
          width: "max-content",
        }}
      >
        {[...TICKER_EVENTS, ...TICKER_EVENTS].map((e, i) => (
          <div
            key={i}
            className="flex items-center gap-2 whitespace-nowrap text-xs"
          >
            <span
              className="flex items-center gap-1 px-2 py-0.5 rounded-full font-mono"
              style={{
                background: typeColor[e.type] + "15",
                color: typeColor[e.type],
                border: `1px solid ${typeColor[e.type]}30`,
              }}
            >
              {typeIcon[e.type]} {e.type}
            </span>
            <span className="text-white/70">{e.msg}</span>
            <span className="text-vault-muted font-mono">{e.addr}</span>
            <span className="text-vault-border font-mono">{e.time}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Animated vault diagram ──────────────────────────────────────────────────
function VaultDiagram() {
  const [pulse, setPulse] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setPulse((p) => (p + 1) % 3), 1200);
    return () => clearInterval(t);
  }, []);

  return (
    <div className="relative w-full max-w-sm mx-auto h-64 select-none">
      {/* Center vault */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-20">
        <div className="w-20 h-20 rounded-2xl bg-vault-surface border-2 border-sol-purple/50 flex flex-col items-center justify-center gap-1 shadow-lg shadow-sol-purple/20">
          <Shield size={22} className="text-sol-purple" />
          <span className="text-xs font-bold text-sol-purple mono">VAULT</span>
        </div>
        {/* Rotating ring */}
        <div
          className="absolute inset-0 -m-3 rounded-3xl border border-dashed border-sol-purple/20 animate-spin"
          style={{ animationDuration: "12s" }}
        />
      </div>

      {/* Wallet node */}
      <div className="absolute top-4 left-8">
        <NodeBox
          icon={<Users size={14} />}
          label="Owner"
          color="#9945FF"
          active={pulse === 0}
        />
      </div>
      {/* Bot node */}
      <div className="absolute top-4 right-8">
        <NodeBox
          icon={<Bot size={14} />}
          label="Bot"
          color="#14F195"
          active={pulse === 1}
        />
      </div>
      {/* Chain node */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2">
        <NodeBox
          icon={<Cpu size={14} />}
          label="Solana"
          color="#FF6B9D"
          active={pulse === 2}
        />
      </div>

      {/* Connecting lines — SVG */}
      <svg
        className="absolute inset-0 w-full h-full pointer-events-none"
        style={{ zIndex: 5 }}
      >
        {/* Owner → Vault */}
        <line
          x1="90"
          y1="44"
          x2="160"
          y2="120"
          stroke="rgba(153,69,255,0.25)"
          strokeWidth="1"
          strokeDasharray="4 4"
        />
        {/* Bot → Vault */}
        <line
          x1="260"
          y1="44"
          x2="180"
          y2="120"
          stroke="rgba(20,241,149,0.25)"
          strokeWidth="1"
          strokeDasharray="4 4"
        />
        {/* Vault → Solana */}
        <line
          x1="170"
          y1="148"
          x2="170"
          y2="208"
          stroke="rgba(255,107,157,0.25)"
          strokeWidth="1"
          strokeDasharray="4 4"
        />
        {/* Animated dot on owner→vault */}
        {pulse === 0 && (
          <circle r="3" fill="#9945FF">
            <animateMotion dur="1.2s" repeatCount="1" path="M90,44 L160,120" />
          </circle>
        )}
        {/* Animated dot on bot→vault */}
        {pulse === 1 && (
          <circle r="3" fill="#14F195">
            <animateMotion dur="1.2s" repeatCount="1" path="M260,44 L180,120" />
          </circle>
        )}
        {/* Animated dot on vault→solana */}
        {pulse === 2 && (
          <circle r="3" fill="#FF6B9D">
            <animateMotion
              dur="1.2s"
              repeatCount="1"
              path="M170,148 L170,208"
            />
          </circle>
        )}
      </svg>
    </div>
  );
}

function NodeBox({
  icon,
  label,
  color,
  active,
}: {
  icon: React.ReactNode;
  label: string;
  color: string;
  active: boolean;
}) {
  return (
    <div
      className="w-16 h-16 rounded-xl flex flex-col items-center justify-center gap-1 border transition-all duration-500"
      style={{
        background: active ? color + "18" : "rgba(26,26,36,0.8)",
        borderColor: active ? color + "80" : "#2D2D3D",
        boxShadow: active ? `0 0 16px ${color}30` : "none",
      }}
    >
      <span style={{ color }}>{icon}</span>
      <span
        className="text-xs font-mono"
        style={{ color: active ? color : "#A0A0B0" }}
      >
        {label}
      </span>
    </div>
  );
}

// ─── How it works step ───────────────────────────────────────────────────────
function HowStep({
  n,
  icon,
  title,
  desc,
  color,
}: {
  n: number;
  icon: React.ReactNode;
  title: string;
  desc: string;
  color: string;
}) {
  return (
    <div className="flex gap-4">
      <div className="flex flex-col items-center gap-2 flex-shrink-0">
        <div
          className="w-10 h-10 rounded-xl flex items-center justify-center border"
          style={{ background: color + "12", borderColor: color + "40", color }}
        >
          {icon}
        </div>
        {n < 4 && (
          <div
            className="w-px flex-1 min-h-6"
            style={{ background: `linear-gradient(${color}40, transparent)` }}
          />
        )}
      </div>
      <div className="pb-6">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-xs font-mono" style={{ color }}>
            {String(n).padStart(2, "0")}
          </span>
          <h4 className="text-sm font-bold text-white">{title}</h4>
        </div>
        <p className="text-sm text-vault-muted leading-relaxed">{desc}</p>
      </div>
    </div>
  );
}

// ─── Security feature pill ───────────────────────────────────────────────────
function SecurityPill({
  icon,
  label,
}: {
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-vault-bg border border-vault-border text-xs text-vault-muted hover:text-white hover:border-sol-purple/30 transition-colors">
      <span className="text-sol-green">{icon}</span>
      {label}
    </div>
  );
}

// ─── Main page ───────────────────────────────────────────────────────────────
export default function LandingPage() {
  const { walletConnected, connectWallet } = useVault();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    const particles: Array<{
      x: number;
      y: number;
      vx: number;
      vy: number;
      r: number;
      a: number;
    }> = [];
    for (let i = 0; i < 70; i++) {
      particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        vx: (Math.random() - 0.5) * 0.25,
        vy: (Math.random() - 0.5) * 0.25,
        r: Math.random() * 1.5 + 0.5,
        a: Math.random() * 0.6 + 0.1,
      });
    }

    let raf: number;
    const draw = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      particles.forEach((p) => {
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < 0) p.x = canvas.width;
        if (p.x > canvas.width) p.x = 0;
        if (p.y < 0) p.y = canvas.height;
        if (p.y > canvas.height) p.y = 0;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(153,69,255,${p.a * 0.35})`;
        ctx.fill();
      });
      particles.forEach((p, i) => {
        particles.slice(i + 1).forEach((q) => {
          const dx = p.x - q.x,
            dy = p.y - q.y,
            dist = Math.sqrt(dx * dx + dy * dy);
          if (dist < 130) {
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.strokeStyle = `rgba(153,69,255,${(1 - dist / 130) * 0.09})`;
            ctx.lineWidth = 1;
            ctx.stroke();
          }
        });
      });
      raf = requestAnimationFrame(draw);
    };
    draw();
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return (
    <div className="min-h-screen relative overflow-x-hidden grid-texture">
      <canvas
        ref={canvasRef}
        className="fixed inset-0 pointer-events-none"
        style={{ zIndex: 0 }}
      />

      {/* Glow orbs */}
      <div
        className="fixed top-0 left-1/2 -translate-x-1/2 w-[600px] h-64 rounded-full bg-sol-purple/8 blur-3xl pointer-events-none"
        style={{ zIndex: 1 }}
      />
      <div
        className="fixed top-1/3 right-0 w-72 h-72 rounded-full bg-sol-green/5 blur-3xl pointer-events-none"
        style={{ zIndex: 1 }}
      />
      <div
        className="fixed bottom-1/3 left-0 w-72 h-72 rounded-full bg-sol-purple/5 blur-3xl pointer-events-none"
        style={{ zIndex: 1 }}
      />

      {/* ── Navbar ── */}
      <header className="relative z-40 border-b border-vault-border bg-vault-bg/60 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center">
              <Shield size={14} className="text-sol-purple" />
            </div>
            <span className="text-sm font-bold tracking-tight">
              Exec<span className="text-sol-purple">Vault</span>
            </span>
          </div>
          <nav className="hidden md:flex items-center gap-1">
            <a
              href="#how"
              className="px-3 py-1.5 text-xs text-vault-muted hover:text-white transition-colors rounded-lg hover:bg-white/5"
            >
              How it works
            </a>
            <a
              href="#security"
              className="px-3 py-1.5 text-xs text-vault-muted hover:text-white transition-colors rounded-lg hover:bg-white/5"
            >
              Security
            </a>
            <Link
              href="/create"
              className="px-3 py-1.5 text-xs text-vault-muted hover:text-white transition-colors rounded-lg hover:bg-white/5"
            >
              Create Vault
            </Link>
          </nav>
          <WalletButton />
        </div>
      </header>

      {/* ── Live ticker banner ── */}
      <div className="relative z-30 border-b border-vault-border bg-vault-surface/40 backdrop-blur py-1.5 px-6">
        <div className="max-w-7xl mx-auto flex items-center gap-4">
          <span className="flex items-center gap-1.5 text-xs font-mono text-sol-green flex-shrink-0 border-r border-vault-border pr-4">
            <Activity size={10} className="animate-pulse" /> LIVE
          </span>
          <LiveTicker />
        </div>
      </div>

      {/* ── Hero ── */}
      <section className="relative z-10 max-w-6xl mx-auto px-6 pt-20 pb-16">
        <div
          className={`grid lg:grid-cols-2 gap-12 items-center transition-all duration-700 ${mounted ? "opacity-100 translate-y-0" : "opacity-0 translate-y-8"}`}
        >
          {/* Left: copy */}
          <div>
            {/* Live badge — green lamp */}
            <div
              className="inline-flex items-center gap-2.5 px-3.5 py-1.5 rounded-full mb-8 relative overflow-hidden"
              style={{
                background: "rgba(20,241,149,0.08)",
                border: "1px solid rgba(20,241,149,0.25)",
              }}
            >
              {/* Lamp glow layers */}
              <span className="relative flex-shrink-0">
                {/* Outer halo */}
                <span
                  className="absolute inset-0 -m-1.5 rounded-full bg-sol-green/20 animate-ping"
                  style={{ animationDuration: "2s" }}
                />
                {/* Mid ring */}
                <span className="absolute inset-0 -m-0.5 rounded-full bg-sol-green/30" />
                {/* Inner dot */}
                <span
                  className="relative block w-2 h-2 rounded-full bg-sol-green"
                  style={{ boxShadow: "0 0 6px #14F195, 0 0 12px #14F19580" }}
                />
              </span>
              <span
                className="text-xs font-mono font-semibold"
                style={{ color: "#14F195" }}
              >
                Live on Solana Devnet
              </span>
            </div>

            <h1 className="text-5xl md:text-6xl font-extrabold leading-[1.05] tracking-tight mb-6">
              Secure Temporary
              <br />
              <span
                className="gradient-text"
                style={{ textShadow: "0 0 60px rgba(153,69,255,0.4)" }}
              >
                Trading Access
              </span>
            </h1>

            <p className="text-base text-vault-muted leading-relaxed mb-8 max-w-lg">
              Pre-approve spending limits for automated bots with time-locked
              sessions. Full on-chain enforcement — no trust required. Revoke
              instantly anytime.
            </p>

            {/* CTA buttons */}
            <div className="flex flex-wrap gap-3 mb-10">
              {walletConnected ? (
                <Link
                  href="/dashboard"
                  className="inline-flex items-center gap-2 px-6 py-3 rounded-xl btn-shimmer text-white font-bold text-sm shadow-lg shadow-sol-purple/30 hover:brightness-110 transition-all"
                >
                  Open Dashboard <ArrowRight size={16} />
                </Link>
              ) : (
                <button
                  onClick={connectWallet}
                  className="inline-flex items-center gap-2 px-6 py-3 rounded-xl btn-shimmer text-white font-bold text-sm shadow-lg shadow-sol-purple/30 hover:brightness-110 transition-all"
                >
                  Connect &amp; Start <ArrowRight size={16} />
                </button>
              )}
              <Link
                href="/create"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-xl border border-vault-border text-vault-muted text-sm font-semibold hover:text-white hover:border-sol-purple/40 transition-all"
              >
                Create Vault <ChevronRight size={16} />
              </Link>
            </div>

            {/* Trust signals */}
            <div className="flex flex-wrap gap-2">
              {[
                { icon: <CheckCircle2 size={11} />, label: "Non-custodial" },
                { icon: <CheckCircle2 size={11} />, label: "Open source" },
                {
                  icon: <CheckCircle2 size={11} />,
                  label: "Audited contracts",
                },
                { icon: <CheckCircle2 size={11} />, label: "Instant revoke" },
              ].map((t) => (
                <span
                  key={t.label}
                  className="inline-flex items-center gap-1.5 text-xs text-vault-muted"
                >
                  <span className="text-sol-green">{t.icon}</span>
                  {t.label}
                </span>
              ))}
            </div>
          </div>

          {/* Right: diagram */}
          <div className="relative hidden lg:block">
            {/* Card frame */}
            <div
              className="rounded-2xl border border-vault-border bg-vault-surface/60 backdrop-blur p-6"
              style={{
                boxShadow:
                  "0 0 60px rgba(153,69,255,0.08), inset 0 1px 0 rgba(255,255,255,0.04)",
              }}
            >
              <div className="flex items-center gap-2 mb-4">
                <div className="w-2.5 h-2.5 rounded-full bg-red-500/60" />
                <div className="w-2.5 h-2.5 rounded-full bg-amber-500/60" />
                <div className="w-2.5 h-2.5 rounded-full bg-sol-green/60" />
                <span className="ml-2 text-xs text-vault-muted mono">
                  exec-vault · live
                </span>
              </div>
              <VaultDiagram />
              <div className="mt-4 pt-4 border-t border-vault-border grid grid-cols-3 gap-3 text-center">
                {[
                  { label: "Approved", val: "10.0 SOL", color: "#9945FF" },
                  { label: "Active", val: "45:12", color: "#14F195" },
                  { label: "Trades", val: "23", color: "#FF6B9D" },
                ].map((m) => (
                  <div key={m.label}>
                    <p className="text-xs text-vault-muted">{m.label}</p>
                    <p
                      className="text-sm font-bold mono mt-0.5"
                      style={{ color: m.color }}
                    >
                      {m.val}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── Stats ── */}
      <section className="relative z-10 border-y border-vault-border bg-vault-surface/20 backdrop-blur">
        <div className="max-w-6xl mx-auto px-6 py-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
            {[
              {
                label: "Vaults Created",
                value: "1,247",
                icon: <Lock size={14} />,
                color: "#9945FF",
              },
              {
                label: "SOL Protected",
                value: "84,392",
                icon: <Shield size={14} />,
                color: "#14F195",
              },
              {
                label: "Trades Executed",
                value: "203K",
                icon: <TrendingUp size={14} />,
                color: "#FF6B9D",
              },
              {
                label: "Avg Session",
                value: "47 min",
                icon: <Timer size={14} />,
                color: "#9945FF",
              },
            ].map((s) => (
              <div key={s.label} className="group">
                <div className="flex items-center justify-center gap-1.5 text-vault-muted text-xs mb-1.5">
                  <span style={{ color: s.color }}>{s.icon}</span>
                  {s.label}
                </div>
                <p className="text-2xl font-extrabold mono gradient-text">
                  {s.value}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Features grid ── */}
      <section className="relative z-10 max-w-6xl mx-auto px-6 py-20">
        <div className="text-center mb-12">
          <p className="text-xs font-mono text-sol-purple uppercase tracking-widest mb-3">
            Core Features
          </p>
          <h2 className="text-3xl font-extrabold">
            Everything you need to trade safely
          </h2>
        </div>
        <div className="grid md:grid-cols-3 gap-4">
          {[
            {
              icon: <Timer size={22} className="text-sol-green" />,
              title: "Time-Limited Access",
              desc: "Sessions expire automatically. No permanent approvals, no forgotten delegates ever lingering on-chain.",
              color: "green" as const,
              tag: "Auto-expire",
            },
            {
              icon: <Shield size={22} className="text-sol-purple" />,
              title: "Spending Caps",
              desc: "Pre-set the exact SOL limit. Bots are cryptographically constrained — they cannot exceed your authorized amount.",
              color: "purple" as const,
              tag: "On-chain enforcement",
            },
            {
              icon: <Zap size={22} className="text-sol-coral" />,
              title: "Instant Revoke",
              desc: "One transaction terminates everything. Funds return to your wallet immediately, no waiting period.",
              color: "coral" as const,
              tag: "Single tx",
            },
            {
              icon: <ArrowDownToLine size={22} className="text-sol-green" />,
              title: "Flexible Deposits",
              desc: "Top up your vault anytime up to the approved limit. Withdraw your idle balance whenever you want.",
              color: "green" as const,
              tag: "Non-custodial",
            },
            {
              icon: <Bot size={22} className="text-sol-purple" />,
              title: "Bot-Ready",
              desc: "Designed for trading bots, arbitrage strategies, and DeFi automation. Any Solana program can be a delegate.",
              color: "purple" as const,
              tag: "Programmable",
            },
            {
              icon: <Activity size={22} className="text-sol-coral" />,
              title: "Real-Time Monitoring",
              desc: "Watch every trade as it happens. Live balance updates, session countdowns, and instant alerts.",
              color: "coral" as const,
              tag: "WebSocket feed",
            },
          ].map((f, i) => {
            const bg =
              f.color === "green"
                ? "rgba(20,241,149,0.07)"
                : f.color === "purple"
                  ? "rgba(153,69,255,0.07)"
                  : "rgba(255,107,157,0.07)";
            const bc =
              f.color === "green"
                ? "rgba(20,241,149,0.18)"
                : f.color === "purple"
                  ? "rgba(153,69,255,0.18)"
                  : "rgba(255,107,157,0.18)";
            const tc =
              f.color === "green"
                ? "#14F195"
                : f.color === "purple"
                  ? "#9945FF"
                  : "#FF6B9D";
            return (
              <div
                key={f.title}
                className="group p-5 rounded-2xl border bg-vault-surface/50 backdrop-blur card-hover transition-all"
                style={{ borderColor: bc }}
              >
                <div className="flex items-start justify-between mb-4">
                  <div
                    className="w-11 h-11 rounded-xl flex items-center justify-center"
                    style={{ background: bg }}
                  >
                    {f.icon}
                  </div>
                  <span
                    className="text-xs font-mono px-2 py-0.5 rounded-full"
                    style={{
                      background: bg,
                      color: tc,
                      border: `1px solid ${tc}30`,
                    }}
                  >
                    {f.tag}
                  </span>
                </div>
                <h3 className="text-sm font-bold text-white mb-1.5">
                  {f.title}
                </h3>
                <p className="text-xs text-vault-muted leading-relaxed">
                  {f.desc}
                </p>
              </div>
            );
          })}
        </div>
      </section>

      {/* ── How it works ── */}
      <section
        id="how"
        className="relative z-10 border-t border-vault-border bg-vault-surface/10"
      >
        <div className="max-w-6xl mx-auto px-6 py-20">
          <div className="text-center mb-12">
            <p className="text-xs font-mono text-sol-green uppercase tracking-widest mb-3">
              How it works
            </p>
            <h2 className="text-3xl font-extrabold">
              Four steps to safe automation
            </h2>
          </div>
          <div className="grid md:grid-cols-2 gap-8 max-w-3xl mx-auto">
            <HowStep
              n={1}
              icon={<Zap size={16} />}
              color="#9945FF"
              title="Connect Wallet"
              desc="Connect your Phantom, Solflare, or any Solana wallet. No email, no KYC."
            />
            <HowStep
              n={2}
              icon={<Lock size={16} />}
              color="#14F195"
              title="Create Vault"
              desc="Set your spending cap, paste the bot's wallet address, and choose a session duration."
            />
            <HowStep
              n={3}
              icon={<Bot size={16} />}
              color="#FF6B9D"
              title="Delegate & Fund"
              desc="Deposit SOL into the vault. The bot can now trade up to your approved limit within the time window."
            />
            <HowStep
              n={4}
              icon={<RotateCcw size={16} />}
              color="#9945FF"
              title="Monitor & Revoke"
              desc="Watch trades live. Extend the session, top up, or revoke instantly to reclaim all funds."
            />
          </div>
        </div>
      </section>

      {/* ── Security section ── */}
      <section
        id="security"
        className="relative z-10 border-t border-vault-border"
      >
        <div className="max-w-6xl mx-auto px-6 py-20">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <p className="text-xs font-mono text-sol-coral uppercase tracking-widest mb-3">
                Security first
              </p>
              <h2 className="text-3xl font-extrabold mb-5">
                Built for trustless environments
              </h2>
              <p className="text-vault-muted text-sm leading-relaxed mb-8">
                Every vault is a Solana program account. Your funds never leave
                the blockchain — only your approved delegate can transact, and
                only up to your set limit, only while the session is active. The
                program is the enforcer.
              </p>
              <div className="flex flex-wrap gap-2">
                {[
                  {
                    icon: <FileCheck size={13} />,
                    label: "Audited by Ottersec",
                  },
                  {
                    icon: <GitBranch size={13} />,
                    label: "Open source on GitHub",
                  },
                  { icon: <Shield size={13} />, label: "Non-custodial" },
                  { icon: <Lock size={13} />, label: "On-chain enforcement" },
                  {
                    icon: <CheckCircle2 size={13} />,
                    label: "Zero trusted parties",
                  },
                  { icon: <Zap size={13} />, label: "Instant revoke" },
                ].map((p) => (
                  <SecurityPill key={p.label} icon={p.icon} label={p.label} />
                ))}
              </div>
            </div>
            {/* Code snippet */}
            <div
              className="rounded-2xl border border-vault-border bg-vault-bg overflow-hidden"
              style={{ boxShadow: "inset 0 1px 0 rgba(255,255,255,0.04)" }}
            >
              <div className="flex items-center gap-2 px-4 py-3 border-b border-vault-border bg-vault-surface/60">
                <div className="w-2.5 h-2.5 rounded-full bg-red-500/60" />
                <div className="w-2.5 h-2.5 rounded-full bg-amber-500/60" />
                <div className="w-2.5 h-2.5 rounded-full bg-sol-green/60" />
                <span className="ml-2 text-xs text-vault-muted mono">
                  vault.rs · on-chain constraints
                </span>
              </div>
              <pre
                className="p-5 text-xs leading-relaxed overflow-x-auto"
                style={{ fontFamily: "Space Mono, monospace" }}
              >
                {`// Enforce spending cap before every trade
require!(
  vault.current_balance + amount 
    <= vault.approved_amount,
  VaultError::ExceedsApprovedLimit
);

// Enforce session expiry
require!(
  Clock::get()?.unix_timestamp 
    < vault.session_expiry,
  VaultError::SessionExpired
);

// Enforce delegate authority
require!(
  ctx.accounts.delegate.key() 
    == vault.delegate,
  VaultError::UnauthorizedDelegate
);`}
              </pre>
            </div>
          </div>
        </div>
      </section>

      {/* ── CTA banner ── */}
      <section className="relative z-10 border-t border-vault-border overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-r from-sol-purple/8 via-transparent to-sol-green/8 pointer-events-none" />
        <div className="max-w-6xl mx-auto px-6 py-16 text-center relative">
          <h2 className="text-3xl md:text-4xl font-extrabold mb-4">
            Ready to automate <span className="gradient-text">safely?</span>
          </h2>
          <p className="text-vault-muted mb-8 max-w-md mx-auto text-sm">
            Create your first Execvault in under two minutes. No
            registration required.
          </p>
          <div className="flex flex-wrap justify-center gap-3">
            <button
              onClick={connectWallet}
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl btn-shimmer text-white font-bold text-sm shadow-lg shadow-sol-purple/30 hover:brightness-110 transition-all"
            >
              Get Started <ArrowRight size={16} />
            </button>
            <a
              href="#"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl border border-vault-border text-vault-muted text-sm font-semibold hover:text-white hover:border-sol-purple/40 transition-all"
            >
              <BookOpen size={15} /> Read Docs
            </a>
          </div>
        </div>
      </section>

      {/* ── Footer ── */}
      <footer className="relative z-10 border-t border-vault-border bg-vault-surface/20">
        <div className="max-w-6xl mx-auto px-6 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="flex items-center gap-2.5">
              <div className="w-6 h-6 rounded-lg bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center">
                <Shield size={12} className="text-sol-purple" />
              </div>
              <span className="text-sm font-bold">
                Exec<span className="text-sol-purple">Vault</span>
              </span>
              <span className="text-xs text-vault-muted ml-1">
                — Built on Solana
              </span>
            </div>
            <div className="flex items-center gap-5 text-xs text-vault-muted">
              <a
                href="#"
                className="hover:text-white transition-colors flex items-center gap-1.5"
              >
                <BookOpen size={12} /> Docs
              </a>
              <a
                href="#"
                className="hover:text-white transition-colors flex items-center gap-1.5"
              >
                <Github size={12} /> GitHub
              </a>
              <a
                href="#"
                className="hover:text-white transition-colors flex items-center gap-1.5"
              >
                <FileCheck size={12} /> Audit
              </a>
            </div>
            <div className="flex items-center gap-1.5 text-xs text-vault-muted">
              <span className="w-1.5 h-1.5 rounded-full bg-sol-green inline-block pulse-dot" />
              All systems operational
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}