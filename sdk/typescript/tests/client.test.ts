import { describe, it, expect } from "vitest";
import { NornClient } from "../src/client.js";
import { Wallet } from "../src/wallet.js";

describe("NornClient", () => {
  it("can be constructed", () => {
    const client = new NornClient({ url: "http://localhost:9944" });
    expect(client).toBeDefined();
  });

  it("can be constructed with API key", () => {
    const client = new NornClient({
      url: "http://localhost:9944",
      apiKey: "test-key",
      timeout: 5000,
    });
    expect(client).toBeDefined();
  });
});

describe("Wallet", () => {
  it("creates from private key", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    expect(wallet.publicKey.length).toBe(32);
    expect(wallet.address.length).toBe(20);
    expect(wallet.publicKeyHex.length).toBe(64);
    expect(wallet.addressHex.startsWith("0x")).toBe(true);
  });

  it("creates from hex private key", () => {
    const hexKey = "01".repeat(32);
    const wallet = Wallet.fromPrivateKeyHex(hexKey);
    expect(wallet.publicKey.length).toBe(32);
  });

  it("creates from hex with 0x prefix", () => {
    const hexKey = "0x" + "01".repeat(32);
    const wallet = Wallet.fromPrivateKeyHex(hexKey);
    expect(wallet.publicKey.length).toBe(32);
  });

  it("generates random wallet", () => {
    const w1 = Wallet.generate();
    const w2 = Wallet.generate();
    expect(w1.publicKeyHex).not.toBe(w2.publicKeyHex);
  });

  it("signs messages", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const message = new TextEncoder().encode("hello");
    const sig = wallet.sign(message);
    expect(sig.length).toBe(64);
  });

  it("deterministic key derivation", () => {
    const key = new Uint8Array(32).fill(42);
    const w1 = Wallet.fromPrivateKey(key);
    const w2 = Wallet.fromPrivateKey(key);
    expect(w1.publicKeyHex).toBe(w2.publicKeyHex);
    expect(w1.addressHex).toBe(w2.addressHex);
  });

  it("rejects wrong key length", () => {
    expect(() => Wallet.fromPrivateKey(new Uint8Array(16))).toThrow(
      "32 bytes",
    );
  });
});
