export const config = {
  rpcUrl: process.env.NEXT_PUBLIC_RPC_URL || "https://seed.norn.network",
  wsUrl: process.env.NEXT_PUBLIC_WS_URL || "wss://seed.norn.network",
  chainName: process.env.NEXT_PUBLIC_CHAIN_NAME || "Norn Devnet",
  nornDecimals: 12,
  // Native NORN token ID — 32 zero bytes, NO 0x prefix (RPC expects raw hex)
  nativeTokenId: "0".repeat(64),
} as const;
