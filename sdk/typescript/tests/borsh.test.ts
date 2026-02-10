import { describe, it, expect } from "vitest";
import { BorshWriter, BorshReader } from "../src/borsh.js";

describe("BorshWriter and BorshReader", () => {
  it("writes and reads u8", () => {
    const w = new BorshWriter();
    w.writeU8(42);
    w.writeU8(255);
    const r = new BorshReader(w.toBytes());
    expect(r.readU8()).toBe(42);
    expect(r.readU8()).toBe(255);
  });

  it("writes and reads u32 little-endian", () => {
    const w = new BorshWriter();
    w.writeU32(0x12345678);
    const bytes = w.toBytes();
    // Little-endian: least significant byte first.
    expect(bytes[0]).toBe(0x78);
    expect(bytes[1]).toBe(0x56);
    expect(bytes[2]).toBe(0x34);
    expect(bytes[3]).toBe(0x12);

    const r = new BorshReader(bytes);
    expect(r.readU32()).toBe(0x12345678);
  });

  it("writes and reads u64", () => {
    const w = new BorshWriter();
    w.writeU64(1234567890123456789n);
    const r = new BorshReader(w.toBytes());
    expect(r.readU64()).toBe(1234567890123456789n);
  });

  it("writes and reads u128", () => {
    const w = new BorshWriter();
    const value = 340282366920938463463374607431768211455n; // max u128
    w.writeU128(value);
    const r = new BorshReader(w.toBytes());
    expect(r.readU128()).toBe(value);
  });

  it("writes and reads u128 zero", () => {
    const w = new BorshWriter();
    w.writeU128(0n);
    const r = new BorshReader(w.toBytes());
    expect(r.readU128()).toBe(0n);
  });

  it("writes and reads fixed bytes", () => {
    const data = new Uint8Array([1, 2, 3, 4, 5]);
    const w = new BorshWriter();
    w.writeFixedBytes(data);
    const r = new BorshReader(w.toBytes());
    expect(r.readFixedBytes(5)).toEqual(data);
  });

  it("writes and reads length-prefixed bytes", () => {
    const data = new Uint8Array([10, 20, 30]);
    const w = new BorshWriter();
    w.writeBytes(data);
    const bytes = w.toBytes();
    // First 4 bytes are u32 length (3), then data.
    expect(bytes.length).toBe(4 + 3);
    const r = new BorshReader(bytes);
    expect(r.readBytes()).toEqual(data);
  });

  it("writes and reads string", () => {
    const w = new BorshWriter();
    w.writeString("hello");
    const r = new BorshReader(w.toBytes());
    expect(r.readString()).toBe("hello");
  });

  it("writes and reads empty string", () => {
    const w = new BorshWriter();
    w.writeString("");
    const r = new BorshReader(w.toBytes());
    expect(r.readString()).toBe("");
  });

  it("writes and reads option (Some)", () => {
    const data = new Uint8Array([1, 2, 3]);
    const w = new BorshWriter();
    w.writeOptionBytes(data);
    const r = new BorshReader(w.toBytes());
    const flag = r.readU8();
    expect(flag).toBe(1);
    const value = r.readBytes();
    expect(value).toEqual(data);
  });

  it("writes and reads option (None)", () => {
    const w = new BorshWriter();
    w.writeOptionBytes(null);
    const r = new BorshReader(w.toBytes());
    const flag = r.readU8();
    expect(flag).toBe(0);
  });

  it("complex structure round-trip", () => {
    const w = new BorshWriter();
    w.writeFixedBytes(new Uint8Array(20).fill(1)); // address
    w.writeFixedBytes(new Uint8Array(20).fill(2)); // address
    w.writeFixedBytes(new Uint8Array(32).fill(0)); // token ID
    w.writeU128(1000000000000n); // 1 NORN
    w.writeU64(1700000000n); // timestamp
    w.writeString("memo");
    w.writeFixedBytes(new Uint8Array(32).fill(3)); // pubkey
    w.writeFixedBytes(new Uint8Array(64).fill(4)); // signature

    const bytes = w.toBytes();
    const r = new BorshReader(bytes);

    const from = r.readFixedBytes(20);
    expect(from).toEqual(new Uint8Array(20).fill(1));

    const to = r.readFixedBytes(20);
    expect(to).toEqual(new Uint8Array(20).fill(2));

    const tokenId = r.readFixedBytes(32);
    expect(tokenId).toEqual(new Uint8Array(32).fill(0));

    const amount = r.readU128();
    expect(amount).toBe(1000000000000n);

    const timestamp = r.readU64();
    expect(timestamp).toBe(1700000000n);

    const memo = r.readString();
    expect(memo).toBe("memo");

    const pubkey = r.readFixedBytes(32);
    expect(pubkey).toEqual(new Uint8Array(32).fill(3));

    const sig = r.readFixedBytes(64);
    expect(sig).toEqual(new Uint8Array(64).fill(4));

    expect(r.remaining()).toBe(0);
  });
});
