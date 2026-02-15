"use client";

import { useEffect, useRef } from "react";
import { useRealtimeStore } from "@/stores/realtime-store";

export function useLoomRefresh(loomId: string, onRefresh: () => void) {
  const loomEvents = useRealtimeStore((s) => s.loomEvents);
  const initializedRef = useRef(false);
  const seenHeightRef = useRef(0);

  useEffect(() => {
    const latest = loomEvents.find((e) => e.loom_id === loomId);
    if (!initializedRef.current) {
      initializedRef.current = true;
      if (latest) seenHeightRef.current = latest.block_height;
      return;
    }
    if (latest && latest.block_height > seenHeightRef.current) {
      seenHeightRef.current = latest.block_height;
      onRefresh();
    }
  }, [loomEvents, loomId, onRefresh]);
}
