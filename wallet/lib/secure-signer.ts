"use client";

import {
  Wallet,
  buildTransfer,
  buildNameRegistration,
  buildTokenDefinition,
  buildTokenMint,
  buildTokenBurn,
  parseAmount,
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
  },
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    const amountBigint = parseAmount(params.amount);
    return buildTransfer(wallet, {
      to: params.to,
      amount: amountBigint,
      tokenId: params.tokenId,
      memo: params.memo,
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
