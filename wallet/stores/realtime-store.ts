"use client";

import { create } from "zustand";
import type {
  BlockInfo,
  TransferEvent,
  PendingTransactionEvent,
  TokenEvent,
  LoomExecutionEvent,
} from "@/types";
import { WS_CAPS } from "@/lib/constants";

type ConnectionState = "connecting" | "connected" | "disconnected";

interface RealtimeState {
  connected: boolean;
  connectionState: ConnectionState;
  latestBlock: BlockInfo | null;
  recentBlocks: BlockInfo[];
  recentTransfers: TransferEvent[];
  pendingTxs: PendingTransactionEvent[];
  tokenEvents: TokenEvent[];
  loomEvents: LoomExecutionEvent[];

  setConnected: (connected: boolean) => void;
  addBlock: (block: BlockInfo) => void;
  addTransfer: (transfer: TransferEvent) => void;
  addPendingTx: (tx: PendingTransactionEvent) => void;
  addTokenEvent: (event: TokenEvent) => void;
  addLoomEvent: (event: LoomExecutionEvent) => void;
  clearPendingTxs: () => void;
}

export const useRealtimeStore = create<RealtimeState>((set) => ({
  connected: false,
  connectionState: "connecting" as ConnectionState,
  latestBlock: null,
  recentBlocks: [],
  recentTransfers: [],
  pendingTxs: [],
  tokenEvents: [],
  loomEvents: [],

  setConnected: (connected) =>
    set({ connected, connectionState: connected ? "connected" : "disconnected" }),

  addBlock: (block) =>
    set((state) => {
      // Deduplicate by block height
      if (state.recentBlocks.some((b) => b.height === block.height)) {
        return { latestBlock: block };
      }
      return {
        latestBlock: block,
        recentBlocks: [block, ...state.recentBlocks].slice(0, WS_CAPS.blocks),
      };
    }),

  addTransfer: (transfer) =>
    set((state) => ({
      recentTransfers: [transfer, ...state.recentTransfers].slice(
        0,
        WS_CAPS.transfers
      ),
    })),

  addPendingTx: (tx) =>
    set((state) => ({
      pendingTxs: [tx, ...state.pendingTxs].slice(0, WS_CAPS.pendingTxs),
    })),

  addTokenEvent: (event) =>
    set((state) => ({
      tokenEvents: [event, ...state.tokenEvents].slice(
        0,
        WS_CAPS.tokenEvents
      ),
    })),

  addLoomEvent: (event) =>
    set((state) => ({
      loomEvents: [event, ...state.loomEvents].slice(0, WS_CAPS.loomEvents),
    })),

  clearPendingTxs: () => set({ pendingTxs: [] }),
}));
