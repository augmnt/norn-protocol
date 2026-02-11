"use client";

import { NornClient } from "@norn-protocol/sdk";

const RPC_PROXY = "/api/rpc";

let clientInstance: NornClient | null = null;
let rpcIdCounter = 0;

export function getClient(): NornClient {
  if (!clientInstance) {
    clientInstance = new NornClient({ url: RPC_PROXY });
  }
  return clientInstance;
}

export async function rpcCall<T>(
  method: string,
  params: unknown[] = []
): Promise<T> {
  const res = await fetch(RPC_PROXY, {
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
