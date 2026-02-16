export interface NetworkConfig {
  name: string;
  rpcUrl: string;
  wsUrl: string;
  chainId: string;
  explorerUrl: string;
  isTestnet: boolean;
}

export const NETWORKS: Record<string, NetworkConfig> = {
  devnet: {
    name: "Norn Devnet",
    rpcUrl: "https://seed.norn.network",
    wsUrl: "wss://seed.norn.network",
    chainId: "norn-devnet",
    explorerUrl: "https://explorer.norn.network",
    isTestnet: true,
  },
  testnet: {
    name: "Norn Testnet",
    rpcUrl: "https://testnet.norn.network",
    wsUrl: "wss://testnet.norn.network",
    chainId: "norn-testnet",
    explorerUrl: "https://testnet-explorer.norn.network",
    isTestnet: true,
  },
  mainnet: {
    name: "Norn Mainnet",
    rpcUrl: "https://rpc.norn.network",
    wsUrl: "wss://rpc.norn.network",
    chainId: "norn-mainnet",
    explorerUrl: "https://explorer.norn.network",
    isTestnet: false,
  },
  local: {
    name: "Local Node",
    rpcUrl: "http://localhost:9741",
    wsUrl: "ws://localhost:9741",
    chainId: "norn-local",
    explorerUrl: "http://localhost:3001",
    isTestnet: true,
  },
};

export const DEFAULT_NETWORK = "devnet";
