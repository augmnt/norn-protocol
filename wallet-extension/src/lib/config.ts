import type { NetworkConfig } from "@/types";

export const DEFAULT_NETWORK: NetworkConfig = {
  rpcUrl: "https://seed.norn.network",
  wsUrl: "wss://seed.norn.network",
  name: "Devnet",
};

export const NETWORK_PRESETS: NetworkConfig[] = [
  { rpcUrl: "https://seed.norn.network", wsUrl: "wss://seed.norn.network", name: "Devnet" },
  { rpcUrl: "https://testnet.norn.network", wsUrl: "wss://testnet.norn.network", name: "Testnet" },
  { rpcUrl: "http://localhost:9741", wsUrl: "ws://localhost:9741", name: "Local" },
];

export const NORN_DECIMALS = 12;

export const AUTO_LOCK_DEFAULT_MINUTES = 15;

export const BALANCE_POLL_INTERVAL = 10_000;

export const KEYSTORE_STORAGE_KEY = "norn_keystore";
export const NETWORK_STORAGE_KEY = "norn_network";
export const LOCKED_STORAGE_KEY = "norn_locked";
