/**
 * Borsh encoding/decoding for the Token Vesting contract messages.
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

function encodeBool(b: boolean): Uint8Array {
  return new Uint8Array([b ? 1 : 0]);
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

// Discriminants: CreateSchedule=0, Claim=1, Revoke=2

export function encodeCreateSchedule(
  beneficiary: string,
  tokenId: string,
  amount: bigint,
  startTime: bigint,
  cliffDuration: bigint,
  totalDuration: bigint,
  revocable: boolean
): string {
  const parts = concat(
    new Uint8Array([0]),
    hexToBytes(beneficiary),
    hexToBytes(tokenId),
    encodeU128(amount),
    encodeU64(startTime),
    encodeU64(cliffDuration),
    encodeU64(totalDuration),
    encodeBool(revocable)
  );
  return bytesToHex(parts);
}

export function encodeClaim(scheduleId: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU64(scheduleId)));
}

export function encodeRevoke(scheduleId: bigint): string {
  return bytesToHex(concat(new Uint8Array([2]), encodeU64(scheduleId)));
}

// ── Query message encoders ──────────────────────────────────────────

// GetSchedule=0, GetScheduleCount=1, GetClaimable=2

export function encodeGetSchedule(scheduleId: bigint): string {
  return bytesToHex(concat(new Uint8Array([0]), encodeU64(scheduleId)));
}

export function encodeGetScheduleCount(): string {
  return bytesToHex(new Uint8Array([1]));
}

export function encodeGetClaimable(scheduleId: bigint): string {
  return bytesToHex(concat(new Uint8Array([2]), encodeU64(scheduleId)));
}

// ── Response decoders ─────────────────────────────────────────────────

export interface VestingSchedule {
  id: bigint;
  creator: string;
  beneficiary: string;
  tokenId: string;
  totalAmount: bigint;
  claimedAmount: bigint;
  startTime: bigint;
  cliffDuration: bigint;
  totalDuration: bigint;
  revocable: boolean;
  revoked: boolean;
  createdAt: bigint;
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

export function decodeVestingSchedule(hex: string): VestingSchedule {
  const data = hexToBytes(hex);
  let offset = 0;

  let id: bigint;
  [id, offset] = readU64(data, offset);
  let creator: string;
  [creator, offset] = readAddress(data, offset);
  let beneficiary: string;
  [beneficiary, offset] = readAddress(data, offset);
  let tokenId: string;
  [tokenId, offset] = readTokenId(data, offset);
  let totalAmount: bigint;
  [totalAmount, offset] = readU128(data, offset);
  let claimedAmount: bigint;
  [claimedAmount, offset] = readU128(data, offset);
  let startTime: bigint;
  [startTime, offset] = readU64(data, offset);
  let cliffDuration: bigint;
  [cliffDuration, offset] = readU64(data, offset);
  let totalDuration: bigint;
  [totalDuration, offset] = readU64(data, offset);
  let revocable: boolean;
  [revocable, offset] = readBool(data, offset);
  let revoked: boolean;
  [revoked, offset] = readBool(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return {
    id,
    creator,
    beneficiary,
    tokenId,
    totalAmount,
    claimedAmount,
    startTime,
    cliffDuration,
    totalDuration,
    revocable,
    revoked,
    createdAt,
  };
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}

export function decodeU128(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 16);
  const lo = view.getBigUint64(0, true);
  const hi = view.getBigUint64(8, true);
  return (hi << 64n) | lo;
}
