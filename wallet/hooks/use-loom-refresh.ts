"use client";

import { useEffect, useRef } from "react";
import { useRealtimeStore } from "@/stores/realtime-store";

/**
 * Refreshes loom page data via two mechanisms:
 * 1. Instant — reacts to WebSocket loom execution events in the realtime store
 * 2. Fallback — polls on a periodic interval to catch missed events
 *
 * The callback is stored in a ref so it never appears in effect deps,
 * avoiding stale-closure and re-initialization issues.
 */
export function useLoomRefresh(
  loomId: string,
  onRefresh: () => void,
  pollIntervalMs = 8000,
) {
  const loomEvents = useRealtimeStore((s) => s.loomEvents);

  // Always-current callback ref — never in deps
  const onRefreshRef = useRef(onRefresh);
  onRefreshRef.current = onRefresh;

  // Track event count for this loom (-1 = not yet initialized)
  const countRef = useRef(-1);

  // Instant: react to WebSocket loom events
  useEffect(() => {
    const count = loomEvents.filter((e) => e.loom_id === loomId).length;

    if (countRef.current < 0) {
      // First run — record baseline, don't trigger refresh
      countRef.current = count;
      return;
    }

    if (count !== countRef.current) {
      countRef.current = count;
      onRefreshRef.current();
    }
  }, [loomEvents, loomId]);

  // Fallback: periodic poll catches missed WebSocket events
  useEffect(() => {
    const id = setInterval(() => {
      onRefreshRef.current();
    }, pollIntervalMs);
    return () => clearInterval(id);
  }, [pollIntervalMs]);
}
