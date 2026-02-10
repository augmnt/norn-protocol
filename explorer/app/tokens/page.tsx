"use client";

import { useState } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { AddressDisplay } from "@/components/ui/address-display";
import { HashDisplay } from "@/components/ui/hash-display";
import { Badge } from "@/components/ui/badge";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { useTokensList } from "@/hooks/use-tokens-list";
import { formatAmount } from "@/lib/format";
import { PAGE_SIZE } from "@/lib/constants";
import type { TokenInfo } from "@/types";

const columns = [
  {
    header: "Symbol",
    key: "symbol",
    render: (token: TokenInfo) => (
      <Link
        href={`/token/${token.token_id}`}
        className="text-norn hover:underline"
      >
        <Badge variant="outline" className="font-mono">
          {token.symbol}
        </Badge>
      </Link>
    ),
  },
  {
    header: "Name",
    key: "name",
    render: (token: TokenInfo) => (
      <span className="text-sm font-medium">{token.name}</span>
    ),
  },
  {
    header: "Supply",
    key: "supply",
    className: "text-right",
    render: (token: TokenInfo) => (
      <span className="font-mono text-sm tabular-nums">
        {formatAmount(token.current_supply, token.decimals)}
      </span>
    ),
  },
  {
    header: "Creator",
    key: "creator",
    render: (token: TokenInfo) => (
      <AddressDisplay address={token.creator} />
    ),
  },
  {
    header: "Decimals",
    key: "decimals",
    className: "text-right",
    render: (token: TokenInfo) => (
      <span className="font-mono text-sm">{token.decimals}</span>
    ),
  },
];

export default function TokensPage() {
  const [page, setPage] = useState(1);
  const { data: tokens, isLoading, error, refetch } = useTokensList(page);

  return (
    <PageContainer title="Tokens">
      {isLoading ? (
        <TableSkeleton rows={10} cols={5} />
      ) : error ? (
        <ErrorState message="Failed to load tokens" retry={() => refetch()} />
      ) : (
        <>
          <DataTable
            columns={columns}
            data={tokens ?? []}
            keyExtractor={(t) => t.token_id}
            emptyMessage="No tokens created yet"
          />
          <Pagination
            page={page}
            hasNext={(tokens?.length ?? 0) >= PAGE_SIZE}
            onPageChange={setPage}
            className="mt-4"
          />
        </>
      )}
    </PageContainer>
  );
}
