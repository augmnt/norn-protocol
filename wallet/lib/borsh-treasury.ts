/**
 * Borsh encoding/decoding for the Multisig Treasury contract messages.
 *
 * Borsh format for enums: 1-byte discriminant + field data.
 * Borsh strings: 4-byte LE length + UTF-8 bytes.
 * Borsh u64: 8 bytes LE.
 * Borsh u128: 16 bytes LE.
 * Borsh Address: 20 raw bytes.
 * Borsh TokenId: 32 raw bytes.
 * Borsh Vec<T>: 4-byte LE length + N×T bytes.
 */

// ── Helpers ────────────────────────────────────────────────────────────

function encodeString(s: string): Uint8Array {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(s);
  const buf = new Uint8Array(4 + bytes.length);
  new DataView(buf.buffer).setUint32(0, bytes.length, true);
  buf.set(bytes, 4);
  return buf;
}

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

// Discriminants: Initialize=0, Propose=1, Approve=2, Reject=3,
// Deposit=4, RevokeApproval=5, ExpireProposal=6

function encodeAddress(addr: string): Uint8Array {
  return hexToBytes(addr);
}

function encodeVecAddress(addrs: string[]): Uint8Array {
  const len = new Uint8Array(4);
  new DataView(len.buffer).setUint32(0, addrs.length, true);
  return concat(len, ...addrs.map((a) => encodeAddress(a)));
}

export function encodeInitialize(
  owners: string[],
  requiredApprovals: bigint,
  name: string
): string {
  const parts = concat(
    new Uint8Array([0]),
    encodeVecAddress(owners),
    encodeU64(requiredApprovals),
    encodeString(name)
  );
  return bytesToHex(parts);
}

export function encodePropose(
  to: string,
  tokenId: string,
  amount: bigint,
  description: string,
  deadline: bigint
): string {
  const parts = concat(
    new Uint8Array([1]),
    hexToBytes(to),
    hexToBytes(tokenId),
    encodeU128(amount),
    encodeString(description),
    encodeU64(deadline)
  );
  return bytesToHex(parts);
}

export function encodeApprove(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([2]), encodeU64(proposalId)));
}

export function encodeReject(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([3]), encodeU64(proposalId)));
}

export function encodeDeposit(tokenId: string, amount: bigint): string {
  return bytesToHex(
    concat(new Uint8Array([4]), hexToBytes(tokenId), encodeU128(amount))
  );
}

export function encodeRevokeApproval(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([5]), encodeU64(proposalId)));
}

export function encodeExpireProposal(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([6]), encodeU64(proposalId)));
}

// ── Query message encoders ──────────────────────────────────────────

// GetConfig=0, GetProposal=1, GetProposalCount=2

export function encodeGetConfig(): string {
  return bytesToHex(new Uint8Array([0]));
}

export function encodeGetProposal(proposalId: bigint): string {
  return bytesToHex(concat(new Uint8Array([1]), encodeU64(proposalId)));
}

export function encodeGetProposalCount(): string {
  return bytesToHex(new Uint8Array([2]));
}

// ── Response decoders ─────────────────────────────────────────────────

export type ProposalStatus = "Proposed" | "Executed" | "Rejected" | "Expired";

const STATUS_NAMES: ProposalStatus[] = [
  "Proposed",
  "Executed",
  "Rejected",
  "Expired",
];

export interface TreasuryConfig {
  name: string;
  owners: string[];
  requiredApprovals: bigint;
  createdAt: bigint;
}

export interface Proposal {
  id: bigint;
  proposer: string;
  to: string;
  tokenId: string;
  amount: bigint;
  description: string;
  status: ProposalStatus;
  approvalCount: bigint;
  createdAt: bigint;
  deadline: bigint;
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

function readString(data: Uint8Array, offset: number): [string, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 4);
  const len = view.getUint32(0, true);
  const str = new TextDecoder().decode(data.slice(offset + 4, offset + 4 + len));
  return [str, offset + 4 + len];
}

export function decodeTreasuryConfig(hex: string): TreasuryConfig {
  const data = hexToBytes(hex);
  let offset = 0;

  let name: string;
  [name, offset] = readString(data, offset);

  // Vec<Address>: 4-byte length + N×20 bytes
  const ownerCountView = new DataView(data.buffer, data.byteOffset + offset, 4);
  const ownerCount = ownerCountView.getUint32(0, true);
  offset += 4;
  const owners: string[] = [];
  for (let i = 0; i < ownerCount; i++) {
    let addr: string;
    [addr, offset] = readAddress(data, offset);
    owners.push(addr);
  }

  let requiredApprovals: bigint;
  [requiredApprovals, offset] = readU64(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return { name, owners, requiredApprovals, createdAt };
}

export function decodeProposal(hex: string): Proposal {
  const data = hexToBytes(hex);
  let offset = 0;

  let id: bigint;
  [id, offset] = readU64(data, offset);
  let proposer: string;
  [proposer, offset] = readAddress(data, offset);
  let to: string;
  [to, offset] = readAddress(data, offset);
  let tokenId: string;
  [tokenId, offset] = readTokenId(data, offset);
  let amount: bigint;
  [amount, offset] = readU128(data, offset);
  let description: string;
  [description, offset] = readString(data, offset);
  const statusByte = data[offset];
  offset += 1;
  const status = STATUS_NAMES[statusByte] ?? "Proposed";
  let approvalCount: bigint;
  [approvalCount, offset] = readU64(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);
  let deadline: bigint;
  [deadline, offset] = readU64(data, offset);

  return {
    id,
    proposer,
    to,
    tokenId,
    amount,
    description,
    status,
    approvalCount,
    createdAt,
    deadline,
  };
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}
