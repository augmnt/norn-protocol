import type { TransactionHistoryEntry, TokenInfo, SubmitResult, NameInfo } from "@norn-protocol/sdk";
import { useNetworkStore } from "@/stores/network-store";
import { strip0x } from "./format";

/** 32 zero bytes as hex — the native NORN token ID. */
const NATIVE_TOKEN_ID = "0".repeat(64);

let nextId = 1;

async function rawRpc<T>(method: string, params: unknown[]): Promise<T> {
  const { rpcUrl } = useNetworkStore.getState();

  const response = await fetch(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      method,
      params,
      id: nextId++,
    }),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const json = await response.json();
  if (json.error) {
    throw new Error(`RPC error ${json.error.code}: ${json.error.message}`);
  }

  return json.result as T;
}

/** RPC wrapper that matches the actual Norn node API (strip0x, correct param counts). */
export const rpc = {
  /** Get balance — node requires (address, token_id), returns plain string. */
  async getBalance(address: string, tokenId?: string): Promise<string> {
    const tid = tokenId ? strip0x(tokenId) : NATIVE_TOKEN_ID;
    return rawRpc<string>("norn_getBalance", [strip0x(address), tid]);
  },

  async getTransactionHistory(address: string, limit = 50, offset = 0): Promise<TransactionHistoryEntry[]> {
    return rawRpc<TransactionHistoryEntry[]>("norn_getTransactionHistory", [strip0x(address), limit, offset]);
  },

  async resolveName(name: string): Promise<{ name: string; owner: string } | null> {
    return rawRpc<{ name: string; owner: string } | null>("norn_resolveName", [name]);
  },

  async listTokens(limit = 200, offset = 0): Promise<TokenInfo[]> {
    return rawRpc<TokenInfo[]>("norn_listTokens", [limit, offset]);
  },

  async submitKnot(knotHex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_submitKnot", [knotHex]);
  },

  async getFeeEstimate() {
    return rawRpc<{ base_fee: string; fee_multiplier: number }>("norn_getFeeEstimate", []);
  },

  async registerName(name: string, ownerHex: string, knotHex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_registerName", [name, strip0x(ownerHex), knotHex]);
  },

  async listNames(address: string): Promise<NameInfo[]> {
    return rawRpc<NameInfo[]>("norn_listNames", [strip0x(address)]);
  },

  async faucet(address: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_faucet", [strip0x(address)]);
  },

  async createToken(hex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_createToken", [hex]);
  },

  async mintToken(hex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_mintToken", [hex]);
  },

  async burnToken(hex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_burnToken", [hex]);
  },

  async getTokenInfo(tokenId: string): Promise<TokenInfo | null> {
    return rawRpc<TokenInfo | null>("norn_getTokenInfo", [strip0x(tokenId)]);
  },

  async getTokenBySymbol(symbol: string): Promise<TokenInfo | null> {
    return rawRpc<TokenInfo | null>("norn_getTokenBySymbol", [symbol]);
  },

  async transferName(name: string, ownerHex: string, transferHex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_transferName", [name, strip0x(ownerHex), transferHex]);
  },

  async reverseName(addressHex: string): Promise<string | null> {
    return rawRpc<string | null>("norn_reverseName", [strip0x(addressHex)]);
  },

  async setNameRecord(name: string, key: string, value: string, ownerHex: string, updateHex: string): Promise<SubmitResult> {
    return rawRpc<SubmitResult>("norn_setNameRecord", [name, key, value, strip0x(ownerHex), updateHex]);
  },

  async getNameRecords(name: string): Promise<Record<string, string>> {
    const result = await rawRpc<Record<string, string> | null>("norn_getNameRecords", [name]);
    return result ?? {};
  },
};

export function resetClient(): void {
  nextId = 1;
}
