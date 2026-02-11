export const QUERY_KEYS = {
  health: ["health"] as const,
  weaveState: ["weaveState"] as const,
  validatorSet: ["validatorSet"] as const,
  stakingInfo: ["stakingInfo"] as const,
  block: (height: number) => ["block", height] as const,
  blockTransactions: (height: number) => ["blockTransactions", height] as const,
  blocks: (page: number) => ["blocks", page] as const,
  balance: (address: string, tokenId?: string) =>
    ["balance", address, tokenId] as const,
  threadState: (address: string) => ["threadState", address] as const,
  txHistory: (address: string, page: number) =>
    ["txHistory", address, page] as const,
  names: (address: string) => ["names", address] as const,
  resolveName: (name: string) => ["resolveName", name] as const,
  tokenInfo: (tokenId: string) => ["tokenInfo", tokenId] as const,
  tokenBySymbol: (symbol: string) => ["tokenBySymbol", symbol] as const,
  tokensList: (page: number) => ["tokensList", page] as const,
  loomInfo: (loomId: string) => ["loomInfo", loomId] as const,
  loomsList: (page: number) => ["loomsList", page] as const,
  transaction: (knotId: string) => ["transaction", knotId] as const,
  feeEstimate: ["feeEstimate"] as const,
  nodeInfo: ["nodeInfo"] as const,
} as const;

export const STALE_TIMES = {
  immutable: Infinity,
  semiStatic: 30_000,
  dynamic: 15_000,
  realtime: 5_000,
} as const;

export const PAGE_SIZE = 20;

export const NATIVE_TOKEN_ID = "0".repeat(64);

export const WS_CAPS = {
  blocks: 50,
  transfers: 200,
  pendingTxs: 100,
  tokenEvents: 100,
  loomEvents: 100,
} as const;
