/** A stored account in IndexedDB. Never contains private keys. */
export interface StoredAccount {
  /** Index used for salt derivation */
  index: number;
  /** Human-readable label */
  label: string;
  /** Hex address with 0x prefix */
  address: string;
  /** Hex-encoded 32-byte public key */
  publicKeyHex: string;
  /** Created timestamp */
  createdAt: number;
}

/** Wallet metadata stored in IndexedDB. */
export interface StoredWalletMeta {
  /** WebAuthn credential ID (base64url) */
  credentialId: string;
  /** All accounts derived from this credential */
  accounts: StoredAccount[];
  /** Whether this wallet uses PRF (true) or password fallback (false) */
  usesPrf: boolean;
  /** If password fallback: encrypted key blob (base64) */
  encryptedKeyBlob?: string;
  /** If password fallback: PBKDF2 salt (base64) */
  pbkdfSalt?: string;
  /** If password fallback: AES-GCM IV (base64) */
  aesIv?: string;
  /** Relying party ID used for passkey */
  rpId: string;
  /** Created timestamp */
  createdAt: number;
}

/** The three possible wallet states. */
export type WalletState = "uninitialized" | "locked" | "unlocked";
