"use client";

/**
 * Fallback for browsers without PRF extension support.
 * Uses WebCrypto: PBKDF2-SHA256 + AES-256-GCM to encrypt a random private key.
 */

interface EncryptedKeyBundle {
  /** Base64-encoded encrypted private key */
  encryptedKeyBlob: string;
  /** Base64-encoded PBKDF2 salt */
  pbkdfSalt: string;
  /** Base64-encoded AES-GCM IV */
  aesIv: string;
  /** The raw 32-byte private key (caller must zero after use) */
  privateKey: Uint8Array;
}

function toBase64(buf: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buf)));
}

function fromBase64(b64: string): ArrayBuffer {
  const str = atob(b64);
  const buf = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) buf[i] = str.charCodeAt(i);
  return buf.buffer as ArrayBuffer;
}

async function deriveAesKey(
  password: string,
  salt: Uint8Array
): Promise<CryptoKey> {
  const enc = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    enc.encode(password),
    "PBKDF2",
    false,
    ["deriveKey"]
  );
  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: salt.buffer as ArrayBuffer,
      iterations: 600_000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"]
  );
}

/**
 * Generate a random private key and encrypt it with a password.
 * Returns the encrypted bundle + the raw key (caller MUST zero after use).
 */
export async function createEncryptedKey(
  password: string
): Promise<EncryptedKeyBundle> {
  const privateKey = new Uint8Array(32);
  crypto.getRandomValues(privateKey);

  const salt = new Uint8Array(32);
  crypto.getRandomValues(salt);

  const iv = new Uint8Array(12);
  crypto.getRandomValues(iv);

  const aesKey = await deriveAesKey(password, salt);
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    aesKey,
    privateKey
  );

  return {
    encryptedKeyBlob: toBase64(encrypted),
    pbkdfSalt: toBase64(salt.buffer as ArrayBuffer),
    aesIv: toBase64(iv.buffer as ArrayBuffer),
    privateKey,
  };
}

/**
 * Decrypt a previously encrypted private key.
 * Returns 32-byte private key (caller MUST zero after use).
 */
export async function decryptKey(
  encryptedKeyBlob: string,
  password: string,
  pbkdfSalt: string,
  aesIv: string
): Promise<Uint8Array> {
  const salt = new Uint8Array(fromBase64(pbkdfSalt));
  const iv = new Uint8Array(fromBase64(aesIv));
  const encrypted = fromBase64(encryptedKeyBlob);

  const aesKey = await deriveAesKey(password, salt);

  try {
    const decrypted = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv },
      aesKey,
      encrypted
    );
    return new Uint8Array(decrypted);
  } catch {
    throw new Error("Incorrect password or corrupted key data");
  }
}

/**
 * Encrypt a given private key with a password (for imported keys).
 */
export async function encryptExistingKey(
  privateKey: Uint8Array,
  password: string
): Promise<Omit<EncryptedKeyBundle, "privateKey">> {
  const salt = new Uint8Array(32);
  crypto.getRandomValues(salt);

  const iv = new Uint8Array(12);
  crypto.getRandomValues(iv);

  const aesKey = await deriveAesKey(password, salt);
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    aesKey,
    privateKey.buffer as ArrayBuffer
  );

  return {
    encryptedKeyBlob: toBase64(encrypted),
    pbkdfSalt: toBase64(salt.buffer as ArrayBuffer),
    aesIv: toBase64(iv.buffer as ArrayBuffer),
  };
}
