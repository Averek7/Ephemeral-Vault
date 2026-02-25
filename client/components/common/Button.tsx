'use client';

import React from 'react';
import { clsx } from 'clsx';
import { Loader2 } from 'lucide-react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline-green';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  children: React.ReactNode;
}

export function Button({
  variant = 'primary',
  size = 'md',
  loading = false,
  children,
  className,
  disabled,
  ...props
}: ButtonProps) {
  return (
    <button
      {...props}
      disabled={disabled || loading}
      className={clsx(
        'relative inline-flex items-center justify-center gap-2 font-semibold rounded-lg transition-all duration-150 cursor-pointer select-none',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        // Size
        size === 'sm' && 'px-3 py-1.5 text-sm',
        size === 'md' && 'px-5 py-2.5 text-sm',
        size === 'lg' && 'px-7 py-3.5 text-base',
        // Variants
        variant === 'primary' && [
          'btn-shimmer text-white',
          'hover:brightness-110 active:scale-[0.98]',
          'shadow-lg shadow-sol-purple/20',
        ],
        variant === 'secondary' && [
          'border border-sol-purple/60 text-sol-purple bg-sol-purple/5',
          'hover:bg-sol-purple/15 hover:border-sol-purple active:scale-[0.98]',
        ],
        variant === 'danger' && [
          'bg-red-500 text-white',
          'hover:bg-red-600 active:scale-[0.98]',
          'shadow-lg shadow-red-500/20',
        ],
        variant === 'ghost' && [
          'text-vault-muted hover:text-white hover:bg-white/5 active:scale-[0.98]',
        ],
        variant === 'outline-green' && [
          'border border-sol-green/60 text-sol-green bg-sol-green/5',
          'hover:bg-sol-green/15 hover:border-sol-green active:scale-[0.98]',
        ],
        className
      )}
    >
      {loading && <Loader2 size={14} className="animate-spin" />}
      {children}
    </button>
  );
}
