/**
 * Borsh encoding/decoding for the Timelock contract messages.
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

// Discriminants: Lock=0, Withdraw=1

export function encodeLock(
  tokenId: string,
  amount: bigint,
  unlockTime: bigint
): string {
  const parts = concat(
    new Uint8Array([0]),
    hexToBytes(tokenId),
    encodeU128(amount),
    encodeU64(unlockTime)
  );
  return bytesToHex(parts);
}

export function encodeWithdraw(lockId: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU64(lockId)));
}

// ── Query message encoders ──────────────────────────────────────────

// GetLock=0, GetLockCount=1

export function encodeGetLock(lockId: bigint): string {
  return bytesToHex(concat(new Uint8Array([0]), encodeU64(lockId)));
}

export function encodeGetLockCount(): string {
  return bytesToHex(new Uint8Array([1]));
}

// ── Response decoders ─────────────────────────────────────────────────

export interface LockInfo {
  id: bigint;
  owner: string;
  tokenId: string;
  amount: bigint;
  unlockTime: bigint;
  withdrawn: boolean;
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

export function decodeLockInfo(hex: string): LockInfo {
  const data = hexToBytes(hex);
  let offset = 0;

  let id: bigint;
  [id, offset] = readU64(data, offset);
  let owner: string;
  [owner, offset] = readAddress(data, offset);
  let tokenId: string;
  [tokenId, offset] = readTokenId(data, offset);
  let amount: bigint;
  [amount, offset] = readU128(data, offset);
  let unlockTime: bigint;
  [unlockTime, offset] = readU64(data, offset);
  let withdrawn: boolean;
  [withdrawn, offset] = readBool(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return {
    id,
    owner,
    tokenId,
    amount,
    unlockTime,
    withdrawn,
    createdAt,
  };
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}
