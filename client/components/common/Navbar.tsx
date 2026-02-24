'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Shield, Globe } from 'lucide-react';
import { WalletButton } from '@/components/wallet/WalletButton';
import { clsx } from 'clsx';

export function Navbar() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-40 border-b border-vault-border bg-vault-bg/80 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 h-14 flex items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2.5 group">
          <div className="w-7 h-7 rounded-lg bg-sol-purple/20 border border-sol-purple/40 flex items-center justify-center group-hover:border-sol-purple/70 transition-colors">
            <Shield size={14} className="text-sol-purple" />
          </div>
          <span className="text-sm font-bold tracking-tight">
            Ephemeral <span className="text-sol-purple">Vault</span>
          </span>
        </Link>

        {/* Nav Links */}
        <nav className="hidden md:flex items-center gap-1">
          {[
            { href: '/dashboard', label: 'Dashboard' },
            { href: '/create', label: 'Create Vault' },
          ].map(({ href, label }) => (
            <Link
              key={href}
              href={href}
              className={clsx(
                'px-3 py-1.5 rounded-lg text-sm transition-colors',
                pathname === href
                  ? 'text-white bg-white/5'
                  : 'text-vault-muted hover:text-white hover:bg-white/5'
              )}
            >
              {label}
            </Link>
          ))}
        </nav>

        {/* Right side */}
        <div className="flex items-center gap-3">
          <div className="hidden sm:flex items-center gap-1.5 text-xs text-vault-muted">
            <Globe size={11} />
            <span className="mono">Mainnet</span>
          </div>
          <div className="w-px h-4 bg-vault-border hidden sm:block" />
          <WalletButton />
        </div>
      </div>
    </header>
  );
}
