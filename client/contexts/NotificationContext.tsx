'use client';

import React, { createContext, useContext, useState, useCallback } from 'react';
import { Toast, ToastType } from '@/lib/types';

interface NotificationContextType {
  toasts: Toast[];
  addToast: (message: string, type?: ToastType, duration?: number) => void;
  removeToast: (id: string) => void;
}

const NotificationContext = createContext<NotificationContextType>({
  toasts: [],
  addToast: () => {},
  removeToast: () => {},
});

export function NotificationProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  const addToast = useCallback((message: string, type: ToastType = 'info', duration = 5000) => {
    const id = Math.random().toString(36).slice(2);
    const toast: Toast = { id, message, type, duration };
    setToasts(prev => [...prev, toast]);
    setTimeout(() => removeToast(id), duration);
  }, [removeToast]);

  return (
    <NotificationContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
    </NotificationContext.Provider>
  );
}

export const useNotification = () => useContext(NotificationContext);
