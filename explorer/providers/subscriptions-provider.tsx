"use client";

import { useEffect, useRef } from "react";
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
import { formatNorn, truncateAddress } from "@/lib/format";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";

export function SubscriptionsProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const queryClient = useQueryClient();
  const subsRef = useRef<Subscription[]>([]);
  useKeyboardShortcuts();

  useEffect(() => {
    const wsOpts = {
      url: config.wsUrl,
      onOpen: () => {
        console.log("[WS] Connected to", config.wsUrl);
        useRealtimeStore.getState().setConnected(true);
      },
      onClose: () => {
        console.log("[WS] Disconnected");
        useRealtimeStore.getState().setConnected(false);
      },
      onError: (e: Event) => {
        console.error("[WS] Error:", e);
        useRealtimeStore.getState().setConnected(false);
      },
    };

    const blockSub = subscribeNewBlocks(
      { ...wsOpts },
      (block) => {
        console.log("[WS] New block:", block.height);
        useRealtimeStore.getState().addBlock(block);
        // Invalidate weave state so dashboard stats refresh.
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.weaveState });

        const txCount = block.transfer_count;
        toast(`Block #${block.height.toLocaleString()}`, {
          description: txCount > 0 ? `${txCount} transaction${txCount !== 1 ? "s" : ""}` : "Empty block",
          duration: 3000,
        });
      }
    );

    const transferSub = subscribeTransfers(
      { ...wsOpts },
      (transfer) => {
        console.log("[WS] New transfer:", transfer.from, "->", transfer.to, transfer.amount);
        useRealtimeStore.getState().addTransfer(transfer);

        toast("Transfer", {
          description: `${truncateAddress(transfer.from)} → ${truncateAddress(transfer.to)}: ${formatNorn(transfer.amount)} NORN`,
          duration: 4000,
        });
      }
    );

    subsRef.current = [blockSub, transferSub];

    return () => {
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
    };
  }, [queryClient]);

  return <>{children}</>;
}
