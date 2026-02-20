"use client";

import { useTokenInfo } from "@/hooks/use-token-info";
import { NATIVE_TOKEN_ID } from "@/lib/constants";
import { truncateAddress } from "@/lib/format";

export function useTokenSymbol(tokenId: string | undefined): string {
  const isNative = tokenId === NATIVE_TOKEN_ID;
  const { data } = useTokenInfo(isNative ? undefined : tokenId);

  if (!tokenId) return "Token";
  if (isNative) return "NORN";
  if (data?.symbol) return data.symbol;
  return truncateAddress("0x" + tokenId.slice(0, 40));
}
