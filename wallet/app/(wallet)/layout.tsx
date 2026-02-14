"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@/hooks/use-wallet";
import { useNetwork } from "@/hooks/use-network";
import { WalletHeader } from "@/components/layout/wallet-header";
import { WalletNav } from "@/components/layout/wallet-nav";
import { WalletFooter } from "@/components/layout/wallet-footer";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";
import { AlertTriangle } from "lucide-react";

export default function WalletLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const { state } = useWallet();
  const { network, isTestnet } = useNetwork();
  useKeyboardShortcuts();

  useEffect(() => {
    if (state === "uninitialized") {
      router.replace("/onboarding");
    } else if (state === "locked") {
      router.replace("/unlock");
    }
  }, [state, router]);

  if (state !== "unlocked") {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col">
      <WalletHeader />
      {isTestnet && (
        <div className="flex items-center justify-center gap-2 bg-amber-500/10 border-b border-amber-500/20 px-4 py-1.5">
          <AlertTriangle className="h-3 w-3 text-amber-500" />
          <p className="text-[11px] text-amber-500 font-medium">
            You are on {network.name}. Tokens have no real value.
          </p>
        </div>
      )}
      <div className="flex flex-1">
        <WalletNav />
        <main className="flex-1 overflow-auto pb-24 md:pb-0">
          {children}
        </main>
      </div>
      <WalletFooter />
    </div>
  );
}
