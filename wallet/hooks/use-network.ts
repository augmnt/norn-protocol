"use client";

import { useNetworkStore } from "@/stores/network-store";
import { NETWORKS } from "@/lib/networks";

export function useNetwork() {
  const activeNetworkId = useNetworkStore((s) => s.activeNetworkId);
  const customRpcUrl = useNetworkStore((s) => s.customRpcUrl);
  const setNetwork = useNetworkStore((s) => s.setNetwork);
  const setCustomRpc = useNetworkStore((s) => s.setCustomRpc);

  const network = NETWORKS[activeNetworkId] ?? NETWORKS.devnet;
  const rpcUrl = customRpcUrl || network.rpcUrl;
  const wsUrl = useNetworkStore.getState().customWsUrl || network.wsUrl;

  return {
    activeNetworkId,
    network,
    rpcUrl,
    wsUrl,
    isTestnet: network.isTestnet,
    setNetwork,
    setCustomRpc,
  };
}
