import { describe, it, expect } from "vitest";
import {
  getBit,
  hashLeaf,
  hashInternal,
  smtKey,
  encodeU128LE,
  verifyStateProof,
} from "../src/merkle.js";
import { blake3Hash, fromHex, toHex } from "../src/crypto.js";

describe("getBit", () => {
  it("extracts MSB-first bits correctly", () => {
    // 0x80 = 10000000 in binary
    const key = new Uint8Array(32);
    key[0] = 0x80;
    expect(getBit(key, 0)).toBe(1);
    expect(getBit(key, 1)).toBe(0);
    expect(getBit(key, 7)).toBe(0);
  });

  it("extracts bits from different byte positions", () => {
    const key = new Uint8Array(32);
    key[1] = 0x01; // bit at position 15 (byte 1, bit 0)
    expect(getBit(key, 8)).toBe(0);
    expect(getBit(key, 15)).toBe(1);
  });

  it("returns 0 for out-of-range depth", () => {
    const key = new Uint8Array(32).fill(0xff);
    expect(getBit(key, 256)).toBe(0);
    expect(getBit(key, 300)).toBe(0);
  });

  it("extracts all 8 bits from 0xFF byte", () => {
    const key = new Uint8Array(32);
    key[0] = 0xff;
    for (let i = 0; i < 8; i++) {
      expect(getBit(key, i)).toBe(1);
    }
  });
});

describe("hashLeaf", () => {
  it("produces a 32-byte hash", () => {
    const key = new Uint8Array(32).fill(0xaa);
    const valueHash = new Uint8Array(32).fill(0xbb);
    const result = hashLeaf(key, valueHash);
    expect(result.length).toBe(32);
  });

  it("includes the 0x00 domain separator", () => {
    const key = new Uint8Array(32);
    const valueHash = new Uint8Array(32);
    const result = hashLeaf(key, valueHash);
    // Should be BLAKE3(0x00 || zeros || zeros) — not the same as BLAKE3(zeros)
    const rawHash = blake3Hash(new Uint8Array(65));
    // With domain separator 0x00, should still match since first byte is 0
    expect(toHex(result)).toBe(toHex(rawHash));
  });

  it("different keys produce different hashes", () => {
    const valueHash = new Uint8Array(32).fill(0xcc);
    const key1 = new Uint8Array(32).fill(0x01);
    const key2 = new Uint8Array(32).fill(0x02);
    const h1 = hashLeaf(key1, valueHash);
    const h2 = hashLeaf(key2, valueHash);
    expect(toHex(h1)).not.toBe(toHex(h2));
  });
});

describe("hashInternal", () => {
  it("produces a 32-byte hash", () => {
    const left = new Uint8Array(32).fill(0xaa);
    const right = new Uint8Array(32).fill(0xbb);
    const result = hashInternal(left, right);
    expect(result.length).toBe(32);
  });

  it("is order-sensitive", () => {
    const a = new Uint8Array(32).fill(0x01);
    const b = new Uint8Array(32).fill(0x02);
    const h1 = hashInternal(a, b);
    const h2 = hashInternal(b, a);
    expect(toHex(h1)).not.toBe(toHex(h2));
  });

  it("uses 0x01 domain separator (different from leaf)", () => {
    const a = new Uint8Array(32).fill(0xaa);
    const b = new Uint8Array(32).fill(0xbb);
    const leaf = hashLeaf(a, b);
    const internal = hashInternal(a, b);
    // Leaf uses 0x00, internal uses 0x01 — must be different
    expect(toHex(leaf)).not.toBe(toHex(internal));
  });
});

describe("smtKey", () => {
  it("produces a 32-byte hash from address + tokenId", () => {
    const address = new Uint8Array(20).fill(0x01);
    const tokenId = new Uint8Array(32).fill(0x02);
    const key = smtKey(address, tokenId);
    expect(key.length).toBe(32);
  });

  it("is deterministic", () => {
    const address = new Uint8Array(20).fill(0xaa);
    const tokenId = new Uint8Array(32).fill(0xbb);
    const k1 = smtKey(address, tokenId);
    const k2 = smtKey(address, tokenId);
    expect(toHex(k1)).toBe(toHex(k2));
  });

  it("different addresses produce different keys", () => {
    const tokenId = new Uint8Array(32);
    const a1 = new Uint8Array(20).fill(0x01);
    const a2 = new Uint8Array(20).fill(0x02);
    const k1 = smtKey(a1, tokenId);
    const k2 = smtKey(a2, tokenId);
    expect(toHex(k1)).not.toBe(toHex(k2));
  });

  it("matches BLAKE3(address ++ tokenId)", () => {
    const address = fromHex("01".repeat(20));
    const tokenId = fromHex("02".repeat(32));
    const concat = new Uint8Array(52);
    concat.set(address, 0);
    concat.set(tokenId, 20);
    const expected = blake3Hash(concat);
    const result = smtKey(address, tokenId);
    expect(toHex(result)).toBe(toHex(expected));
  });
});

describe("encodeU128LE", () => {
  it("encodes zero as 16 zero bytes", () => {
    const result = encodeU128LE(0n);
    expect(result.length).toBe(16);
    expect(toHex(result)).toBe("00".repeat(16));
  });

  it("encodes 1 in little-endian", () => {
    const result = encodeU128LE(1n);
    expect(result[0]).toBe(1);
    for (let i = 1; i < 16; i++) {
      expect(result[i]).toBe(0);
    }
  });

  it("encodes 256 correctly", () => {
    const result = encodeU128LE(256n);
    expect(result[0]).toBe(0);
    expect(result[1]).toBe(1);
  });

  it("encodes large amounts", () => {
    // 1 NORN = 10^12
    const result = encodeU128LE(1000000000000n);
    expect(result.length).toBe(16);
    // Verify round-trip
    let decoded = 0n;
    for (let i = 15; i >= 0; i--) {
      decoded = (decoded << 8n) | BigInt(result[i]);
    }
    expect(decoded).toBe(1000000000000n);
  });
});

describe("verifyStateProof", () => {
  it("verifies empty tree (zero root, empty value, zero siblings)", () => {
    const root = new Uint8Array(32); // empty root = all zeros
    const key = blake3Hash(new Uint8Array(32).fill(0x01));
    const value = new Uint8Array(0); // empty = non-inclusion
    const siblings: Uint8Array[] = [];
    for (let i = 0; i < 256; i++) {
      siblings.push(new Uint8Array(32));
    }
    expect(verifyStateProof(root, key, value, siblings)).toBe(true);
  });

  it("rejects proof with wrong root", () => {
    const root = new Uint8Array(32).fill(0xff); // wrong root
    const key = blake3Hash(new Uint8Array(32).fill(0x01));
    const value = new Uint8Array(0);
    const siblings: Uint8Array[] = [];
    for (let i = 0; i < 256; i++) {
      siblings.push(new Uint8Array(32));
    }
    // Non-inclusion with all-zero siblings computes to EMPTY_HASH root,
    // so a non-zero root should fail.
    expect(verifyStateProof(root, key, value, siblings)).toBe(false);
  });

  it("rejects proof with wrong number of siblings", () => {
    const root = new Uint8Array(32);
    const key = new Uint8Array(32);
    const value = new Uint8Array(0);
    const siblings = [new Uint8Array(32)]; // only 1, need 256
    expect(verifyStateProof(root, key, value, siblings)).toBe(false);
  });

  it("verifies a single-entry tree", () => {
    // Manually compute the root for a tree with one entry.
    const key = blake3Hash(new Uint8Array([1, 2, 3]));
    const value = encodeU128LE(42n);
    const valueHash = blake3Hash(value);
    let current = hashLeaf(key, valueHash);

    // Build from depth 255 up to 0, siblings are all empty.
    const siblings: Uint8Array[] = new Array(256);
    for (let i = 0; i < 256; i++) {
      siblings[i] = new Uint8Array(32);
    }

    // Compute root going up
    for (let depth = 255; depth >= 0; depth--) {
      const bit = getBit(key, depth);
      const sibling = new Uint8Array(32); // empty
      if (bit === 0) {
        current = hashInternal(current, sibling);
      } else {
        current = hashInternal(sibling, current);
      }
    }

    const root = current;
    expect(verifyStateProof(root, key, value, siblings)).toBe(true);
  });
});
