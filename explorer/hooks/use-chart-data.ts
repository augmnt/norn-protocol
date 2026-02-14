"use client";

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useLatestBlock } from "@/hooks/use-block";
import { STALE_TIMES } from "@/lib/constants";
import type { BlockInfo } from "@/types";

const CHART_BLOCK_COUNT = 30;

/**
 * Block times above this threshold (in seconds) are idle gaps, not real
 * production time. The chain target is 3s, so only gaps ≤ 5s represent
 * genuine back-to-back block production.
 */
const MAX_MEANINGFUL_BLOCK_TIME = 5;

export interface BlockChartPoint {
  height: number;
  timestamp: number;
  /** Raw wall-clock gap between consecutive blocks (for charts). */
  rawBlockTime: number | null;
  /** Filtered block time — only meaningful production gaps ≤ threshold (for stats). */
  blockTime: number | null;
  /** Real block production time in milliseconds (from node instrumentation). */
  productionMs: number | null;
  transferCount: number;
  nameCount: number;
  tokenDefCount: number;
  loomDeployCount: number;
  totalActivity: number;
  label: string;
}

function buildChartData(blocks: BlockInfo[]): BlockChartPoint[] {
  if (blocks.length === 0) return [];

  const sorted = [...blocks].sort((a, b) => a.height - b.height);
  return sorted.map((block, i) => {
    const prevTimestamp = i > 0 ? sorted[i - 1].timestamp : null;
    const rawBlockTime =
      prevTimestamp !== null
        ? Math.max(0, block.timestamp - prevTimestamp)
        : null;
    // Treat idle gaps (no transactions = no blocks produced) as null
    const blockTime =
      rawBlockTime !== null && rawBlockTime <= MAX_MEANINGFUL_BLOCK_TIME
        ? rawBlockTime
        : null;

    const totalActivity =
      block.transfer_count +
      block.name_registration_count +
      block.token_definition_count +
      block.token_mint_count +
      block.token_burn_count +
      block.loom_deploy_count;

    // Real production time from node instrumentation (µs → ms).
    const productionMs =
      block.production_us != null ? block.production_us / 1000 : null;

    return {
      height: block.height,
      timestamp: block.timestamp,
      rawBlockTime,
      blockTime,
      productionMs,
      transferCount: block.transfer_count,
      nameCount: block.name_registration_count,
      tokenDefCount: block.token_definition_count,
      loomDeployCount: block.loom_deploy_count,
      totalActivity,
      label: `#${block.height}`,
    };
  });
}

async function fetchHistoricalBlocks(
  latestHeight: number
): Promise<BlockInfo[]> {
  const client = getClient();
  const startHeight = latestHeight;
  const endHeight = Math.max(0, startHeight - CHART_BLOCK_COUNT);

  const promises: Promise<BlockInfo | null>[] = [];
  for (let h = startHeight; h >= endHeight; h--) {
    promises.push(client.getBlock(h).catch(() => null));
  }

  const results = await Promise.all(promises);
  return results.filter((b): b is BlockInfo => b !== null);
}

function useHistoricalBlocks(latestHeight: number | undefined) {
  // Round to nearest 10 so we don't refetch on every single new block
  const bucket = latestHeight !== undefined ? Math.floor(latestHeight / 10) : undefined;
  return useQuery({
    queryKey: ["chartBlocks", bucket],
    queryFn: () => fetchHistoricalBlocks(latestHeight!),
    staleTime: STALE_TIMES.immutable,
    enabled: latestHeight !== undefined && latestHeight >= 0,
  });
}

export function useChartData() {
  const wsBlocks = useRealtimeStore((s) => s.recentBlocks);
  const { data: latestBlock } = useLatestBlock();
  const { data: historicalBlocks } = useHistoricalBlocks(latestBlock?.height);

  const chartData = useMemo(() => {
    const seen = new Set<number>();
    const merged: BlockInfo[] = [];

    // WS blocks are the freshest — add them first
    for (const b of wsBlocks) {
      if (!seen.has(b.height)) {
        seen.add(b.height);
        merged.push(b);
      }
    }

    // Then the polled latest block
    if (latestBlock && !seen.has(latestBlock.height)) {
      seen.add(latestBlock.height);
      merged.push(latestBlock);
    }

    // Then historical blocks fetched on mount
    if (historicalBlocks) {
      for (const b of historicalBlocks) {
        if (!seen.has(b.height)) {
          seen.add(b.height);
          merged.push(b);
        }
      }
    }

    return buildChartData(merged);
  }, [wsBlocks, latestBlock, historicalBlocks]);

  /** Median production time from consecutive blocks (excludes idle gaps). */
  const blockProductionTime = useMemo(() => {
    const times = chartData
      .map((d) => d.blockTime)
      .filter((t): t is number => t !== null && t > 0);
    if (times.length === 0) return null;
    const sorted = [...times].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    return sorted.length % 2 === 0
      ? (sorted[mid - 1] + sorted[mid]) / 2
      : sorted[mid];
  }, [chartData]);

  const totalTxs = useMemo(
    () => chartData.reduce((sum, d) => sum + d.transferCount, 0),
    [chartData]
  );

  const sparklineHeights = useMemo(
    () => chartData.slice(-20).map((d) => d.height),
    [chartData]
  );

  const sparklineTxs = useMemo(
    () => chartData.slice(-20).map((d) => d.transferCount),
    [chartData]
  );

  const sparklineActivity = useMemo(
    () => chartData.slice(-20).map((d) => d.totalActivity),
    [chartData]
  );

  const sparklineBlockTimes = useMemo(
    () =>
      chartData
        .slice(-20)
        .map((d) => d.blockTime)
        .filter((t): t is number => t !== null),
    [chartData]
  );

  return {
    chartData,
    blockProductionTime,
    totalTxs,
    sparklineHeights,
    sparklineTxs,
    sparklineActivity,
    sparklineBlockTimes,
  };
}
