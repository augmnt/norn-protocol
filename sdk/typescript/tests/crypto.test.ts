import { describe, it, expect } from "vitest";
import {
  blake3Hash,
  ed25519Sign,
  ed25519Verify,
  publicKeyFromPrivate,
  publicKeyToAddress,
  toHex,
  fromHex,
  addressToHex,
  hexToAddress,
} from "../src/crypto.js";

describe("blake3Hash", () => {
  it("produces a 32-byte hash", () => {
    const hash = blake3Hash(new Uint8Array([1, 2, 3]));
    expect(hash.length).toBe(32);
  });

  it("is deterministic", () => {
    const data = new TextEncoder().encode("hello norn");
    const h1 = blake3Hash(data);
    const h2 = blake3Hash(data);
    expect(toHex(h1)).toBe(toHex(h2));
  });

  it("different inputs produce different hashes", () => {
    const h1 = blake3Hash(new Uint8Array([1]));
    const h2 = blake3Hash(new Uint8Array([2]));
    expect(toHex(h1)).not.toBe(toHex(h2));
  });

  it("empty input is valid", () => {
    const hash = blake3Hash(new Uint8Array(0));
    expect(hash.length).toBe(32);
  });
});

describe("Ed25519", () => {
  // A fixed test key (32 bytes of 0x01).
  const privateKey = new Uint8Array(32).fill(1);

  it("sign and verify round-trip", () => {
    const message = new TextEncoder().encode("test message");
    const publicKey = publicKeyFromPrivate(privateKey);
    const signature = ed25519Sign(message, privateKey);

    expect(signature.length).toBe(64);
    expect(ed25519Verify(signature, message, publicKey)).toBe(true);
  });

  it("rejects wrong message", () => {
    const publicKey = publicKeyFromPrivate(privateKey);
    const signature = ed25519Sign(new Uint8Array([1, 2, 3]), privateKey);
    expect(ed25519Verify(signature, new Uint8Array([4, 5, 6]), publicKey)).toBe(
      false,
    );
  });

  it("rejects wrong key", () => {
    const message = new TextEncoder().encode("test");
    const signature = ed25519Sign(message, privateKey);
    const wrongKey = publicKeyFromPrivate(new Uint8Array(32).fill(2));
    expect(ed25519Verify(signature, message, wrongKey)).toBe(false);
  });

  it("derives deterministic public key", () => {
    const pk1 = publicKeyFromPrivate(privateKey);
    const pk2 = publicKeyFromPrivate(privateKey);
    expect(toHex(pk1)).toBe(toHex(pk2));
    expect(pk1.length).toBe(32);
  });
});

describe("publicKeyToAddress", () => {
  it("produces a 20-byte address", () => {
    const publicKey = publicKeyFromPrivate(new Uint8Array(32).fill(1));
    const address = publicKeyToAddress(publicKey);
    expect(address.length).toBe(20);
  });

  it("is deterministic", () => {
    const publicKey = publicKeyFromPrivate(new Uint8Array(32).fill(1));
    const a1 = publicKeyToAddress(publicKey);
    const a2 = publicKeyToAddress(publicKey);
    expect(toHex(a1)).toBe(toHex(a2));
  });

  it("different keys produce different addresses", () => {
    const pk1 = publicKeyFromPrivate(new Uint8Array(32).fill(1));
    const pk2 = publicKeyFromPrivate(new Uint8Array(32).fill(2));
    const a1 = publicKeyToAddress(pk1);
    const a2 = publicKeyToAddress(pk2);
    expect(toHex(a1)).not.toBe(toHex(a2));
  });
});

describe("hex utilities", () => {
  it("toHex and fromHex round-trip", () => {
    const bytes = new Uint8Array([0xde, 0xad, 0xbe, 0xef]);
    const hex = toHex(bytes);
    expect(hex).toBe("deadbeef");
    expect(fromHex(hex)).toEqual(bytes);
  });

  it("fromHex strips 0x prefix", () => {
    const bytes = fromHex("0xdeadbeef");
    expect(toHex(bytes)).toBe("deadbeef");
  });

  it("addressToHex adds 0x prefix", () => {
    const addr = new Uint8Array(20).fill(0xab);
    const hex = addressToHex(addr);
    expect(hex.startsWith("0x")).toBe(true);
    expect(hex.length).toBe(42); // "0x" + 40 hex chars
  });

  it("hexToAddress validates length", () => {
    expect(() => hexToAddress("0xdeadbeef")).toThrow("Expected 20 bytes");
    expect(() => hexToAddress("0x" + "ab".repeat(20))).not.toThrow();
  });

  it("fromHex rejects odd-length", () => {
    expect(() => fromHex("abc")).toThrow("odd length");
  });
});
