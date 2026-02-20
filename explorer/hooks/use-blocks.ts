"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES, PAGE_SIZE } from "@/lib/constants";
import type { BlockInfo } from "@/types";

async function fetchBlocks(
  page: number,
  latestHeight: number
): Promise<{ blocks: BlockInfo[]; hasNext: boolean }> {
  const client = getClient();
  const startHeight = latestHeight - (page - 1) * PAGE_SIZE;

  const fetches: Promise<BlockInfo | null>[] = [];
  for (let h = startHeight; h > startHeight - PAGE_SIZE && h >= 0; h--) {
    fetches.push(client.getBlock(h).catch(() => null));
  }

  const results = await Promise.all(fetches);
  const blocks = results.filter((b): b is BlockInfo => b !== null);

  return {
    blocks,
    hasNext: startHeight - PAGE_SIZE > 0,
  };
}

export function useBlocks(page: number, latestHeight: number | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.blocks(page),
    queryFn: () => fetchBlocks(page, latestHeight!),
    staleTime: STALE_TIMES.semiStatic,
    enabled: latestHeight !== undefined && latestHeight > 0,
  });
}
