import { create } from "zustand";
import { DEFAULT_NETWORK, NETWORK_STORAGE_KEY } from "@/lib/config";
import { resetClient } from "@/lib/rpc";

interface NetworkState {
  rpcUrl: string;
  wsUrl: string;
  networkName: string;
  isConnected: boolean;
  setNetwork: (rpcUrl: string, wsUrl: string, name: string) => Promise<void>;
  setConnected: (connected: boolean) => void;
  loadSaved: () => Promise<void>;
}

export const useNetworkStore = create<NetworkState>((set) => ({
  rpcUrl: DEFAULT_NETWORK.rpcUrl,
  wsUrl: DEFAULT_NETWORK.wsUrl,
  networkName: DEFAULT_NETWORK.name,
  isConnected: false,

  setNetwork: async (rpcUrl, wsUrl, name) => {
    set({ rpcUrl, wsUrl, networkName: name });
    resetClient();
    await chrome.storage.local.set({
      [NETWORK_STORAGE_KEY]: { rpcUrl, wsUrl, name },
    });
  },

  setConnected: (connected) => set({ isConnected: connected }),

  loadSaved: async () => {
    const result = await chrome.storage.local.get(NETWORK_STORAGE_KEY);
    const saved = result[NETWORK_STORAGE_KEY];
    if (saved) {
      set({
        rpcUrl: saved.rpcUrl,
        wsUrl: saved.wsUrl,
        networkName: saved.name,
      });
      resetClient();
    }
  },
}));
