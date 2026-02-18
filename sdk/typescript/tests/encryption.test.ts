import { describe, it, expect } from "vitest";
import {
  ed25519ToX25519Secret,
  ed25519ToX25519Public,
  encrypt,
  decrypt,
  deriveSharedSecret,
  symmetricEncrypt,
  symmetricDecrypt,
} from "../src/encryption.js";
import { toHex } from "../src/crypto.js";

describe("X25519 key derivation", () => {
  const privateKey = new Uint8Array(32).fill(1);

  it("derives deterministic X25519 secret", () => {
    const s1 = ed25519ToX25519Secret(privateKey);
    const s2 = ed25519ToX25519Secret(privateKey);
    expect(toHex(s1)).toBe(toHex(s2));
    expect(s1.length).toBe(32);
  });

  it("derives deterministic X25519 public key", () => {
    const p1 = ed25519ToX25519Public(privateKey);
    const p2 = ed25519ToX25519Public(privateKey);
    expect(toHex(p1)).toBe(toHex(p2));
    expect(p1.length).toBe(32);
  });

  it("different Ed25519 keys produce different X25519 keys", () => {
    const k1 = ed25519ToX25519Public(new Uint8Array(32).fill(1));
    const k2 = ed25519ToX25519Public(new Uint8Array(32).fill(2));
    expect(toHex(k1)).not.toBe(toHex(k2));
  });
});

describe("asymmetric encrypt/decrypt", () => {
  const recipientPrivateKey = new Uint8Array(32).fill(1);
  const recipientX25519Public = ed25519ToX25519Public(recipientPrivateKey);

  it("roundtrip encrypt/decrypt", () => {
    const plaintext = new TextEncoder().encode("hello, encrypted norn world!");
    const encrypted = encrypt(recipientX25519Public, plaintext);
    const decrypted = decrypt(
      recipientPrivateKey,
      encrypted.ephemeralPubkey,
      encrypted.nonce,
      encrypted.ciphertext,
    );
    expect(new TextDecoder().decode(decrypted)).toBe("hello, encrypted norn world!");
  });

  it("wrong key fails to decrypt", () => {
    const plaintext = new TextEncoder().encode("secret message");
    const encrypted = encrypt(recipientX25519Public, plaintext);
    const wrongKey = new Uint8Array(32).fill(2);
    expect(() =>
      decrypt(wrongKey, encrypted.ephemeralPubkey, encrypted.nonce, encrypted.ciphertext),
    ).toThrow();
  });

  it("tampered ciphertext fails", () => {
    const plaintext = new TextEncoder().encode("secret message");
    const encrypted = encrypt(recipientX25519Public, plaintext);
    const tampered = new Uint8Array(encrypted.ciphertext);
    tampered[0] ^= 0xff;
    expect(() =>
      decrypt(recipientPrivateKey, encrypted.ephemeralPubkey, encrypted.nonce, tampered),
    ).toThrow();
  });

  it("empty plaintext roundtrip", () => {
    const plaintext = new Uint8Array(0);
    const encrypted = encrypt(recipientX25519Public, plaintext);
    const decrypted = decrypt(
      recipientPrivateKey,
      encrypted.ephemeralPubkey,
      encrypted.nonce,
      encrypted.ciphertext,
    );
    expect(decrypted.length).toBe(0);
  });

  it("large plaintext roundtrip", () => {
    const plaintext = new Uint8Array(65536).fill(0xab);
    const encrypted = encrypt(recipientX25519Public, plaintext);
    const decrypted = decrypt(
      recipientPrivateKey,
      encrypted.ephemeralPubkey,
      encrypted.nonce,
      encrypted.ciphertext,
    );
    expect(toHex(decrypted)).toBe(toHex(plaintext));
  });
});

describe("DM shared secret", () => {
  it("both parties derive the same shared secret", () => {
    const alicePrivate = new Uint8Array(32).fill(1);
    const bobPrivate = new Uint8Array(32).fill(2);
    const aliceX25519Public = ed25519ToX25519Public(alicePrivate);
    const bobX25519Public = ed25519ToX25519Public(bobPrivate);

    const aliceShared = deriveSharedSecret(alicePrivate, bobX25519Public);
    const bobShared = deriveSharedSecret(bobPrivate, aliceX25519Public);

    expect(toHex(aliceShared)).toBe(toHex(bobShared));
    expect(aliceShared.length).toBe(32);
  });

  it("different pairs produce different shared secrets", () => {
    const key1 = new Uint8Array(32).fill(1);
    const key2 = new Uint8Array(32).fill(2);
    const key3 = new Uint8Array(32).fill(3);

    const shared12 = deriveSharedSecret(key1, ed25519ToX25519Public(key2));
    const shared13 = deriveSharedSecret(key1, ed25519ToX25519Public(key3));

    expect(toHex(shared12)).not.toBe(toHex(shared13));
  });
});

describe("symmetric encrypt/decrypt", () => {
  const key = new Uint8Array(32).fill(0xaa);

  it("roundtrip encrypt/decrypt", () => {
    const plaintext = new TextEncoder().encode("symmetric test");
    const encrypted = symmetricEncrypt(key, plaintext);
    const decrypted = symmetricDecrypt(key, encrypted.nonce, encrypted.ciphertext);
    expect(new TextDecoder().decode(decrypted)).toBe("symmetric test");
  });

  it("wrong key fails", () => {
    const plaintext = new TextEncoder().encode("secret");
    const encrypted = symmetricEncrypt(key, plaintext);
    const wrongKey = new Uint8Array(32).fill(0xbb);
    expect(() => symmetricDecrypt(wrongKey, encrypted.nonce, encrypted.ciphertext)).toThrow();
  });

  it("tampered ciphertext fails", () => {
    const plaintext = new TextEncoder().encode("secret");
    const encrypted = symmetricEncrypt(key, plaintext);
    const tampered = new Uint8Array(encrypted.ciphertext);
    tampered[0] ^= 0xff;
    expect(() => symmetricDecrypt(key, encrypted.nonce, tampered)).toThrow();
  });
});
