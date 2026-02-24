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
} from "lucide-react";
import { WalletButton } from "@/components/wallet/WalletButton";
import { useVault } from "@/contexts/VaultContext";
import { useRouter } from "next/navigation";


export default function Home() {

  const { walletConnected } = useVault();
  const router = useRouter();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Particle animation
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
    for (let i = 0; i < 60; i++) {
      particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        r: Math.random() * 2 + 0.5,
        a: Math.random(),
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
        ctx.fillStyle = `rgba(153, 69, 255, ${p.a * 0.4})`;
        ctx.fill();
      });
      // Draw connections
      particles.forEach((p, i) => {
        particles.slice(i + 1).forEach((q) => {
          const dx = p.x - q.x,
            dy = p.y - q.y;
          const dist = Math.sqrt(dx * dx + dy * dy);
          if (dist < 120) {
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.strokeStyle = `rgba(153, 69, 255, ${(1 - dist / 120) * 0.12})`;
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
    <div className="min-h-screen relative overflow-hidden grid-texture">
      {/* Canvas background */}
      <canvas
        ref={canvasRef}
        className="fixed inset-0 pointer-events-none"
        style={{ zIndex: 0 }}
      />

      {/* Glow orbs */}
      <div className="fixed top-1/4 left-1/4 w-96 h-96 rounded-full bg-sol-purple/5 blur-3xl pointer-events-none" />
      <div className="fixed bottom-1/4 right-1/4 w-80 h-80 rounded-full bg-sol-green/5 blur-3xl pointer-events-none" />

      {/* Navbar */}
      <header className="relative z-10 border-b border-vault-border bg-vault-bg/50 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center">
              <Shield size={14} className="text-sol-purple" />
            </div>
            <span className="text-sm font-bold tracking-tight">
              Exec<span className="text-sol-purple">Vault</span>
            </span>
          </div>
          <WalletButton />
        </div>
      </header>

      {/* Hero */}
      <section className="relative z-10 max-w-5xl mx-auto px-6 pt-24 pb-20 text-center">
        <div
          className={`transition-all duration-700 ${mounted ? "opacity-100 translate-y-0" : "opacity-0 translate-y-8"}`}
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-sol-purple/10 border border-sol-purple/30 text-sol-purple text-xs font-mono mb-8">
            <span className="w-1.5 h-1.5 rounded-full bg-sol-green inline-block pulse-dot" />
            Live on Solana Devnet
          </div>

          <h1 className="text-5xl md:text-7xl font-extrabold leading-none tracking-tight mb-6">
            Secure Temporary
            <br />
            <span className="gradient-text text-glow-purple">
              Trading Access
            </span>
          </h1>

          <p className="text-lg text-vault-muted max-w-2xl mx-auto mb-10 leading-relaxed">
            Pre-approve spending limits for automated bots with time-locked
            sessions. Revoke instantly. Keep full control.
          </p>

          <div className="flex flex-wrap items-center justify-center gap-3">
            {walletConnected ? (
              <Link
                href="/dashboard"
                className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl btn-shimmer text-white font-bold text-base shadow-lg shadow-sol-purple/30 hover:brightness-110 transition-all"
              >
                Open Dashboard <ArrowRight size={18} />
              </Link>
            ) : (
              <button
                onClick={() => {}}
                className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl btn-shimmer text-white font-bold text-base shadow-lg shadow-sol-purple/30 hover:brightness-110 transition-all"
              >
                Get Started <ArrowRight size={18} />
              </button>
            )}
            <Link
              href="/create"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-xl border border-vault-border text-vault-muted text-base font-semibold hover:text-white hover:border-sol-purple/40 transition-all"
            >
              Create Vault <ChevronRight size={18} />
            </Link>
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="relative z-10 max-w-5xl mx-auto px-6 pb-24">
        <div className="grid md:grid-cols-3 gap-4">
          {[
            {
              icon: <Timer size={24} className="text-sol-green" />,
              title: "Time-Limited Access",
              desc: "Sessions expire automatically. No permanent approvals, no forgotten delegates.",
              color: "green" as const,
            },
            {
              icon: <Shield size={24} className="text-sol-purple" />,
              title: "Spending Caps",
              desc: "Pre-set exact limits. Bots cannot exceed your authorized amount.",
              color: "purple" as const,
            },
            {
              icon: <Zap size={24} className="text-sol-coral" />,
              title: "Instant Revoke",
              desc: "One click to terminate. Funds return to your wallet immediately.",
              color: "coral" as const,
            },
          ].map((f, i) => (
            <div
              key={f.title}
              className={`p-6 rounded-2xl border bg-vault-surface/60 backdrop-blur card-hover transition-all fade-in-up-${i + 1}`}
              style={{
                borderColor:
                  f.color === "green"
                    ? "rgba(20,241,149,0.15)"
                    : f.color === "purple"
                      ? "rgba(153,69,255,0.15)"
                      : "rgba(255,107,157,0.15)",
              }}
            >
              <div
                className="w-12 h-12 rounded-xl flex items-center justify-center mb-4"
                style={{
                  background:
                    f.color === "green"
                      ? "rgba(20,241,149,0.1)"
                      : f.color === "purple"
                        ? "rgba(153,69,255,0.1)"
                        : "rgba(255,107,157,0.1)",
                }}
              >
                {f.icon}
              </div>
              <h3 className="text-base font-bold text-white mb-2">{f.title}</h3>
              <p className="text-sm text-vault-muted leading-relaxed">
                {f.desc}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* Stats bar */}
      <section className="relative z-10 border-t border-vault-border bg-vault-surface/30 backdrop-blur">
        <div className="max-w-5xl mx-auto px-6 py-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
            {[
              {
                label: "Total Vaults Created",
                value: "1,247",
                icon: <Lock size={14} />,
              },
              {
                label: "SOL Protected",
                value: "84,392",
                icon: <Shield size={14} />,
              },
              {
                label: "Trades Executed",
                value: "203K",
                icon: <TrendingUp size={14} />,
              },
              {
                label: "Avg Session",
                value: "47 min",
                icon: <Timer size={14} />,
              },
            ].map((s) => (
              <div key={s.label}>
                <div className="flex items-center justify-center gap-1.5 text-vault-muted text-xs mb-1">
                  {s.icon} {s.label}
                </div>
                <p className="text-2xl font-bold mono gradient-text">
                  {s.value}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="relative z-10 border-t border-vault-border py-6">
        <div className="max-w-5xl mx-auto px-6 flex flex-col md:flex-row items-center justify-between gap-4 text-xs text-vault-muted">
          <div className="flex items-center gap-2">
            <Shield size={12} className="text-sol-purple" />
            <span>Ephemeral Vault — Built on Solana</span>
          </div>
          <div className="flex items-center gap-4">
            <a href="#" className="hover:text-white transition-colors">
              Docs
            </a>
            <a href="#" className="hover:text-white transition-colors">
              GitHub
            </a>
            <a href="#" className="hover:text-white transition-colors">
              Audit
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}