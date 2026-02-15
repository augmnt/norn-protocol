"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { getAppTypeForCodeHash } from "@/lib/code-hash-registry";
import { STALE_TIMES } from "@/lib/constants";
import { FEED_DECODERS, type FeedSummary } from "@/lib/feed-config-decoders";
import { strip0x } from "@/lib/format";
import type { LoomInfo, QueryResult } from "@/types";

export interface FeedItem {
  loomId: string;
  appType: string;
  name: string;
  deployedAt: number;
  active: boolean;
  participantCount: number;
  summary: FeedSummary | null;
}

async function queryLoom(
  loomId: string,
  inputHex: string
): Promise<{ output_hex?: string }> {
  const result = await rpcCall<QueryResult>("norn_queryLoom", [
    strip0x(loomId),
    inputHex,
  ]);
  return { output_hex: result?.output_hex };
}

async function fetchFeed(appTypeFilter?: string): Promise<FeedItem[]> {
  // Fetch all looms in a single batch
  const looms = await rpcCall<LoomInfo[]>("norn_listLooms", [200, 0]);

  // Filter to known app types
  const appLooms = looms
    .filter((loom) => {
      const type = loom.code_hash ? getAppTypeForCodeHash(loom.code_hash) : undefined;
      if (!type) return false;
      if (appTypeFilter && type !== appTypeFilter) return false;
      return true;
    })
    .map((loom) => ({
      loom,
      appType: getAppTypeForCodeHash(loom.code_hash!)!,
    }));

  // Fetch config summaries in parallel
  const items = await Promise.all(
    appLooms.map(async ({ loom, appType }) => {
      const decoder = FEED_DECODERS[appType];
      let summary: FeedSummary | null = null;
      if (decoder) {
        try {
          summary = await decoder.fetchSummary(loom.loom_id, queryLoom);
        } catch {
          // Config fetch failed — show card without summary
        }
      }
      return {
        loomId: loom.loom_id,
        appType,
        name: loom.name,
        deployedAt: loom.deployed_at,
        active: loom.active,
        participantCount: loom.participant_count,
        summary,
      };
    })
  );

  // Sort by deployed_at descending (newest first)
  items.sort((a, b) => b.deployedAt - a.deployedAt);
  return items;
}

export function useDiscoverFeed(appTypeFilter?: string) {
  return useQuery({
    queryKey: ["discoverFeed", appTypeFilter ?? "all"],
    queryFn: () => fetchFeed(appTypeFilter),
    staleTime: STALE_TIMES.semiStatic,
  });
}
