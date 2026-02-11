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

interface RealtimeState {
  connected: boolean;
  connectedCount: number;
  latestBlock: BlockInfo | null;
  recentBlocks: BlockInfo[];
  recentTransfers: TransferEvent[];
  pendingTxs: PendingTransactionEvent[];
  tokenEvents: TokenEvent[];
  loomEvents: LoomExecutionEvent[];

  incrementConnected: () => void;
  decrementConnected: () => void;
  resetConnected: () => void;
  addBlock: (block: BlockInfo) => void;
  addTransfer: (transfer: TransferEvent) => void;
  addPendingTx: (tx: PendingTransactionEvent) => void;
  addTokenEvent: (event: TokenEvent) => void;
  addLoomEvent: (event: LoomExecutionEvent) => void;
  clearPendingTxs: () => void;
}

export const useRealtimeStore = create<RealtimeState>((set) => ({
  connected: false,
  connectedCount: 0,
  latestBlock: null,
  recentBlocks: [],
  recentTransfers: [],
  pendingTxs: [],
  tokenEvents: [],
  loomEvents: [],

  incrementConnected: () =>
    set((state) => {
      const count = state.connectedCount + 1;
      return { connectedCount: count, connected: count > 0 };
    }),
  decrementConnected: () =>
    set((state) => {
      const count = Math.max(0, state.connectedCount - 1);
      return { connectedCount: count, connected: count > 0 };
    }),
  resetConnected: () => set({ connectedCount: 0, connected: false }),

  addBlock: (block) =>
    set((state) => ({
      latestBlock: block,
      recentBlocks: [block, ...state.recentBlocks].slice(0, WS_CAPS.blocks),
    })),

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
