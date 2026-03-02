import type { Metadata } from "next";
import "./globals.css";
import { VaultProvider } from "@/contexts/VaultContext";
import { NotificationProvider } from "@/contexts/NotificationContext";
import { ToastContainer } from "@/components/common/Toast";
import { SolanaProvider } from "@/provider/SolanaProvider";

export const metadata: Metadata = {
  title: "ExecVault — Secure Temporary Trading on Solana",
  description:
    "Pre-approve spending limits for automated trading with time-based session control on Solana.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="scanline">
        <SolanaProvider>
          <NotificationProvider>
            <VaultProvider>
              {children}
              <ToastContainer />
            </VaultProvider>
          </NotificationProvider>
        </SolanaProvider>
      </body>
    </html>
  );
}
