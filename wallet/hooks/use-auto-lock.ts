"use client";

import { useEffect, useRef } from "react";
import { useWalletStore } from "@/stores/wallet-store";
import { useSettingsStore } from "@/stores/settings-store";

export function useAutoLock() {
  const walletState = useWalletStore((s) => s.state);
  const setState = useWalletStore((s) => s.setState);
  const timeout = useSettingsStore((s) => s.autoLockTimeout);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    if (walletState !== "unlocked") return;

    const resetTimer = () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        setState("locked");
      }, timeout);
    };

    const handleVisibilityChange = () => {
      if (document.hidden) {
        // Clear any existing timer before starting a new one
        if (timerRef.current) clearTimeout(timerRef.current);
        timerRef.current = setTimeout(() => {
          setState("locked");
        }, timeout);
      } else {
        resetTimer();
      }
    };

    const handleActivity = () => resetTimer();

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("mousemove", handleActivity);
    window.addEventListener("keydown", handleActivity);
    window.addEventListener("touchstart", handleActivity);

    resetTimer();

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("mousemove", handleActivity);
      window.removeEventListener("keydown", handleActivity);
      window.removeEventListener("touchstart", handleActivity);
    };
  }, [walletState, setState, timeout]);
}
