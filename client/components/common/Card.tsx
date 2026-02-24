import React from 'react';
import { clsx } from 'clsx';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  elevated?: boolean;
  hover?: boolean;
  glow?: 'purple' | 'green' | 'coral' | null;
}

export function Card({ children, className, elevated, hover, glow }: CardProps) {
  return (
    <div className={clsx(
      'rounded-xl border p-6',
      'bg-vault-surface border-vault-border',
      elevated && 'bg-[#1F1F2E] border-sol-purple/20',
      hover && 'card-hover cursor-pointer',
      glow === 'purple' && 'border-sol-purple/30 shadow-lg shadow-sol-purple/10',
      glow === 'green' && 'border-sol-green/30 shadow-lg shadow-sol-green/10',
      glow === 'coral' && 'border-sol-coral/30 shadow-lg shadow-sol-coral/10',
      className
    )}>
      {children}
    </div>
  );
}

interface CardHeaderProps {
  title: string;
  subtitle?: string;
  action?: React.ReactNode;
  icon?: React.ReactNode;
  badge?: React.ReactNode;
}

export function CardHeader({ title, subtitle, action, icon, badge }: CardHeaderProps) {
  return (
    <div className="flex items-start justify-between mb-5">
      <div className="flex items-center gap-3">
        {icon && (
          <div className="w-9 h-9 rounded-lg bg-sol-purple/10 border border-sol-purple/20 flex items-center justify-center text-sol-purple">
            {icon}
          </div>
        )}
        <div>
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-semibold text-white/90 tracking-wide uppercase">{title}</h3>
            {badge}
          </div>
          {subtitle && <p className="text-xs text-vault-muted mt-0.5">{subtitle}</p>}
        </div>
      </div>
      {action && <div className="flex-shrink-0">{action}</div>}
    </div>
  );
}
