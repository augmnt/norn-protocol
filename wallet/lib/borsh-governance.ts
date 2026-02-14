/**
 * Borsh encoding/decoding for the Governance contract messages.
 *
 * Borsh format for enums: 1-byte discriminant + field data.
 * Borsh strings: 4-byte LE length + UTF-8 bytes.
 * Borsh u64: 8 bytes LE.
 * Borsh u128: 16 bytes LE.
 * Borsh Address: 20 raw bytes.
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

// Discriminants: Initialize=0, Propose=1, Vote=2, Finalize=3

export function encodeInitialize(
  name: string,
  votingPeriod: bigint,
  quorum: bigint
): string {
  const parts = concat(
    new Uint8Array([0]),
    encodeString(name),
    encodeU64(votingPeriod),
    encodeU128(quorum)
  );
  return bytesToHex(parts);
}

export function encodePropose(title: string, description: string): string {
  const parts = concat(
    new Uint8Array([1]),
    encodeString(title),
    encodeString(description)
  );
  return bytesToHex(parts);
}

export function encodeVote(proposalId: bigint, support: boolean): string {
  const parts = concat(
    new Uint8Array([2]),
    encodeU64(proposalId),
    encodeBool(support)
  );
  return bytesToHex(parts);
}

export function encodeFinalize(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([3]), encodeU64(proposalId)));
}

// ── Query message encoders ──────────────────────────────────────────

// GetConfig=0, GetProposal=1, GetProposalCount=2, GetVote=3

export function encodeGetConfig(): string {
  return bytesToHex(new Uint8Array([0]));
}

export function encodeGetProposal(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU64(proposalId)));
}

export function encodeGetProposalCount(): string {
  return bytesToHex(new Uint8Array([2]));
}

export function encodeGetVote(proposalId: bigint, voter: string): string {
  return bytesToHex(
    concat(new Uint8Array([3]), encodeU64(proposalId), hexToBytes(voter))
  );
}

// ── Response decoders ─────────────────────────────────────────────────

export type ProposalStatus = "Active" | "Passed" | "Rejected" | "Expired";

export interface GovConfig {
  creator: string;
  name: string;
  votingPeriod: bigint;
  quorum: bigint;
  createdAt: bigint;
}

export interface GovProposal {
  id: bigint;
  proposer: string;
  title: string;
  description: string;
  forVotes: bigint;
  againstVotes: bigint;
  startTime: bigint;
  endTime: bigint;
  status: ProposalStatus;
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

function readString(data: Uint8Array, offset: number): [string, number] {
  const [len, newOffset] = readU32(data, offset);
  const strBytes = data.slice(newOffset, newOffset + len);
  return [new TextDecoder().decode(strBytes), newOffset + len];
}

function readBool(data: Uint8Array, offset: number): [boolean, number] {
  return [data[offset] !== 0, offset + 1];
}

function readProposalStatus(
  data: Uint8Array,
  offset: number
): [ProposalStatus, number] {
  const discriminant = data[offset];
  const statusMap: ProposalStatus[] = [
    "Active",
    "Passed",
    "Rejected",
    "Expired",
  ];
  return [statusMap[discriminant] ?? "Active", offset + 1];
}

export function decodeGovConfig(hex: string): GovConfig {
  const data = hexToBytes(hex);
  let offset = 0;

  let creator: string;
  [creator, offset] = readAddress(data, offset);
  let name: string;
  [name, offset] = readString(data, offset);
  let votingPeriod: bigint;
  [votingPeriod, offset] = readU64(data, offset);
  let quorum: bigint;
  [quorum, offset] = readU128(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return {
    creator,
    name,
    votingPeriod,
    quorum,
    createdAt,
  };
}

export function decodeGovProposal(hex: string): GovProposal {
  const data = hexToBytes(hex);
  let offset = 0;

  let id: bigint;
  [id, offset] = readU64(data, offset);
  let proposer: string;
  [proposer, offset] = readAddress(data, offset);
  let title: string;
  [title, offset] = readString(data, offset);
  let description: string;
  [description, offset] = readString(data, offset);
  let forVotes: bigint;
  [forVotes, offset] = readU128(data, offset);
  let againstVotes: bigint;
  [againstVotes, offset] = readU128(data, offset);
  let startTime: bigint;
  [startTime, offset] = readU64(data, offset);
  let endTime: bigint;
  [endTime, offset] = readU64(data, offset);
  let status: ProposalStatus;
  [status, offset] = readProposalStatus(data, offset);

  return {
    id,
    proposer,
    title,
    description,
    forVotes,
    againstVotes,
    startTime,
    endTime,
    status,
  };
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}

export function decodeBool(hex: string): boolean {
  const data = hexToBytes(hex);
  return data[0] !== 0;
}
