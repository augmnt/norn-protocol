/**
 * Borsh encoding/decoding for the Crowdfund contract messages.
 *
 * Borsh format for enums: 1-byte discriminant + field data.
 * Borsh strings: 4-byte LE length + UTF-8 bytes.
 * Borsh u64: 8 bytes LE.
 * Borsh u128: 16 bytes LE.
 * Borsh Address: 20 raw bytes.
 * Borsh TokenId: 32 raw bytes.
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

function encodeString(s: string): Uint8Array {
  const encoded = new TextEncoder().encode(s);
  const lenBuf = new Uint8Array(4);
  const lenView = new DataView(lenBuf.buffer);
  lenView.setUint32(0, encoded.length, true);
  return concat(lenBuf, encoded);
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

// Discriminants: Initialize=0, Contribute=1, Finalize=2, Refund=3

export function encodeInitialize(
  title: string,
  description: string,
  tokenId: string,
  goal: bigint,
  deadline: bigint
): string {
  const parts = concat(
    new Uint8Array([0]),
    encodeString(title),
    encodeString(description),
    hexToBytes(tokenId),
    encodeU128(goal),
    encodeU64(deadline)
  );
  return bytesToHex(parts);
}

export function encodeContribute(amount: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU128(amount)));
}

export function encodeFinalize(): string {
  return bytesToHex(new Uint8Array([2]));
}

export function encodeRefund(): string {
  return bytesToHex(new Uint8Array([3]));
}

// ── Query message encoders ──────────────────────────────────────────

// GetConfig=0, GetContribution=1, GetTotalRaised=2, GetContributorCount=3

export function encodeGetConfig(): string {
  return bytesToHex(new Uint8Array([0]));
}

export function encodeGetContribution(addr: string): string {
  return bytesToHex(concat(new Uint8Array([1]), hexToBytes(addr)));
}

export function encodeGetTotalRaised(): string {
  return bytesToHex(new Uint8Array([2]));
}

export function encodeGetContributorCount(): string {
  return bytesToHex(new Uint8Array([3]));
}

// ── Response decoders ─────────────────────────────────────────────────

export type CampaignStatus = "Active" | "Succeeded" | "Failed";

export interface CrowdfundConfig {
  creator: string;
  title: string;
  description: string;
  tokenId: string;
  goal: bigint;
  deadline: bigint;
  status: CampaignStatus;
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

function readU32(data: Uint8Array, offset: number): [number, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 4);
  return [view.getUint32(0, true), offset + 4];
}

function readAddress(data: Uint8Array, offset: number): [string, number] {
  const bytes = data.slice(offset, offset + 20);
  return ["0x" + bytesToHex(bytes), offset + 20];
}

function readTokenId(data: Uint8Array, offset: number): [string, number] {
  const bytes = data.slice(offset, offset + 32);
  return [bytesToHex(bytes), offset + 32];
}

function readString(data: Uint8Array, offset: number): [string, number] {
  const [len, newOffset] = readU32(data, offset);
  const strBytes = data.slice(newOffset, newOffset + len);
  return [new TextDecoder().decode(strBytes), newOffset + len];
}

function readCampaignStatus(
  data: Uint8Array,
  offset: number
): [CampaignStatus, number] {
  const discriminant = data[offset];
  const statusMap: CampaignStatus[] = ["Active", "Succeeded", "Failed"];
  return [statusMap[discriminant] ?? "Active", offset + 1];
}

export function decodeCrowdfundConfig(hex: string): CrowdfundConfig {
  const data = hexToBytes(hex);
  let offset = 0;

  let creator: string;
  [creator, offset] = readAddress(data, offset);
  let title: string;
  [title, offset] = readString(data, offset);
  let description: string;
  [description, offset] = readString(data, offset);
  let tokenId: string;
  [tokenId, offset] = readTokenId(data, offset);
  let goal: bigint;
  [goal, offset] = readU128(data, offset);
  let deadline: bigint;
  [deadline, offset] = readU64(data, offset);
  let status: CampaignStatus;
  [status, offset] = readCampaignStatus(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return {
    creator,
    title,
    description,
    tokenId,
    goal,
    deadline,
    status,
    createdAt,
  };
}

export function decodeU128(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 16);
  const lo = view.getBigUint64(0, true);
  const hi = view.getBigUint64(8, true);
  return (hi << 64n) | lo;
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}
