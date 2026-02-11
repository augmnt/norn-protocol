"use client";

import { useNetworkStore } from "@/stores/network-store";
import { NETWORKS, DEFAULT_NETWORK } from "@/lib/networks";

/** Get the current explorer base URL from the active network. */
function baseUrl(): string {
  const { activeNetworkId, customRpcUrl } = useNetworkStore.getState();
  // If using a custom RPC, assume local explorer
  if (customRpcUrl) return "http://localhost:3001";
  return (NETWORKS[activeNetworkId] ?? NETWORKS[DEFAULT_NETWORK]).explorerUrl;
}

export function explorerBlockUrl(height: number | string): string {
  return `${baseUrl()}/block/${height}`;
}

export function explorerTxUrl(hash: string): string {
  return `${baseUrl()}/tx/${hash}`;
}

export function explorerAddressUrl(address: string): string {
  return `${baseUrl()}/address/${address}`;
}

export function explorerTokenUrl(tokenId: string): string {
  return `${baseUrl()}/token/${tokenId}`;
}

export function explorerContractUrl(loomId: string): string {
  return `${baseUrl()}/contract/${loomId}`;
}
