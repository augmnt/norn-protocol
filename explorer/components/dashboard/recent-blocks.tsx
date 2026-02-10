"use client";

import Link from "next/link";
import { Blocks } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TimeAgo } from "@/components/ui/time-ago";
import { HashDisplay } from "@/components/ui/hash-display";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useRecentBlocks } from "@/hooks/use-recent-blocks";
import { EmptyState } from "@/components/ui/empty-state";
import { formatNumber } from "@/lib/format";
import type { BlockInfo } from "@/types";

export function RecentBlocks() {
  const wsBlocks = useRealtimeStore((s) => s.recentBlocks);
  const { data: fetchedBlocks } = useRecentBlocks();

  // Merge WS blocks with RPC-fetched blocks, deduplicate by height
  const blocks = deduplicateBlocks(wsBlocks, fetchedBlocks ?? []);

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">Recent Blocks</CardTitle>
          <Link
            href="/blocks"
            className="text-xs text-norn hover:underline"
          >
            View all
          </Link>
        </div>
      </CardHeader>
      <CardContent className="px-0">
        {blocks.length === 0 ? (
          <EmptyState
            icon={Blocks}
            title="No blocks yet"
            description="Waiting for new blocks..."
          />
        ) : (
          <div className="space-y-0">
            {blocks.slice(0, 5).map((block) => (
              <div
                key={block.height}
                className="flex items-center justify-between px-6 py-2.5 border-b last:border-0 animate-slide-in"
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted">
                    <Blocks className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                  <div>
                    <Link
                      href={`/block/${block.height}`}
                      className="text-sm font-medium text-norn hover:underline tabular-nums"
                    >
                      #{formatNumber(block.height)}
                    </Link>
                    <p className="text-xs text-muted-foreground">
                      {block.transfer_count} txns
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <HashDisplay
                    hash={block.hash}
                    href={`/block/${block.height}`}
                    chars={4}
                    copy={false}
                  />
                  <TimeAgo timestamp={block.timestamp} className="text-xs" />
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function deduplicateBlocks(
  wsBlocks: BlockInfo[],
  fetchedBlocks: BlockInfo[],
): BlockInfo[] {
  const seen = new Set<number>();
  const result: BlockInfo[] = [];

  // WS blocks first (newest)
  for (const block of wsBlocks) {
    if (!seen.has(block.height)) {
      seen.add(block.height);
      result.push(block);
    }
  }

  // Then RPC-fetched blocks
  for (const block of fetchedBlocks) {
    if (!seen.has(block.height)) {
      seen.add(block.height);
      result.push(block);
    }
  }

  return result.sort((a, b) => b.height - a.height);
}
