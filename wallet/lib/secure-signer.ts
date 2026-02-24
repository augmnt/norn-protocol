"use client";

import {
  Wallet,
  buildTransfer,
  buildNameRegistration,
  buildNameTransfer,
  buildNameRecordUpdate,
  buildTokenDefinition,
  buildTokenMint,
  buildTokenBurn,
  buildLoomRegistration,
  parseAmount,
  blake3Hash,
  toHex,
  fromHex,
} from "@norn-protocol/sdk";
import { zeroBytes } from "./passkey-crypto";
import { getWalletForSigning, getWalletForSigningWithPassword } from "./wallet-manager";
import type { StoredWalletMeta } from "@/types/passkey";

async function getWallet(
  meta: StoredWalletMeta,
  accountIndex: number,
  password?: string
): Promise<Wallet> {
  if (meta.usesPrf) {
    return getWalletForSigning(meta, accountIndex);
  }
  if (!password) {
    throw new Error("Password required for non-PRF wallet");
  }
  return getWalletForSigningWithPassword(meta, password);
}

function cleanupWallet(wallet: Wallet): void {
  // Zero the private key from memory
  zeroBytes(wallet.privateKey);
}

/** Sign a transfer transaction. Returns hex-encoded knot. */
export async function signTransfer(
  meta: StoredWalletMeta,
  params: {
    to: string;
    amount: string;
    tokenId?: string;
    memo?: string;
    decimals?: number;
    beforeState?: { version: bigint; stateHash: string };
  },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    const amountBigint = parseAmount(params.amount, params.decimals);
    return buildTransfer(wallet, {
      to: params.to,
      amount: amountBigint,
      tokenId: params.tokenId,
      memo: params.memo,
      beforeState: params.beforeState,
    });
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a name registration. Returns hex-encoded registration data. */
export async function signNameRegistration(
  meta: StoredWalletMeta,
  name: string,
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildNameRegistration(wallet, name);
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a name transfer. Returns hex-encoded transfer data. */
export async function signNameTransfer(
  meta: StoredWalletMeta,
  params: { name: string; to: string },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildNameTransfer(wallet, params);
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a name record update. Returns hex-encoded update data. */
export async function signNameRecordUpdate(
  meta: StoredWalletMeta,
  params: { name: string; key: string; value: string },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildNameRecordUpdate(wallet, params);
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a token definition. Returns hex-encoded definition data. */
export async function signTokenDefinition(
  meta: StoredWalletMeta,
  params: {
    name: string;
    symbol: string;
    decimals: number;
    maxSupply: string;
    initialSupply?: string;
  },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildTokenDefinition(wallet, {
      name: params.name,
      symbol: params.symbol,
      decimals: params.decimals,
      maxSupply: parseAmount(params.maxSupply, params.decimals),
      initialSupply: params.initialSupply
        ? parseAmount(params.initialSupply, params.decimals)
        : 0n,
    });
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a token mint. Returns hex-encoded mint data. */
export async function signTokenMint(
  meta: StoredWalletMeta,
  params: {
    tokenId: string;
    to: string;
    amount: string;
    decimals?: number;
  },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildTokenMint(wallet, {
      tokenId: params.tokenId,
      to: params.to,
      amount: parseAmount(params.amount, params.decimals ?? 12),
    });
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a token burn. Returns hex-encoded burn data. */
export async function signTokenBurn(
  meta: StoredWalletMeta,
  params: {
    tokenId: string;
    amount: string;
    decimals?: number;
  },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildTokenBurn(wallet, {
      tokenId: params.tokenId,
      amount: parseAmount(params.amount, params.decimals ?? 12),
    });
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a loom registration. Returns hex-encoded borsh bytes. */
export async function signLoomRegistration(
  meta: StoredWalletMeta,
  name: string,
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return buildLoomRegistration(wallet, { name });
  } finally {
    cleanupWallet(wallet);
  }
}

/** Concatenate multiple Uint8Arrays. */
function concatBytes(...arrays: Uint8Array[]): Uint8Array {
  const total = arrays.reduce((acc, a) => acc + a.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const a of arrays) {
    result.set(a, offset);
    offset += a.length;
  }
  return result;
}

/** Sign a loom execution request. Returns { signatureHex, pubkeyHex, senderHex }. */
export async function signExecuteLoom(
  meta: StoredWalletMeta,
  loomIdHex: string,
  inputHex: string,
  accountIndex = 0,
  password?: string
): Promise<{ signatureHex: string; pubkeyHex: string; senderHex: string }> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    const loomIdBytes = fromHex(loomIdHex);
    const inputBytes = fromHex(inputHex);
    const senderBytes = wallet.address;
    const signingMsg = blake3Hash(
      concatBytes(
        new TextEncoder().encode("norn_execute_loom"),
        loomIdBytes,
        inputBytes,
        senderBytes,
      )
    );
    const signature = wallet.sign(signingMsg);
    return {
      signatureHex: toHex(signature),
      pubkeyHex: toHex(wallet.publicKey),
      senderHex: toHex(senderBytes),
    };
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign a bytecode upload request. Returns { signatureHex, pubkeyHex }. */
export async function signBytecodeUpload(
  meta: StoredWalletMeta,
  loomIdHex: string,
  bytecodeBytes: Uint8Array,
  accountIndex = 0,
  password?: string
): Promise<{ signatureHex: string; pubkeyHex: string }> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    const loomIdBytes = fromHex(loomIdHex);
    const bytecodeHash = blake3Hash(bytecodeBytes);
    const signingMsg = blake3Hash(
      concatBytes(
        new TextEncoder().encode("norn_upload_bytecode"),
        loomIdBytes,
        bytecodeHash,
      )
    );
    const signature = wallet.sign(signingMsg);
    return {
      signatureHex: toHex(signature),
      pubkeyHex: toHex(wallet.publicKey),
    };
  } finally {
    cleanupWallet(wallet);
  }
}
