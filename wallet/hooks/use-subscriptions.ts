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
import { QUERY_KEYS } from "@/lib/constants";
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

    function connect() {
      // Clean up existing subs
      subsRef.current.forEach((sub) => sub.unsubscribe());
      subsRef.current = [];
      useRealtimeStore.getState().resetConnected();

      const wsUrl = useNetworkStore.getState().wsUrl
        ?? useNetworkStore.getState().customWsUrl
        ?? "wss://seed.norn.network";

      const makeWsOpts = () => {
        let isOpen = false;
        return {
          url: wsUrl,
          onOpen: () => {
            if (!mounted || isOpen) return;
            isOpen = true;
            useRealtimeStore.getState().incrementConnected();
            attemptRef.current = 0;
          },
          onClose: () => {
            if (!mounted) return;
            if (isOpen) {
              isOpen = false;
              useRealtimeStore.getState().decrementConnected();
            }
            scheduleReconnect();
          },
          onError: () => {
            if (!mounted) return;
            if (isOpen) {
              isOpen = false;
              useRealtimeStore.getState().decrementConnected();
            }
          },
        };
      };

      const blockSub = subscribeNewBlocks(makeWsOpts(), (block) => {
        useRealtimeStore.getState().addBlock(block);
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.weaveState });
        if (filterAddress) {
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(filterAddress) });
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(filterAddress) });
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
              queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(filterAddress) });
              queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(filterAddress) });
              queryClient.invalidateQueries({ queryKey: ["txHistory", filterAddress] });
            }
          }
        },
        filterAddress
      );

      const tokenSub = subscribeTokenEvents(makeWsOpts(), (event) => {
        useRealtimeStore.getState().addTokenEvent(event);
        if (filterAddress) {
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(filterAddress) });
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(filterAddress) });
          queryClient.invalidateQueries({ queryKey: ["createdTokens", filterAddress] });
        }
        queryClient.invalidateQueries({ queryKey: ["tokensList"] });
        if (event.token_id) {
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tokenInfo(event.token_id) });
        }
      });

      subsRef.current = [blockSub, transferSub, tokenSub];
    }

    function scheduleReconnect() {
      if (!mounted) return;
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
      useRealtimeStore.getState().resetConnected();
    };
  }, [queryClient, filterAddress, activeNetworkId]);
}
