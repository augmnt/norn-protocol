"use client";

import { NornClient } from "@norn-protocol/sdk";
import { useNetworkStore } from "@/stores/network-store";

const RPC_PROXY = "/api/rpc";

let clientInstance: NornClient | null = null;
let lastUrl: string | null = null;
let rpcIdCounter = 0;

export function getClient(): NornClient {
  const url = typeof window !== "undefined"
    ? useNetworkStore.getState().rpcUrl
    : RPC_PROXY;

  if (!clientInstance || lastUrl !== url) {
    clientInstance = new NornClient({ url });
    lastUrl = url;
  }
  return clientInstance;
}

function isLocalhost(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.hostname === "localhost" || parsed.hostname === "127.0.0.1";
  } catch {
    return false;
  }
}

export async function rpcCall<T>(
  method: string,
  params: unknown[] = []
): Promise<T> {
  const url = typeof window !== "undefined"
    ? useNetworkStore.getState().rpcUrl
    : RPC_PROXY;

  // Enforce HTTPS for non-localhost, non-proxy URLs
  if (!url.startsWith("/") && !isLocalhost(url) && !url.startsWith("https://")) {
    throw new Error("Non-local RPC endpoints must use HTTPS");
  }

  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: ++rpcIdCounter,
      method,
      params,
    }),
  });

  if (!res.ok) {
    throw new Error(`RPC HTTP error: ${res.status}`);
  }

  const json = await res.json();
  if (json.error) {
    throw new Error(json.error.message || "RPC error");
  }
  return json.result as T;
}
