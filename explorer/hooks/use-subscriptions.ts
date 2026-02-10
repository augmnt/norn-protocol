"use client";

import { useEffect, useRef } from "react";
import {
  subscribePendingTransactions,
  subscribeTokenEvents,
  subscribeLoomEvents,
  subscribeTransfers,
  type Subscription,
} from "@norn-protocol/sdk";
import { config } from "@/lib/config";
import { useRealtimeStore } from "@/stores/realtime-store";

const wsOpts = {
  url: config.wsUrl,
};

export function usePendingTxSubscription(enabled = true) {
  const subRef = useRef<Subscription | null>(null);

  useEffect(() => {
    if (!enabled) return;

    subRef.current = subscribePendingTransactions(wsOpts, (event) => {
      useRealtimeStore.getState().addPendingTx(event);
    });

    return () => {
      subRef.current?.unsubscribe();
      subRef.current = null;
    };
  }, [enabled]);
}

export function useTokenEventsSubscription(
  tokenId?: string,
  enabled = true
) {
  const subRef = useRef<Subscription | null>(null);

  useEffect(() => {
    if (!enabled) return;

    subRef.current = subscribeTokenEvents(
      wsOpts,
      (event) => {
        useRealtimeStore.getState().addTokenEvent(event);
      },
      tokenId
    );

    return () => {
      subRef.current?.unsubscribe();
      subRef.current = null;
    };
  }, [tokenId, enabled]);
}

export function useLoomEventsSubscription(
  loomId?: string,
  enabled = true
) {
  const subRef = useRef<Subscription | null>(null);

  useEffect(() => {
    if (!enabled) return;

    subRef.current = subscribeLoomEvents(
      wsOpts,
      (event) => {
        useRealtimeStore.getState().addLoomEvent(event);
      },
      loomId
    );

    return () => {
      subRef.current?.unsubscribe();
      subRef.current = null;
    };
  }, [loomId, enabled]);
}

export function useAddressTransfersSubscription(
  address?: string,
  enabled = true
) {
  const subRef = useRef<Subscription | null>(null);

  useEffect(() => {
    if (!enabled || !address) return;

    subRef.current = subscribeTransfers(
      wsOpts,
      (transfer) => {
        useRealtimeStore.getState().addTransfer(transfer);
      },
      address
    );

    return () => {
      subRef.current?.unsubscribe();
      subRef.current = null;
    };
  }, [address, enabled]);
}
