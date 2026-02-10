"use client";

import { useState } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { useBlocks } from "@/hooks/use-blocks";
import { useWeaveState } from "@/hooks/use-weave-state";
import { useRealtimeStore } from "@/stores/realtime-store";
import { formatNumber } from "@/lib/format";
import type { BlockInfo } from "@/types";

const columns = [
  {
    header: "Height",
    key: "height",
    render: (block: BlockInfo) => (
      <Link
        href={`/block/${block.height}`}
        className="font-mono text-sm text-norn hover:underline tabular-nums"
      >
        #{formatNumber(block.height)}
      </Link>
    ),
  },
  {
    header: "Hash",
    key: "hash",
    render: (block: BlockInfo) => (
      <HashDisplay
        hash={block.hash}
        href={`/block/${block.height}`}
        chars={6}
      />
    ),
  },
  {
    header: "Txns",
    key: "txns",
    className: "text-right",
    render: (block: BlockInfo) => (
      <span className="font-mono text-sm tabular-nums">
        {block.transfer_count}
      </span>
    ),
  },
  {
    header: "Time",
    key: "time",
    className: "text-right",
    render: (block: BlockInfo) => <TimeAgo timestamp={block.timestamp} />,
  },
];

export default function BlocksPage() {
  const [page, setPage] = useState(1);
  const { data: weave } = useWeaveState();
  const { data, isLoading, error, refetch } = useBlocks(page, weave?.height);
  const connected = useRealtimeStore((s) => s.connected);

  return (
    <PageContainer
      title="Blocks"
      action={<LiveIndicator active={connected} />}
    >
      {isLoading ? (
        <TableSkeleton rows={10} cols={4} />
      ) : error ? (
        <ErrorState
          message="Failed to load blocks"
          retry={() => refetch()}
        />
      ) : (
        <>
          <DataTable
            columns={columns}
            data={data?.blocks ?? []}
            keyExtractor={(b) => String(b.height)}
          />
          <Pagination
            page={page}
            hasNext={data?.hasNext ?? false}
            onPageChange={setPage}
            className="mt-4"
          />
        </>
      )}
    </PageContainer>
  );
}
