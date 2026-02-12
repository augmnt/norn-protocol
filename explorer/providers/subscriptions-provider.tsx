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
import { truncateAddress } from "@/lib/format";
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
    let openCount = 0;

    function updateConnected(delta: 1 | -1) {
      openCount = Math.max(0, openCount + delta);
      const nowConnected = openCount > 0;
      if (useRealtimeStore.getState().connected !== nowConnected) {
        useRealtimeStore.getState().setConnected(nowConnected);
      }
    }

    const makeWsOpts = () => ({
      url: config.wsUrl,
      onOpen: () => {
        updateConnected(1);
      },
      onClose: () => {
        updateConnected(-1);
      },
      onError: () => {
        // onClose will follow
      },
    });

    const blockSub = subscribeNewBlocks(
      makeWsOpts(),
      (block) => {
        useRealtimeStore.getState().addBlock(block);
        // Invalidate queries affected by new block
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.weaveState });
        queryClient.invalidateQueries({ queryKey: ["blocks"] });
        queryClient.invalidateQueries({ queryKey: ["blockTransactions"] });
        queryClient.invalidateQueries({ queryKey: ["recentBlocks"] });

        const txCount = block.transfer_count;
        toast(`Block #${block.height.toLocaleString()}`, {
          description: txCount > 0 ? `${txCount} transaction${txCount !== 1 ? "s" : ""}` : "Empty block",
          duration: 3000,
        });
      }
    );

    const transferSub = subscribeTransfers(
      makeWsOpts(),
      (transfer) => {
        useRealtimeStore.getState().addTransfer(transfer);
        // Invalidate queries for affected addresses
        if (transfer.from) {
          queryClient.invalidateQueries({ queryKey: ["balance", transfer.from] });
          queryClient.invalidateQueries({ queryKey: ["threadState", transfer.from] });
          queryClient.invalidateQueries({ queryKey: ["txHistory", transfer.from] });
        }
        if (transfer.to) {
          queryClient.invalidateQueries({ queryKey: ["balance", transfer.to] });
          queryClient.invalidateQueries({ queryKey: ["threadState", transfer.to] });
          queryClient.invalidateQueries({ queryKey: ["txHistory", transfer.to] });
        }

        toast("Transfer", {
          description: `${truncateAddress(transfer.from)} → ${truncateAddress(transfer.to)}: ${transfer.human_readable} ${transfer.symbol ?? "NORN"}`,
          duration: 4000,
        });
      }
    );

    subsRef.current = [blockSub, transferSub];

    return () => {
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
      useRealtimeStore.getState().setConnected(false);
    };
  }, [queryClient]);

  return <>{children}</>;
}
