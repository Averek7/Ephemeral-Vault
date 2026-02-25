import React from 'react';
import { clsx } from 'clsx';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  hint?: string;
  error?: string;
  suffix?: React.ReactNode;
  prefix?: React.ReactNode;
}

export function Input({ label, hint, error, suffix, prefix, className, ...props }: InputProps) {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-xs font-medium text-vault-muted mb-1.5 uppercase tracking-wider">
          {label}
        </label>
      )}
      <div className="relative flex items-center">
        {prefix && (
          <div className="absolute left-3 text-vault-muted">{prefix}</div>
        )}
        <input
          {...props}
          className={clsx(
            'w-full rounded-lg px-4 py-3 text-sm',
            'bg-vault-bg border border-vault-border',
            'text-white placeholder:text-vault-border',
            'focus:border-sol-purple focus:ring-2 focus:ring-sol-purple/20',
            'transition-all duration-150',
            prefix && 'pl-10',
            suffix && 'pr-16',
            error && 'border-red-500 focus:border-red-500 focus:ring-red-500/20',
            className
          )}
        />
        {suffix && (
          <div className="absolute right-3 text-xs font-semibold text-vault-muted mono">{suffix}</div>
        )}
      </div>
      {hint && !error && <p className="text-xs text-vault-muted mt-1">{hint}</p>}
      {error && <p className="text-xs text-red-400 mt-1">{error}</p>}
    </div>
  );
}
