"use client";

import { useState } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { HashDisplay } from "@/components/ui/hash-display";
import { Badge } from "@/components/ui/badge";
import { TimeAgo } from "@/components/ui/time-ago";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { useLoomsList } from "@/hooks/use-looms-list";
import { PAGE_SIZE } from "@/lib/constants";
import type { LoomInfo } from "@/types";

const columns = [
  {
    header: "Name",
    key: "name",
    render: (loom: LoomInfo) => (
      <Link
        href={`/contract/${loom.loom_id}`}
        className="text-sm font-medium text-norn hover:underline"
      >
        {loom.name}
      </Link>
    ),
  },
  {
    header: "Loom ID",
    key: "loomId",
    render: (loom: LoomInfo) => (
      <HashDisplay
        hash={loom.loom_id}
        href={`/contract/${loom.loom_id}`}
        chars={6}
      />
    ),
  },
  {
    header: "Status",
    key: "status",
    render: (loom: LoomInfo) => (
      <Badge variant={loom.active ? "default" : "secondary"}>
        {loom.active ? "Active" : "Inactive"}
      </Badge>
    ),
  },
  {
    header: "Bytecode",
    key: "bytecode",
    render: (loom: LoomInfo) => (
      <Badge variant={loom.has_bytecode ? "outline" : "secondary"}>
        {loom.has_bytecode ? "Deployed" : "None"}
      </Badge>
    ),
  },
  {
    header: "Deployed",
    key: "deployed",
    className: "text-right",
    render: (loom: LoomInfo) => <TimeAgo timestamp={loom.deployed_at} />,
  },
];

export default function ContractsPage() {
  const [page, setPage] = useState(1);
  const { data: looms, isLoading, error, refetch } = useLoomsList(page);

  return (
    <PageContainer title="Smart Contracts">
      {isLoading ? (
        <TableSkeleton rows={10} cols={5} />
      ) : error ? (
        <ErrorState
          message="Failed to load contracts"
          retry={() => refetch()}
        />
      ) : (
        <>
          <DataTable
            columns={columns}
            data={looms ?? []}
            keyExtractor={(l) => l.loom_id}
            emptyMessage="No contracts deployed yet"
          />
          <Pagination
            page={page}
            hasNext={(looms?.length ?? 0) >= PAGE_SIZE}
            onPageChange={setPage}
            className="mt-4"
          />
        </>
      )}
    </PageContainer>
  );
}
