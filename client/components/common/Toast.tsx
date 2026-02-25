'use client';

import React from 'react';
import { CheckCircle2, AlertTriangle, XCircle, Info, X } from 'lucide-react';
import { useNotification } from '@/contexts/NotificationContext';
import { clsx } from 'clsx';

const icons = {
  success: <CheckCircle2 size={16} className="text-sol-green" />,
  warning: <AlertTriangle size={16} className="text-amber-400" />,
  error: <XCircle size={16} className="text-red-400" />,
  info: <Info size={16} className="text-blue-400" />,
};

const styles = {
  success: 'border-sol-green/30 bg-sol-green/5',
  warning: 'border-amber-400/30 bg-amber-400/5',
  error: 'border-red-400/30 bg-red-400/5',
  info: 'border-blue-400/30 bg-blue-400/5',
};

export function ToastContainer() {
  const { toasts, removeToast } = useNotification();

  return (
    <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
      {toasts.map(toast => (
        <div
          key={toast.id}
          className={clsx(
            'flex items-center gap-3 px-4 py-3 rounded-xl border backdrop-blur-sm',
            'pointer-events-auto min-w-[280px] max-w-sm',
            'toast-enter',
            styles[toast.type]
          )}
        >
          {icons[toast.type]}
          <span className="text-sm text-white/90 flex-1 font-mono">{toast.message}</span>
          <button
            onClick={() => removeToast(toast.id)}
            className="text-vault-muted hover:text-white transition-colors ml-2"
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
