"use client";

import { useEffect, useState } from "react";
import { useWalletStore } from "@/stores/wallet-store";
import { initialize } from "@/lib/wallet-manager";
import { useAutoLock } from "@/hooks/use-auto-lock";

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function init() {
      try {
        const { state, meta, prfSupported } = await initialize();
        useWalletStore.getState().setState(state);
        useWalletStore.getState().setMeta(meta);
        useWalletStore.getState().setPrfSupported(prfSupported);
      } catch (err) {
        console.error("Wallet initialization failed:", err);
        setError(err instanceof Error ? err.message : "Failed to initialize wallet");
      } finally {
        setReady(true);
      }
    }
    init();
  }, []);

  useAutoLock();

  if (!ready) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center p-4">
        <div className="text-center space-y-3 max-w-sm">
          <p className="text-sm font-medium text-destructive">Wallet initialization failed</p>
          <p className="text-xs text-muted-foreground">{error}</p>
          <button
            onClick={() => window.location.reload()}
            className="text-xs text-foreground underline underline-offset-4 hover:text-muted-foreground transition-colors"
          >
            Reload page
          </button>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
