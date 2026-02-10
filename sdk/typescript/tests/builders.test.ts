import { describe, it, expect } from "vitest";
import { Wallet } from "../src/wallet.js";
import {
  buildTransfer,
  buildNameRegistration,
  buildTokenDefinition,
  buildTokenMint,
  buildTokenBurn,
  parseAmount,
  formatAmount,
} from "../src/builders.js";
import { fromHex, toHex, ed25519Verify, publicKeyFromPrivate } from "../src/crypto.js";
import { BorshReader } from "../src/borsh.js";

describe("parseAmount", () => {
  it("parses whole numbers", () => {
    expect(parseAmount("1")).toBe(1000000000000n);
    expect(parseAmount("100")).toBe(100000000000000n);
  });

  it("parses decimals", () => {
    expect(parseAmount("1.5")).toBe(1500000000000n);
    expect(parseAmount("0.000000000001")).toBe(1n);
  });

  it("parses zero", () => {
    expect(parseAmount("0")).toBe(0n);
    expect(parseAmount("0.0")).toBe(0n);
  });

  it("truncates excess decimals", () => {
    // 13 decimal places -> truncated to 12.
    expect(parseAmount("1.0000000000001")).toBe(1000000000000n);
  });
});

describe("formatAmount", () => {
  it("formats whole numbers", () => {
    expect(formatAmount(1000000000000n)).toBe("1.0");
    expect(formatAmount(100000000000000n)).toBe("100.0");
  });

  it("formats fractional amounts", () => {
    expect(formatAmount(1500000000000n)).toBe("1.5");
  });

  it("formats zero", () => {
    expect(formatAmount(0n)).toBe("0.0");
  });

  it("round-trip with parseAmount", () => {
    const original = "42.123456";
    const parsed = parseAmount(original);
    const formatted = formatAmount(parsed);
    expect(formatted).toBe("42.123456");
  });
});

describe("buildTransfer", () => {
  it("produces a non-empty hex string", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildTransfer(wallet, {
      to: "0x" + "02".repeat(20),
      amount: 1000000000000n,
    });
    expect(hex.length).toBeGreaterThan(0);
    // Should be valid hex.
    expect(() => fromHex(hex)).not.toThrow();
  });

  it("contains a valid Knot structure with sender pubkey in before_states", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildTransfer(wallet, {
      to: "0x" + "02".repeat(20),
      amount: 1000000000000n,
    });
    const bytes = fromHex(hex);
    const r = new BorshReader(bytes);

    // id: [u8; 32]
    const knotId = r.readFixedBytes(32);
    expect(knotId.length).toBe(32);

    // knot_type: KnotType::Transfer = 0
    expect(r.readU8()).toBe(0);

    // timestamp: u64
    const ts = r.readU64();
    expect(ts).toBeGreaterThan(0n);

    // expiry: Option<u64> = None
    expect(r.readU8()).toBe(0);

    // before_states: Vec<ParticipantState> length = 1
    expect(r.readU32()).toBe(1);
    // before_states[0].thread_id = sender address
    const threadId = r.readFixedBytes(20);
    expect(toHex(threadId)).toBe(toHex(wallet.address));
    // before_states[0].pubkey = sender public key
    const pubkey = r.readFixedBytes(32);
    expect(toHex(pubkey)).toBe(toHex(wallet.publicKey));
  });
});

describe("buildNameRegistration", () => {
  it("produces valid hex", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildNameRegistration(wallet, "alice");
    expect(hex.length).toBeGreaterThan(0);
    expect(() => fromHex(hex)).not.toThrow();
  });
});

describe("buildTokenDefinition", () => {
  it("produces valid hex", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildTokenDefinition(wallet, {
      name: "Test Token",
      symbol: "TEST",
      decimals: 8,
      maxSupply: 1000000n * 10n ** 8n,
    });
    expect(hex.length).toBeGreaterThan(0);
    expect(() => fromHex(hex)).not.toThrow();
  });
});

describe("buildTokenMint", () => {
  it("produces valid hex", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildTokenMint(wallet, {
      tokenId: "ab".repeat(32),
      to: "0x" + "02".repeat(20),
      amount: 1000n,
    });
    expect(hex.length).toBeGreaterThan(0);
    expect(() => fromHex(hex)).not.toThrow();
  });
});

describe("buildTokenBurn", () => {
  it("produces valid hex", () => {
    const wallet = Wallet.fromPrivateKey(new Uint8Array(32).fill(1));
    const hex = buildTokenBurn(wallet, {
      tokenId: "ab".repeat(32),
      amount: 500n,
    });
    expect(hex.length).toBeGreaterThan(0);
    expect(() => fromHex(hex)).not.toThrow();
  });
});
