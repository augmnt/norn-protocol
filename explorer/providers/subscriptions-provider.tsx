"use client";

import { useEffect, useRef, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  subscribeNewBlocks,
  subscribeTransfers,
  type Subscription,
} from "@norn-protocol/sdk";
import { toast } from "sonner";
import { config } from "@/lib/config";
import { useRealtimeStore } from "@/stores/realtime-store";
import { QUERY_KEYS } from "@/lib/constants";
import { truncateAddress } from "@/lib/format";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";

/** Min/max reconnect delay in ms. */
const RECONNECT_MIN_MS = 1_000;
const RECONNECT_MAX_MS = 30_000;

export function SubscriptionsProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const queryClient = useQueryClient();
  const subsRef = useRef<Subscription[]>([]);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const attemptRef = useRef(0);
  const unmountedRef = useRef(false);
  useKeyboardShortcuts();

  const connect = useCallback(() => {
    if (unmountedRef.current) return;

    // Clean up any existing subscriptions
    subsRef.current.forEach((sub) => sub.unsubscribe());
    subsRef.current = [];

    let openCount = 0;

    function updateConnected(delta: 1 | -1) {
      openCount = Math.max(0, openCount + delta);
      const nowConnected = openCount > 0;
      if (useRealtimeStore.getState().connected !== nowConnected) {
        useRealtimeStore.getState().setConnected(nowConnected);
      }
    }

    function scheduleReconnect() {
      if (unmountedRef.current) return;
      // Only schedule if fully disconnected (both sockets closed)
      if (openCount > 0) return;

      const delay = Math.min(
        RECONNECT_MIN_MS * 2 ** attemptRef.current,
        RECONNECT_MAX_MS,
      );
      attemptRef.current++;

      // Show connecting state while waiting
      const store = useRealtimeStore.getState();
      if (store.connectionState !== "connecting") {
        store.setConnectionState("connecting");
      }

      reconnectTimerRef.current = setTimeout(() => {
        reconnectTimerRef.current = null;
        connect();
      }, delay);
    }

    const makeWsOpts = () => ({
      url: config.wsUrl,
      onOpen: () => {
        attemptRef.current = 0; // Reset backoff on success
        updateConnected(1);
      },
      onClose: () => {
        updateConnected(-1);
        scheduleReconnect();
      },
      onError: () => {
        // onClose will follow
      },
    });

    const blockSub = subscribeNewBlocks(makeWsOpts(), (block) => {
      useRealtimeStore.getState().addBlock(block);
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.weaveState });
      queryClient.invalidateQueries({ queryKey: ["blocks"] });
      queryClient.invalidateQueries({ queryKey: ["blockTransactions"] });
      queryClient.invalidateQueries({ queryKey: ["recentBlocks"] });

      const txCount = block.transfer_count;
      toast(`Block #${block.height.toLocaleString()}`, {
        description:
          txCount > 0
            ? `${txCount} transaction${txCount !== 1 ? "s" : ""}`
            : "Empty block",
        duration: 3000,
      });
    });

    const transferSub = subscribeTransfers(makeWsOpts(), (transfer) => {
      useRealtimeStore.getState().addTransfer(transfer);
      if (transfer.from) {
        queryClient.invalidateQueries({
          queryKey: ["balance", transfer.from],
        });
        queryClient.invalidateQueries({
          queryKey: ["threadState", transfer.from],
        });
        queryClient.invalidateQueries({
          queryKey: ["txHistory", transfer.from],
        });
      }
      if (transfer.to) {
        queryClient.invalidateQueries({
          queryKey: ["balance", transfer.to],
        });
        queryClient.invalidateQueries({
          queryKey: ["threadState", transfer.to],
        });
        queryClient.invalidateQueries({
          queryKey: ["txHistory", transfer.to],
        });
      }

      toast("Transfer", {
        description: `${truncateAddress(transfer.from)} → ${truncateAddress(transfer.to)}: ${transfer.human_readable} ${transfer.symbol ?? "NORN"}`,
        duration: 4000,
      });
    });

    subsRef.current = [blockSub, transferSub];
  }, [queryClient]);

  useEffect(() => {
    unmountedRef.current = false;
    connect();

    return () => {
      unmountedRef.current = true;
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
      useRealtimeStore.getState().setConnected(false);
    };
  }, [connect]);

  return <>{children}</>;
}
