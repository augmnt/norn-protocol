import {
  BorshWriter,
  nameRegistrationSigningData,
  tokenDefinitionSigningData,
  tokenMintSigningData,
  tokenBurnSigningData,
} from "./borsh.js";
import { blake3Hash, toHex, fromHex } from "./crypto.js";
import type { Wallet } from "./wallet.js";

/** NORN has 12 decimal places. */
const NORN_DECIMALS = 12;

/** Native token ID (32 zero bytes). */
const NATIVE_TOKEN_ID = new Uint8Array(32);

/** Name registration fee: 1 NORN. */
const NAME_REGISTRATION_FEE = BigInt(10) ** BigInt(NORN_DECIMALS);

/** Token creation fee: 10 NORN. */
const TOKEN_CREATION_FEE = BigInt(10) * BigInt(10) ** BigInt(NORN_DECIMALS);

/** Parse a human amount (e.g., "1.5") to the raw u128 representation. */
export function parseAmount(amount: string, decimals = NORN_DECIMALS): bigint {
  const parts = amount.split(".");
  const whole = BigInt(parts[0] || "0");
  let frac = parts[1] || "";
  if (frac.length > decimals) {
    frac = frac.slice(0, decimals);
  }
  frac = frac.padEnd(decimals, "0");
  return whole * BigInt(10) ** BigInt(decimals) + BigInt(frac);
}

/** Format a raw u128 amount to human-readable (e.g., "1.5"). */
export function formatAmount(amount: bigint, decimals = NORN_DECIMALS): string {
  const divisor = BigInt(10) ** BigInt(decimals);
  const whole = amount / divisor;
  const frac = (amount % divisor).toString().padStart(decimals, "0");
  const trimmed = frac.replace(/0+$/, "") || "0";
  return `${whole}.${trimmed}`;
}

/** Get the current timestamp in seconds. */
function now(): bigint {
  return BigInt(Math.floor(Date.now() / 1000));
}

/**
 * Build and sign a transfer transaction.
 *
 * Returns hex-encoded borsh bytes of a full Knot struct ready to submit via
 * `submitKnot`. The Rust handler expects: id, knot_type, timestamp, expiry,
 * before_states, after_states, payload (KnotPayload::Transfer), signatures.
 */
export function buildTransfer(
  wallet: Wallet,
  params: {
    to: string;
    amount: bigint;
    tokenId?: string;
    memo?: string;
  },
): string {
  const from = wallet.address; // 20 bytes
  const to = fromHex(params.to); // 20 bytes
  const tokenId = params.tokenId
    ? fromHex(params.tokenId)
    : NATIVE_TOKEN_ID;
  const timestamp = now();
  const memoBytes = params.memo
    ? new TextEncoder().encode(params.memo)
    : undefined;

  // Serialize the knot body (all fields except id and signatures).
  // This matches Rust's compute_knot_id: BLAKE3(knot_type ++ timestamp ++
  // expiry ++ before_states ++ after_states ++ payload).
  const body = new BorshWriter();

  // knot_type: KnotType::Transfer = variant 0
  body.writeU8(0);

  // timestamp: u64
  body.writeU64(timestamp);

  // expiry: Option<u64> = None
  body.writeU8(0);

  // before_states: Vec<ParticipantState> — 1 entry for sender
  body.writeU32(1);
  body.writeFixedBytes(from); // thread_id: [u8; 20]
  body.writeFixedBytes(wallet.publicKey); // pubkey: [u8; 32]
  body.writeU64(0n); // version: u64
  body.writeFixedBytes(new Uint8Array(32)); // state_hash: [u8; 32]

  // after_states: Vec<ParticipantState> — empty
  body.writeU32(0);

  // payload: KnotPayload::Transfer(TransferPayload)
  body.writeU8(0); // Transfer variant tag
  body.writeFixedBytes(tokenId); // token_id: [u8; 32]
  body.writeU128(params.amount); // amount: u128
  body.writeFixedBytes(from); // from: [u8; 20]
  body.writeFixedBytes(to); // to: [u8; 20]
  body.writeOptionBytes(memoBytes ?? null); // memo: Option<Vec<u8>>

  const bodyBytes = body.toBytes();

  // Compute knot ID = BLAKE3(body) and sign it.
  const knotId = blake3Hash(bodyBytes);
  const signature = wallet.sign(knotId);

  // Serialize the full Knot struct.
  const w = new BorshWriter();
  w.writeFixedBytes(knotId); // id: [u8; 32]
  w.writeFixedBytes(bodyBytes); // knot_type through payload
  // signatures: Vec<Signature> — 1 entry
  w.writeU32(1);
  w.writeFixedBytes(signature); // [u8; 64]

  return toHex(w.toBytes());
}

/**
 * Build and sign a name registration transaction.
 *
 * Returns hex-encoded borsh bytes ready to submit via `registerName`.
 */
export function buildNameRegistration(
  wallet: Wallet,
  name: string,
): string {
  const owner = wallet.address;
  const timestamp = now();
  const feePaid = NAME_REGISTRATION_FEE;

  const sigData = nameRegistrationSigningData({
    name,
    owner,
    timestamp,
    feePaid,
  });
  const signature = wallet.sign(sigData);

  const w = new BorshWriter();
  w.writeString(name);
  w.writeFixedBytes(owner); // 20 bytes
  w.writeFixedBytes(wallet.publicKey); // 32 bytes
  w.writeU64(timestamp);
  w.writeU128(feePaid);
  w.writeFixedBytes(signature); // 64 bytes

  return toHex(w.toBytes());
}

/**
 * Build and sign a token definition transaction.
 *
 * Returns hex-encoded borsh bytes ready to submit via `createToken`.
 */
export function buildTokenDefinition(
  wallet: Wallet,
  params: {
    name: string;
    symbol: string;
    decimals: number;
    maxSupply: bigint;
  },
): string {
  const creator = wallet.address;
  const timestamp = now();

  const sigData = tokenDefinitionSigningData({
    name: params.name,
    symbol: params.symbol,
    decimals: params.decimals,
    maxSupply: params.maxSupply,
    creator,
    timestamp,
  });
  const signature = wallet.sign(sigData);

  // Compute token ID = BLAKE3(creator ++ name ++ symbol ++ decimals ++ max_supply ++ timestamp)
  const idData = new BorshWriter();
  idData.writeFixedBytes(creator);
  idData.writeString(params.name);
  idData.writeString(params.symbol);
  idData.writeU8(params.decimals);
  idData.writeU128(params.maxSupply);
  idData.writeU64(timestamp);
  const tokenId = blake3Hash(idData.toBytes());

  const w = new BorshWriter();
  w.writeFixedBytes(tokenId); // 32 bytes
  w.writeString(params.name);
  w.writeString(params.symbol);
  w.writeU8(params.decimals);
  w.writeU128(params.maxSupply);
  w.writeFixedBytes(creator); // 20 bytes
  w.writeFixedBytes(wallet.publicKey); // 32 bytes
  w.writeU64(timestamp);
  w.writeU128(TOKEN_CREATION_FEE);
  w.writeFixedBytes(signature); // 64 bytes

  return toHex(w.toBytes());
}

/**
 * Build and sign a token mint transaction.
 *
 * Returns hex-encoded borsh bytes ready to submit via `mintToken`.
 */
export function buildTokenMint(
  wallet: Wallet,
  params: {
    tokenId: string;
    to: string;
    amount: bigint;
  },
): string {
  const tokenId = fromHex(params.tokenId);
  const to = fromHex(params.to);
  const timestamp = now();

  const sigData = tokenMintSigningData({
    tokenId,
    to,
    amount: params.amount,
    timestamp,
  });
  const signature = wallet.sign(sigData);

  const w = new BorshWriter();
  w.writeFixedBytes(tokenId); // 32 bytes
  w.writeFixedBytes(to); // 20 bytes
  w.writeU128(params.amount);
  w.writeFixedBytes(wallet.publicKey); // 32 bytes
  w.writeU64(timestamp);
  w.writeFixedBytes(signature); // 64 bytes

  return toHex(w.toBytes());
}

/**
 * Build and sign a token burn transaction.
 *
 * Returns hex-encoded borsh bytes ready to submit via `burnToken`.
 */
export function buildTokenBurn(
  wallet: Wallet,
  params: {
    tokenId: string;
    amount: bigint;
  },
): string {
  const tokenId = fromHex(params.tokenId);
  const burner = wallet.address;
  const timestamp = now();

  const sigData = tokenBurnSigningData({
    tokenId,
    amount: params.amount,
    burner,
    timestamp,
  });
  const signature = wallet.sign(sigData);

  const w = new BorshWriter();
  w.writeFixedBytes(tokenId); // 32 bytes
  w.writeU128(params.amount);
  w.writeFixedBytes(burner); // 20 bytes
  w.writeFixedBytes(wallet.publicKey); // 32 bytes
  w.writeU64(timestamp);
  w.writeFixedBytes(signature); // 64 bytes

  return toHex(w.toBytes());
}
