"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "./use-wallet";
import { usePasskeyAuth } from "./use-passkey-auth";

/**
 * Global keyboard shortcuts for the wallet.
 * - Cmd/Ctrl+K: Go to send page
 * - Cmd/Ctrl+Shift+R: Go to receive page
 * - Cmd/Ctrl+L: Lock wallet
 * - Escape: Close any open dialog (handled natively by radix)
 */
export function useKeyboardShortcuts() {
  const router = useRouter();
  const { isUnlocked } = useWallet();
  const { lock } = usePasskeyAuth();

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!isUnlocked) return;

      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;

      // Don't capture if typing in an input
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;

      if (e.key === "k") {
        e.preventDefault();
        router.push("/send");
      } else if (e.key === "l") {
        e.preventDefault();
        lock();
      } else if (e.key === "r" && e.shiftKey) {
        e.preventDefault();
        router.push("/receive");
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isUnlocked, router, lock]);
}
