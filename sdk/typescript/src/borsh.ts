/**
 * Hand-coded borsh serialization for Norn transaction types.
 *
 * Borsh format:
 * - u8/u16/u32/u64/u128: little-endian fixed-width
 * - Vec<u8>: u32 length prefix + raw bytes
 * - String: u32 length prefix + UTF-8 bytes
 * - [u8; N]: raw N bytes (no length prefix)
 * - Option<T>: u8 (0=None, 1=Some) + T if Some
 */

/** Writer that accumulates bytes for borsh serialization. */
export class BorshWriter {
  private buffer: number[] = [];

  writeU8(v: number): void {
    this.buffer.push(v & 0xff);
  }

  writeU32(v: number): void {
    this.buffer.push(v & 0xff);
    this.buffer.push((v >> 8) & 0xff);
    this.buffer.push((v >> 16) & 0xff);
    this.buffer.push((v >> 24) & 0xff);
  }

  writeU64(v: bigint): void {
    for (let i = 0; i < 8; i++) {
      this.buffer.push(Number((v >> BigInt(i * 8)) & 0xffn));
    }
  }

  writeU128(v: bigint): void {
    for (let i = 0; i < 16; i++) {
      this.buffer.push(Number((v >> BigInt(i * 8)) & 0xffn));
    }
  }

  writeFixedBytes(bytes: Uint8Array): void {
    for (let i = 0; i < bytes.length; i++) {
      this.buffer.push(bytes[i]);
    }
  }

  writeBytes(bytes: Uint8Array): void {
    this.writeU32(bytes.length);
    this.writeFixedBytes(bytes);
  }

  writeString(s: string): void {
    const encoded = new TextEncoder().encode(s);
    this.writeBytes(encoded);
  }

  writeOptionBytes(v: Uint8Array | null): void {
    if (v === null) {
      this.writeU8(0);
    } else {
      this.writeU8(1);
      this.writeBytes(v);
    }
  }

  toBytes(): Uint8Array {
    return new Uint8Array(this.buffer);
  }
}

/** Reader for borsh deserialization. */
export class BorshReader {
  private offset = 0;
  private data: Uint8Array;

  constructor(data: Uint8Array) {
    this.data = data;
  }

  private ensureBytes(n: number): void {
    if (this.offset + n > this.data.length) {
      throw new Error(
        `Buffer underflow: need ${n} bytes at offset ${this.offset}, ` +
        `but only ${this.data.length - this.offset} remaining`,
      );
    }
  }

  readU8(): number {
    this.ensureBytes(1);
    return this.data[this.offset++];
  }

  readU32(): number {
    this.ensureBytes(4);
    const v =
      this.data[this.offset] |
      (this.data[this.offset + 1] << 8) |
      (this.data[this.offset + 2] << 16) |
      (this.data[this.offset + 3] << 24);
    this.offset += 4;
    return v >>> 0;
  }

  readU64(): bigint {
    this.ensureBytes(8);
    let v = 0n;
    for (let i = 0; i < 8; i++) {
      v |= BigInt(this.data[this.offset + i]) << BigInt(i * 8);
    }
    this.offset += 8;
    return v;
  }

  readU128(): bigint {
    this.ensureBytes(16);
    let v = 0n;
    for (let i = 0; i < 16; i++) {
      v |= BigInt(this.data[this.offset + i]) << BigInt(i * 8);
    }
    this.offset += 16;
    return v;
  }

  readFixedBytes(len: number): Uint8Array {
    this.ensureBytes(len);
    const bytes = this.data.slice(this.offset, this.offset + len);
    this.offset += len;
    return bytes;
  }

  readBytes(): Uint8Array {
    const len = this.readU32();
    return this.readFixedBytes(len);
  }

  readString(): string {
    const bytes = this.readBytes();
    return new TextDecoder().decode(bytes);
  }

  remaining(): number {
    return this.data.length - this.offset;
  }
}

/** Signing data for a transfer (matches Rust knot signing). */
export function transferSigningData(params: {
  from: Uint8Array;
  to: Uint8Array;
  tokenId: Uint8Array;
  amount: bigint;
  timestamp: bigint;
  memo?: Uint8Array;
}): Uint8Array {
  const w = new BorshWriter();
  w.writeFixedBytes(params.from); // 20 bytes
  w.writeFixedBytes(params.to); // 20 bytes
  w.writeFixedBytes(params.tokenId); // 32 bytes
  w.writeU128(params.amount);
  w.writeU64(params.timestamp);
  if (params.memo) {
    w.writeBytes(params.memo);
  }
  return w.toBytes();
}

/** Signing data for a name registration. */
export function nameRegistrationSigningData(params: {
  name: string;
  owner: Uint8Array;
  timestamp: bigint;
  feePaid: bigint;
}): Uint8Array {
  const w = new BorshWriter();
  // Rust uses raw name bytes (no borsh length prefix) for signing data
  w.writeFixedBytes(new TextEncoder().encode(params.name));
  w.writeFixedBytes(params.owner); // 20 bytes
  w.writeU64(params.timestamp);
  w.writeU128(params.feePaid);
  return w.toBytes();
}

/** Signing data for a token definition. */
export function tokenDefinitionSigningData(params: {
  name: string;
  symbol: string;
  decimals: number;
  maxSupply: bigint;
  initialSupply: bigint;
  creator: Uint8Array;
  timestamp: bigint;
}): Uint8Array {
  const w = new BorshWriter();
  // Rust uses raw bytes (no borsh length prefix) for signing data
  w.writeFixedBytes(new TextEncoder().encode(params.name));
  w.writeFixedBytes(new TextEncoder().encode(params.symbol));
  w.writeU8(params.decimals);
  w.writeU128(params.maxSupply);
  w.writeU128(params.initialSupply);
  w.writeFixedBytes(params.creator); // 20 bytes
  w.writeU64(params.timestamp);
  return w.toBytes();
}

/** Signing data for a token mint. */
export function tokenMintSigningData(params: {
  tokenId: Uint8Array;
  to: Uint8Array;
  amount: bigint;
  authority: Uint8Array;
  timestamp: bigint;
}): Uint8Array {
  const w = new BorshWriter();
  w.writeFixedBytes(params.tokenId); // 32 bytes
  w.writeFixedBytes(params.to); // 20 bytes
  w.writeU128(params.amount);
  w.writeFixedBytes(params.authority); // 20 bytes
  w.writeU64(params.timestamp);
  return w.toBytes();
}

/** Signing data for a token burn. */
export function tokenBurnSigningData(params: {
  tokenId: Uint8Array;
  burner: Uint8Array;
  amount: bigint;
  timestamp: bigint;
}): Uint8Array {
  const w = new BorshWriter();
  w.writeFixedBytes(params.tokenId); // 32 bytes
  w.writeFixedBytes(params.burner); // 20 bytes
  w.writeU128(params.amount);
  w.writeU64(params.timestamp);
  return w.toBytes();
}

/** Signing data for a loom deployment (matches Rust loom_deploy_signing_data). */
export function loomDeploySigningData(params: {
  name: string;
  operator: Uint8Array;
  timestamp: bigint;
}): Uint8Array {
  const w = new BorshWriter();
  // Rust uses raw name bytes (no borsh length prefix) for signing data
  w.writeFixedBytes(new TextEncoder().encode(params.name));
  w.writeFixedBytes(params.operator); // 32 bytes
  w.writeU64(params.timestamp);
  return w.toBytes();
}
