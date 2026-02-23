import type { Metadata } from 'next';
import '@/app/globals.css';
import { VaultProvider } from '@/contexts/VaultContext';
import { NotificationProvider } from '@/contexts/NotificationContext';
// import { ToastContainer } from '@/components/common/Toast';

export const metadata: Metadata = {
  title: 'Ephemeral Vault — Secure Temporary Trading on Solana',
  description: 'Pre-approve spending limits for automated trading with time-based session control on Solana.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="scanline">
        <NotificationProvider>
          <VaultProvider>
            {children}
            {/* <ToastContainer /> */}
          </VaultProvider>
        </NotificationProvider>
      </body>
    </html>
  );
}
