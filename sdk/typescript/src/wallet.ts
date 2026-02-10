import {
  publicKeyFromPrivate,
  publicKeyToAddress,
  ed25519Sign,
  toHex,
  addressToHex,
} from "./crypto.js";

/** An Ed25519 wallet for signing Norn transactions. */
export class Wallet {
  /** 32-byte private key. */
  readonly privateKey: Uint8Array;
  /** 32-byte public key. */
  readonly publicKey: Uint8Array;
  /** 20-byte address. */
  readonly address: Uint8Array;

  private constructor(privateKey: Uint8Array) {
    if (privateKey.length !== 32) {
      throw new Error(`Private key must be 32 bytes, got ${privateKey.length}`);
    }
    this.privateKey = privateKey;
    this.publicKey = publicKeyFromPrivate(privateKey);
    this.address = publicKeyToAddress(this.publicKey);
  }

  /** Create a wallet from a 32-byte private key. */
  static fromPrivateKey(privateKey: Uint8Array): Wallet {
    return new Wallet(privateKey);
  }

  /** Create a wallet from a hex-encoded private key. */
  static fromPrivateKeyHex(hex: string): Wallet {
    const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
    const bytes = new Uint8Array(clean.length / 2);
    for (let i = 0; i < bytes.length; i++) {
      bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
    }
    return new Wallet(bytes);
  }

  /** Generate a new random wallet. */
  static generate(): Wallet {
    const privateKey = new Uint8Array(32);
    if (typeof globalThis.crypto !== "undefined") {
      globalThis.crypto.getRandomValues(privateKey);
    } else {
      // Fallback for environments without Web Crypto.
      for (let i = 0; i < 32; i++) {
        privateKey[i] = Math.floor(Math.random() * 256);
      }
    }
    return new Wallet(privateKey);
  }

  /** Public key as hex string. */
  get publicKeyHex(): string {
    return toHex(this.publicKey);
  }

  /** Address as hex string with 0x prefix. */
  get addressHex(): string {
    return addressToHex(this.address);
  }

  /** Sign a message. Returns 64-byte signature. */
  sign(message: Uint8Array): Uint8Array {
    return ed25519Sign(message, this.privateKey);
  }
}
