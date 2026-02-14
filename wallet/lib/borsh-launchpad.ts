/**
 * Borsh encoding/decoding for the Launchpad contract messages.
 *
 * Borsh format for enums: 1-byte discriminant + field data.
 * Borsh strings: 4-byte LE length + UTF-8 bytes.
 * Borsh u64: 8 bytes LE.
 * Borsh u128: 16 bytes LE.
 * Borsh Address: 20 raw bytes.
 * Borsh TokenId: 32 raw bytes.
 * Borsh bool: 1 byte (0=false, 1=true).
 */

// ── Helpers ────────────────────────────────────────────────────────────

function encodeU64(n: bigint): Uint8Array {
  const buf = new Uint8Array(8);
  const view = new DataView(buf.buffer);
  view.setBigUint64(0, n, true);
  return buf;
}

function encodeU128(n: bigint): Uint8Array {
  const buf = new Uint8Array(16);
  const view = new DataView(buf.buffer);
  view.setBigUint64(0, n & 0xffffffffffffffffn, true);
  view.setBigUint64(8, n >> 64n, true);
  return buf;
}

function hexToBytes(hex: string): Uint8Array {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex;
  const bytes = new Uint8Array(h.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(h.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function concat(...arrays: Uint8Array[]): Uint8Array {
  const total = arrays.reduce((sum, a) => sum + a.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const a of arrays) {
    result.set(a, offset);
    offset += a.length;
  }
  return result;
}

// ── Execute message encoders ──────────────────────────────────────────

// Discriminants: Initialize=0, Contribute=1, ClaimTokens=2, Finalize=3, Refund=4

export function encodeInitialize(
  tokenId: string,
  price: bigint,
  hardCap: bigint,
  maxPerWallet: bigint,
  startTime: bigint,
  endTime: bigint,
  totalTokens: bigint
): string {
  const parts = concat(
    new Uint8Array([0]),
    hexToBytes(tokenId),
    encodeU128(price),
    encodeU128(hardCap),
    encodeU128(maxPerWallet),
    encodeU64(startTime),
    encodeU64(endTime),
    encodeU128(totalTokens)
  );
  return bytesToHex(parts);
}

export function encodeContribute(amount: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU128(amount)));
}

export function encodeClaimTokens(): string {
  return bytesToHex(new Uint8Array([2]));
}

export function encodeFinalize(): string {
  return bytesToHex(new Uint8Array([3]));
}

export function encodeRefund(): string {
  return bytesToHex(new Uint8Array([4]));
}

// ── Query message encoders ──────────────────────────────────────────

// GetConfig=0, GetContribution=1, GetTotalRaised=2

export function encodeGetConfig(): string {
  return bytesToHex(new Uint8Array([0]));
}

export function encodeGetContribution(addr: string): string {
  return bytesToHex(concat(new Uint8Array([1]), hexToBytes(addr)));
}

export function encodeGetTotalRaised(): string {
  return bytesToHex(new Uint8Array([2]));
}

// ── Response decoders ─────────────────────────────────────────────────

export interface LaunchConfig {
  creator: string;
  tokenId: string;
  price: bigint;
  hardCap: bigint;
  maxPerWallet: bigint;
  startTime: bigint;
  endTime: bigint;
  totalTokens: bigint;
  finalized: boolean;
}

function readU64(data: Uint8Array, offset: number): [bigint, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 8);
  return [view.getBigUint64(0, true), offset + 8];
}

function readU128(data: Uint8Array, offset: number): [bigint, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 16);
  const lo = view.getBigUint64(0, true);
  const hi = view.getBigUint64(8, true);
  return [(hi << 64n) | lo, offset + 16];
}

function readAddress(data: Uint8Array, offset: number): [string, number] {
  const bytes = data.slice(offset, offset + 20);
  return ["0x" + bytesToHex(bytes), offset + 20];
}

function readTokenId(data: Uint8Array, offset: number): [string, number] {
  const bytes = data.slice(offset, offset + 32);
  return [bytesToHex(bytes), offset + 32];
}

function readBool(data: Uint8Array, offset: number): [boolean, number] {
  return [data[offset] !== 0, offset + 1];
}

export function decodeLaunchConfig(hex: string): LaunchConfig {
  const data = hexToBytes(hex);
  let offset = 0;

  let creator: string;
  [creator, offset] = readAddress(data, offset);
  let tokenId: string;
  [tokenId, offset] = readTokenId(data, offset);
  let price: bigint;
  [price, offset] = readU128(data, offset);
  let hardCap: bigint;
  [hardCap, offset] = readU128(data, offset);
  let maxPerWallet: bigint;
  [maxPerWallet, offset] = readU128(data, offset);
  let startTime: bigint;
  [startTime, offset] = readU64(data, offset);
  let endTime: bigint;
  [endTime, offset] = readU64(data, offset);
  let totalTokens: bigint;
  [totalTokens, offset] = readU128(data, offset);
  let finalized: boolean;
  [finalized, offset] = readBool(data, offset);

  return {
    creator,
    tokenId,
    price,
    hardCap,
    maxPerWallet,
    startTime,
    endTime,
    totalTokens,
    finalized,
  };
}

export function decodeU128(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 16);
  const lo = view.getBigUint64(0, true);
  const hi = view.getBigUint64(8, true);
  return (hi << 64n) | lo;
}
