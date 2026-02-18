import { describe, it, expect } from "vitest";
import {
  createChatEvent,
  verifyChatEvent,
  encryptDmContent,
  decryptDmContent,
} from "../src/chat.js";
import { ed25519ToX25519Public } from "../src/encryption.js";
import { publicKeyFromPrivate, toHex } from "../src/crypto.js";

describe("createChatEvent", () => {
  const privateKey = new Uint8Array(32).fill(1);

  it("creates a valid signed event", () => {
    const event = createChatEvent(privateKey, 30003, "hello world", [["c", "general"]]);

    expect(event.id).toHaveLength(64); // 32 bytes as hex
    expect(event.pubkey).toBe(toHex(publicKeyFromPrivate(privateKey)));
    expect(event.kind).toBe(30003);
    expect(event.content).toBe("hello world");
    expect(event.tags).toEqual([["c", "general"]]);
    expect(event.sig).toHaveLength(128); // 64 bytes as hex
    expect(event.created_at).toBeGreaterThan(0);
  });

  it("produces different IDs for different content", () => {
    const e1 = createChatEvent(privateKey, 30003, "hello", []);
    const e2 = createChatEvent(privateKey, 30003, "world", []);
    expect(e1.id).not.toBe(e2.id);
  });
});

describe("verifyChatEvent", () => {
  const privateKey = new Uint8Array(32).fill(1);

  it("verifies a valid event", () => {
    const event = createChatEvent(privateKey, 30003, "test message", []);
    expect(verifyChatEvent(event)).toBe(true);
  });

  it("rejects tampered content", () => {
    const event = createChatEvent(privateKey, 30003, "original", []);
    event.content = "tampered";
    expect(verifyChatEvent(event)).toBe(false);
  });

  it("rejects tampered tags", () => {
    const event = createChatEvent(privateKey, 30003, "test", [["c", "general"]]);
    event.tags = [["c", "other"]];
    expect(verifyChatEvent(event)).toBe(false);
  });

  it("rejects wrong signature", () => {
    const event = createChatEvent(privateKey, 30003, "test", []);
    // Flip a byte in the signature
    const sigBytes = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      sigBytes[i] = parseInt(event.sig.substring(i * 2, i * 2 + 2), 16);
    }
    sigBytes[0] ^= 0xff;
    event.sig = toHex(sigBytes);
    expect(verifyChatEvent(event)).toBe(false);
  });

  it("rejects event from different key", () => {
    const event = createChatEvent(privateKey, 30003, "test", []);
    // Replace pubkey with a different key's pubkey
    const otherKey = new Uint8Array(32).fill(2);
    event.pubkey = toHex(publicKeyFromPrivate(otherKey));
    expect(verifyChatEvent(event)).toBe(false);
  });
});

describe("DM encryption", () => {
  const alicePrivate = new Uint8Array(32).fill(1);
  const bobPrivate = new Uint8Array(32).fill(2);
  const aliceX25519Public = ed25519ToX25519Public(alicePrivate);
  const bobX25519Public = ed25519ToX25519Public(bobPrivate);

  it("encrypt and decrypt DM content", () => {
    const { content, nonceTags } = encryptDmContent(
      alicePrivate,
      bobX25519Public,
      "secret message from alice",
    );

    expect(content).not.toBe("secret message from alice");
    expect(nonceTags).toHaveLength(1);
    expect(nonceTags[0][0]).toBe("nonce");

    const decrypted = decryptDmContent(
      bobPrivate,
      aliceX25519Public,
      content,
      nonceTags[0][1],
    );
    expect(decrypted).toBe("secret message from alice");
  });

  it("sender can also decrypt own DM", () => {
    const { content, nonceTags } = encryptDmContent(
      alicePrivate,
      bobX25519Public,
      "hello bob",
    );

    // Alice can decrypt using Bob's X25519 public key
    const decrypted = decryptDmContent(
      alicePrivate,
      bobX25519Public,
      content,
      nonceTags[0][1],
    );
    expect(decrypted).toBe("hello bob");
  });

  it("third party cannot decrypt DM", () => {
    const { content, nonceTags } = encryptDmContent(
      alicePrivate,
      bobX25519Public,
      "private conversation",
    );

    const evePrivate = new Uint8Array(32).fill(3);
    expect(() =>
      decryptDmContent(evePrivate, aliceX25519Public, content, nonceTags[0][1]),
    ).toThrow();
  });

  it("handles unicode content", () => {
    const { content, nonceTags } = encryptDmContent(
      alicePrivate,
      bobX25519Public,
      "hello world!",
    );

    const decrypted = decryptDmContent(
      bobPrivate,
      aliceX25519Public,
      content,
      nonceTags[0][1],
    );
    expect(decrypted).toBe("hello world!");
  });
});
