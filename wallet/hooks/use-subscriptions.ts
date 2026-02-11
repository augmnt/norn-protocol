"use client";

import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  subscribeNewBlocks,
  subscribeTransfers,
  subscribeTokenEvents,
  type Subscription,
} from "@norn-protocol/sdk";
import { toast } from "sonner";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useNetworkStore } from "@/stores/network-store";
import { formatNorn, truncateAddress } from "@/lib/format";

const MAX_RECONNECT_DELAY = 30_000;
const BASE_RECONNECT_DELAY = 1_000;

export function useSubscriptions(filterAddress?: string) {
  const queryClient = useQueryClient();
  const subsRef = useRef<Subscription[]>([]);
  const reconnectRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const attemptRef = useRef(0);
  const activeNetworkId = useNetworkStore((s) => s.activeNetworkId);

  useEffect(() => {
    let mounted = true;
    let openCount = 0;

    function updateConnected(delta: 1 | -1) {
      openCount = Math.max(0, openCount + delta);
      const nowConnected = openCount > 0;
      if (useRealtimeStore.getState().connected !== nowConnected) {
        useRealtimeStore.getState().setConnected(nowConnected);
      }
    }

    function invalidateBalances(address: string) {
      // Use 2-element prefix ["balance", address] to match all token variants
      queryClient.invalidateQueries({ queryKey: ["balance", address] });
      queryClient.invalidateQueries({ queryKey: ["threadState", address] });
    }

    function connect() {
      // Clean up existing subs
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
      openCount = 0;
      useRealtimeStore.getState().setConnected(false);

      const wsUrl = useNetworkStore.getState().wsUrl
        ?? useNetworkStore.getState().customWsUrl
        ?? "wss://seed.norn.network";

      const makeWsOpts = () => ({
        url: wsUrl,
        onOpen: () => {
          if (!mounted) return;
          updateConnected(1);
          attemptRef.current = 0;
        },
        onClose: () => {
          if (!mounted) return;
          updateConnected(-1);
          if (openCount === 0) scheduleReconnect();
        },
        onError: () => {
          // onClose will follow — no state change needed here
        },
      });

      const blockSub = subscribeNewBlocks(makeWsOpts(), (block) => {
        useRealtimeStore.getState().addBlock(block);
        queryClient.invalidateQueries({ queryKey: ["weaveState"] });
        if (filterAddress) {
          invalidateBalances(filterAddress);
        }
      });

      const transferSub = subscribeTransfers(
        makeWsOpts(),
        (transfer) => {
          useRealtimeStore.getState().addTransfer(transfer);

          if (filterAddress) {
            const isIncoming = transfer.to?.toLowerCase() === filterAddress.toLowerCase();
            const isOutgoing = transfer.from?.toLowerCase() === filterAddress.toLowerCase();
            if (isIncoming) {
              toast("Incoming Transfer", {
                description: `${truncateAddress(transfer.from)} sent you ${formatNorn(transfer.amount)} NORN`,
                duration: 5000,
              });
            }
            if (isIncoming || isOutgoing) {
              invalidateBalances(filterAddress);
              queryClient.invalidateQueries({ queryKey: ["txHistory", filterAddress] });
            }
          }
        },
        filterAddress
      );

      const tokenSub = subscribeTokenEvents(makeWsOpts(), (event) => {
        useRealtimeStore.getState().addTokenEvent(event);
        if (filterAddress) {
          invalidateBalances(filterAddress);
          queryClient.invalidateQueries({ queryKey: ["createdTokens", filterAddress] });
        }
        queryClient.invalidateQueries({ queryKey: ["tokensList"] });
        if (event.token_id) {
          queryClient.invalidateQueries({ queryKey: ["tokenInfo", event.token_id] });
        }
      });

      subsRef.current = [blockSub, transferSub, tokenSub];
    }

    function scheduleReconnect() {
      if (!mounted) return;
      // Clear any pending reconnect to avoid stacking
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      const delay = Math.min(
        BASE_RECONNECT_DELAY * Math.pow(2, attemptRef.current),
        MAX_RECONNECT_DELAY
      );
      attemptRef.current++;
      reconnectRef.current = setTimeout(() => {
        if (mounted) connect();
      }, delay);
    }

    connect();

    return () => {
      mounted = false;
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
      useRealtimeStore.getState().setConnected(false);
    };
  }, [queryClient, filterAddress, activeNetworkId]);
}
