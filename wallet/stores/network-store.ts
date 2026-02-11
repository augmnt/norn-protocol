"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";
import { NETWORKS, DEFAULT_NETWORK, type NetworkConfig } from "@/lib/networks";

interface NetworkStoreState {
  activeNetworkId: string;
  customRpcUrl: string | null;
  customWsUrl: string | null;

  // Computed
  network: NetworkConfig;
  rpcUrl: string;
  wsUrl: string;

  // Actions
  setNetwork: (networkId: string) => void;
  setCustomRpc: (rpcUrl: string, wsUrl: string) => void;
  clearCustomRpc: () => void;
}

export const useNetworkStore = create<NetworkStoreState>()(
  persist(
    (set, get) => ({
      activeNetworkId: DEFAULT_NETWORK,
      customRpcUrl: null,
      customWsUrl: null,

      get network() {
        return NETWORKS[get().activeNetworkId] ?? NETWORKS[DEFAULT_NETWORK];
      },

      get rpcUrl() {
        const { customRpcUrl, activeNetworkId } = get();
        if (customRpcUrl) return customRpcUrl;
        return (NETWORKS[activeNetworkId] ?? NETWORKS[DEFAULT_NETWORK]).rpcUrl;
      },

      get wsUrl() {
        const { customWsUrl, activeNetworkId } = get();
        if (customWsUrl) return customWsUrl;
        return (NETWORKS[activeNetworkId] ?? NETWORKS[DEFAULT_NETWORK]).wsUrl;
      },

      setNetwork: (networkId) =>
        set({ activeNetworkId: networkId, customRpcUrl: null, customWsUrl: null }),

      setCustomRpc: (rpcUrl, wsUrl) =>
        set({ customRpcUrl: rpcUrl, customWsUrl: wsUrl }),

      clearCustomRpc: () =>
        set({ customRpcUrl: null, customWsUrl: null }),
    }),
    {
      name: "norn-wallet-network",
      partialize: (state) => ({
        activeNetworkId: state.activeNetworkId,
        customRpcUrl: state.customRpcUrl,
        customWsUrl: state.customWsUrl,
      }),
    }
  )
);
