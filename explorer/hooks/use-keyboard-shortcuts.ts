"use client";

import { useEffect, useRef } from "react";
import { useRouter } from "next/navigation";

export function useKeyboardShortcuts() {
  const router = useRouter();
  const pendingRef = useRef<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Skip if user is typing in an input
      const target = e.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        return;
      }

      // Cmd+K is handled by SearchBar
      if (e.metaKey || e.ctrlKey || e.altKey) return;

      const key = e.key.toLowerCase();

      // Two-key chord: g + <letter>
      if (pendingRef.current === "g") {
        pendingRef.current = null;
        if (timerRef.current) clearTimeout(timerRef.current);

        const routes: Record<string, string> = {
          d: "/",
          b: "/blocks",
          t: "/transactions",
          k: "/tokens",
          c: "/contracts",
          v: "/validators",
        };

        if (routes[key]) {
          e.preventDefault();
          router.push(routes[key]);
        }
        return;
      }

      if (key === "g") {
        pendingRef.current = "g";
        timerRef.current = setTimeout(() => {
          pendingRef.current = null;
        }, 500);
        return;
      }

      // ? to show shortcuts help (not implemented as modal, just logs for now)
    };

    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("keydown", handler);
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [router]);
}
