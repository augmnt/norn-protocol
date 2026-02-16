/**
 * Borsh encoding/decoding for the AMM Pool contract messages.
 *
 * Borsh format for enums: 1-byte discriminant + field data.
 * Borsh u64: 8 bytes LE.
 * Borsh u128: 16 bytes LE.
 * Borsh u16: 2 bytes LE.
 * Borsh Address: 20 raw bytes.
 * Borsh TokenId: 32 raw bytes.
 * Borsh bool: 1 byte (0=false, 1=true).
 */

// ── Helpers ────────────────────────────────────────────────────────────

function encodeU16(n: number): Uint8Array {
  const buf = new Uint8Array(2);
  const view = new DataView(buf.buffer);
  view.setUint16(0, n, true);
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

// Discriminants: CreatePool=0, AddLiquidity=1, RemoveLiquidity=2,
// SwapNornForToken=3, SwapTokenForNorn=4, SetFeeBps=5

export function encodeCreatePool(
  token: string,
  nornAmount: bigint,
  tokenAmount: bigint
): string {
  return bytesToHex(
    concat(
      new Uint8Array([0]),
      hexToBytes(token),
      encodeU128(nornAmount),
      encodeU128(tokenAmount)
    )
  );
}

export function encodeAddLiquidity(
  poolId: bigint,
  nornAmount: bigint,
  tokenAmount: bigint
): string {
  return bytesToHex(
    concat(
      new Uint8Array([1]),
      encodeU64(poolId),
      encodeU128(nornAmount),
      encodeU128(tokenAmount)
    )
  );
}

export function encodeRemoveLiquidity(
  poolId: bigint,
  lpAmount: bigint
): string {
  return bytesToHex(
    concat(new Uint8Array([2]), encodeU64(poolId), encodeU128(lpAmount))
  );
}

export function encodeSwapNornForToken(
  poolId: bigint,
  nornAmount: bigint,
  minTokenOut: bigint
): string {
  return bytesToHex(
    concat(
      new Uint8Array([3]),
      encodeU64(poolId),
      encodeU128(nornAmount),
      encodeU128(minTokenOut)
    )
  );
}

export function encodeSwapTokenForNorn(
  poolId: bigint,
  tokenAmount: bigint,
  minNornOut: bigint
): string {
  return bytesToHex(
    concat(
      new Uint8Array([4]),
      encodeU64(poolId),
      encodeU128(tokenAmount),
      encodeU128(minNornOut)
    )
  );
}

// ── Query message encoders ──────────────────────────────────────────

// GetPool=0, GetPoolByToken=1, GetPoolCount=2, GetLpBalance=3,
// GetQuote=4, GetConfig=5

export function encodeGetPool(poolId: bigint): string {
  return bytesToHex(concat(new Uint8Array([0]), encodeU64(poolId)));
}

export function encodeGetPoolByToken(token: string): string {
  return bytesToHex(concat(new Uint8Array([1]), hexToBytes(token)));
}

export function encodeGetPoolCount(): string {
  return bytesToHex(new Uint8Array([2]));
}

export function encodeGetLpBalance(poolId: bigint, address: string): string {
  return bytesToHex(
    concat(new Uint8Array([3]), encodeU64(poolId), hexToBytes(address))
  );
}

export function encodeGetQuote(
  poolId: bigint,
  inputIsNorn: boolean,
  amountIn: bigint
): string {
  return bytesToHex(
    concat(
      new Uint8Array([4]),
      encodeU64(poolId),
      new Uint8Array([inputIsNorn ? 1 : 0]),
      encodeU128(amountIn)
    )
  );
}

export function encodeGetConfig(): string {
  return bytesToHex(new Uint8Array([5]));
}

// ── Response decoders ─────────────────────────────────────────────────

function readU16(data: Uint8Array, offset: number): [number, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 2);
  return [view.getUint16(0, true), offset + 2];
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

export interface AmmPool {
  id: bigint;
  token: string;
  reserveNorn: bigint;
  reserveToken: bigint;
  createdAt: bigint;
}

export interface AmmConfig {
  feeBps: number;
  owner: string;
}

export function decodePool(hex: string): AmmPool {
  const data = hexToBytes(hex);
  let offset = 0;

  let id: bigint;
  [id, offset] = readU64(data, offset);
  let token: string;
  [token, offset] = readTokenId(data, offset);
  let reserveNorn: bigint;
  [reserveNorn, offset] = readU128(data, offset);
  let reserveToken: bigint;
  [reserveToken, offset] = readU128(data, offset);
  let createdAt: bigint;
  [createdAt, offset] = readU64(data, offset);

  return { id, token, reserveNorn, reserveToken, createdAt };
}

export function decodeLpBalance(hex: string): bigint {
  const data = hexToBytes(hex);
  const [val] = readU128(data, 0);
  return val;
}

export function decodeQuote(hex: string): bigint {
  const data = hexToBytes(hex);
  const [val] = readU128(data, 0);
  return val;
}

export function decodeAmmConfig(hex: string): AmmConfig {
  const data = hexToBytes(hex);
  let offset = 0;

  let feeBps: number;
  [feeBps, offset] = readU16(data, offset);
  let owner: string;
  [owner, offset] = readAddress(data, offset);

  return { feeBps, owner };
}

export function decodeU64(hex: string): bigint {
  const data = hexToBytes(hex);
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}
