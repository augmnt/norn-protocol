export type {
  AddressHex,
  HashHex,
  PubKeyHex,
  Amount,
  BlockInfo,
  TransactionHistoryEntry,
  TokenInfo,
  FeeEstimateInfo,
  SubmitResult,
  HealthInfo,
  TransferEvent,
  NameInfo,
} from "@norn-protocol/sdk";

export type Route =
  | "welcome"
  | "create-wallet"
  | "import-wallet"
  | "import-cli"
  | "unlock"
  | "dashboard"
  | "send"
  | "confirm"
  | "receive"
  | "activity"
  | "tokens"
  | "create-token"
  | "token-detail"
  | "mint-token"
  | "burn-token"
  | "settings"
  | "accounts"
  | "register-name";

export interface StoredAccount {
  id: string;
  name: string;
  address: string;
  publicKey: string;
  encryptedPrivateKey: EncryptedData;
  createdAt: number;
}

export interface EncryptedData {
  ciphertext: string;
  iv: string;
  salt: string;
}

export interface KeystoreData {
  accounts: StoredAccount[];
  activeAccountId: string | null;
  autoLockMinutes: number;
}

export interface SendParams {
  to: string;
  amount: string;
  memo?: string;
}

export interface NetworkConfig {
  rpcUrl: string;
  wsUrl: string;
  name: string;
}
