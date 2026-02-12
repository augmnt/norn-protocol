import { blake3Hash, fromHex } from "./crypto.js";

/** Tree depth: 256 bits = 32-byte key space. */
const TREE_DEPTH = 256;

/** The empty hash (32 zero bytes). */
const EMPTY_HASH = new Uint8Array(32);

/** Get bit at position `depth` from a hash key (MSB-first ordering). */
export function getBit(key: Uint8Array, depth: number): number {
  const byteIndex = Math.floor(depth / 8);
  const bitIndex = 7 - (depth % 8);
  if (byteIndex < 32) {
    return (key[byteIndex] >> bitIndex) & 1;
  }
  return 0;
}

/** Hash a leaf node: BLAKE3(0x00 || key || valueHash). */
export function hashLeaf(key: Uint8Array, valueHash: Uint8Array): Uint8Array {
  const data = new Uint8Array(65);
  data[0] = 0x00;
  data.set(key, 1);
  data.set(valueHash, 33);
  return blake3Hash(data);
}

/** Hash an internal node: BLAKE3(0x01 || left || right). */
export function hashInternal(
  left: Uint8Array,
  right: Uint8Array,
): Uint8Array {
  const data = new Uint8Array(65);
  data[0] = 0x01;
  data.set(left, 1);
  data.set(right, 33);
  return blake3Hash(data);
}

/** Compute SMT key for a balance entry: BLAKE3(address || tokenId). */
export function smtKey(address: Uint8Array, tokenId: Uint8Array): Uint8Array {
  const data = new Uint8Array(20 + 32);
  data.set(address, 0);
  data.set(tokenId, 20);
  return blake3Hash(data);
}

/** Encode a bigint as a 16-byte little-endian Uint8Array (u128 LE). */
export function encodeU128LE(value: bigint): Uint8Array {
  const bytes = new Uint8Array(16);
  let v = value;
  for (let i = 0; i < 16; i++) {
    bytes[i] = Number(v & 0xffn);
    v >>= 8n;
  }
  return bytes;
}

/** Check if two Uint8Arrays are equal. */
function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/**
 * Verify a sparse Merkle tree proof.
 *
 * @param root - The expected state root (32 bytes).
 * @param key - The SMT key being proved (32 bytes).
 * @param value - The value at the key (raw bytes; empty for non-inclusion).
 * @param siblings - Array of 256 sibling hashes (each 32 bytes).
 * @returns `true` if the proof is valid, `false` otherwise.
 */
export function verifyStateProof(
  root: Uint8Array,
  key: Uint8Array,
  value: Uint8Array,
  siblings: Uint8Array[],
): boolean {
  if (siblings.length !== TREE_DEPTH) return false;

  let current: Uint8Array;
  if (value.length === 0) {
    current = EMPTY_HASH;
  } else {
    const valueHash = blake3Hash(value);
    current = hashLeaf(key, valueHash);
  }

  for (let depth = TREE_DEPTH - 1; depth >= 0; depth--) {
    const bit = getBit(key, depth);
    const sibling = siblings[depth];

    if (bytesEqual(current, EMPTY_HASH) && bytesEqual(sibling, EMPTY_HASH)) {
      current = EMPTY_HASH;
    } else if (bit === 0) {
      current = hashInternal(current, sibling);
    } else {
      current = hashInternal(sibling, current);
    }
  }

  return bytesEqual(current, root);
}

/**
 * Verify a balance proof from the RPC `norn_getStateProof` response.
 *
 * Convenience wrapper that accepts hex strings as returned by the RPC.
 *
 * @param stateRoot - Hex-encoded state root.
 * @param address - Hex-encoded 20-byte address.
 * @param tokenId - Hex-encoded 32-byte token ID.
 * @param balance - The balance as a bigint (u128).
 * @param proof - Array of hex-encoded sibling hashes.
 * @returns `true` if the proof verifies correctly.
 */
export function verifyBalanceProof(
  stateRoot: string,
  address: string,
  tokenId: string,
  balance: bigint,
  proof: string[],
): boolean {
  const root = fromHex(stateRoot);
  const key = smtKey(fromHex(address), fromHex(tokenId));
  const value = encodeU128LE(balance);
  const siblings = proof.map((h) => fromHex(h));
  return verifyStateProof(root, key, value, siblings);
}
