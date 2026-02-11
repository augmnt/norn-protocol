"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Blocks } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TimeAgo } from "@/components/ui/time-ago";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useRecentBlocks } from "@/hooks/use-recent-blocks";
import { EmptyState } from "@/components/ui/empty-state";
import { formatNumber } from "@/lib/format";
import type { BlockInfo } from "@/types";

export function RecentBlocks() {
  const router = useRouter();
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
      <CardContent className="px-0 pb-2">
        {blocks.length === 0 ? (
          <EmptyState
            icon={Blocks}
            title="No blocks yet"
            description="Waiting for new blocks..."
          />
        ) : (
          <div>
            {blocks.slice(0, 6).map((block) => (
              <div
                key={block.height}
                className="flex items-center justify-between px-6 py-3 cursor-pointer transition-colors hover:bg-muted/50"
                onClick={() => router.push(`/block/${block.height}`)}
              >
                <div>
                  <span className="text-sm font-medium tabular-nums">
                    #{formatNumber(block.height)}
                  </span>
                  <span className="ml-2 text-xs text-muted-foreground">
                    {block.transfer_count} txn{block.transfer_count !== 1 ? "s" : ""}
                  </span>
                </div>
                <TimeAgo timestamp={block.timestamp} className="text-xs" />
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

  // RPC-fetched blocks first (authoritative source of truth)
  for (const block of fetchedBlocks) {
    if (!seen.has(block.height)) {
      seen.add(block.height);
      result.push(block);
    }
  }

  // WS blocks fill in new blocks not yet available via RPC
  for (const block of wsBlocks) {
    if (!seen.has(block.height)) {
      seen.add(block.height);
      result.push(block);
    }
  }

  return result.sort((a, b) => b.height - a.height);
}
