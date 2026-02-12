"use client";

import { Suspense, useState } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { TransactionsFeed } from "@/components/transactions/transactions-feed";
import { PendingTransactions } from "@/components/transactions/pending-transactions";
import { useRealtimeStore } from "@/stores/realtime-store";
import { usePendingTxSubscription } from "@/hooks/use-subscriptions";
import { useRecentHistory } from "@/hooks/use-recent-history";
import { PAGE_SIZE } from "@/lib/constants";
import type { TransactionHistoryEntry } from "@/types";

const columns = [
  {
    header: "Tx Hash",
    key: "knot_id",
    render: (tx: TransactionHistoryEntry) => (
      <HashDisplay
        hash={tx.knot_id}
        href={`/tx/${tx.knot_id}`}
        chars={6}
        copy={false}
      />
    ),
  },
  {
    header: "From",
    key: "from",
    render: (tx: TransactionHistoryEntry) => (
      <AddressDisplay address={tx.from} />
    ),
  },
  {
    header: "To",
    key: "to",
    render: (tx: TransactionHistoryEntry) => (
      <AddressDisplay address={tx.to} />
    ),
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (tx: TransactionHistoryEntry) => (
      <AmountDisplay
        amount={tx.amount}
        humanReadable={tx.human_readable}
        symbol={tx.symbol}
      />
    ),
  },
  {
    header: "Time",
    key: "time",
    className: "text-right",
    render: (tx: TransactionHistoryEntry) => (
      <TimeAgo timestamp={tx.timestamp} />
    ),
  },
];

export default function TransactionsPage() {
  return (
    <Suspense>
      <TransactionsContent />
    </Suspense>
  );
}

function TransactionsContent() {
  const connected = useRealtimeStore((s) => s.connectionState === "connected");
  const searchParams = useSearchParams();
  const router = useRouter();
  usePendingTxSubscription();

  const activeTab = searchParams.get("tab") || "all";
  const [page, setPage] = useState(1);

  const handleTabChange = (tab: string) => {
    const params = new URLSearchParams(searchParams);
    params.set("tab", tab);
    router.replace(`/transactions?${params.toString()}`);
  };

  const offset = (page - 1) * PAGE_SIZE;
  const {
    data: recentHistory,
    isLoading,
    error,
    refetch,
  } = useRecentHistory(PAGE_SIZE, offset);

  return (
    <PageContainer
      title="Transactions"
      action={<LiveIndicator active={connected} />}
    >
      <Tabs value={activeTab} onValueChange={handleTabChange}>
        <TabsList>
          <TabsTrigger value="all">All</TabsTrigger>
          <TabsTrigger value="live">Live Feed</TabsTrigger>
          <TabsTrigger value="pending">Pending</TabsTrigger>
        </TabsList>

        <TabsContent value="all">
          {isLoading ? (
            <TableSkeleton rows={10} cols={4} />
          ) : error ? (
            <ErrorState
              message="Failed to load transactions"
              retry={() => refetch()}
            />
          ) : (
            <>
              <DataTable
                columns={columns}
                data={recentHistory ?? []}
                keyExtractor={(tx, i) => `${tx.knot_id}-${i}`}
                emptyMessage="No transactions found"
                onRowClick={(tx) => router.push(`/tx/${tx.knot_id}`)}
              />
              <Pagination
                page={page}
                hasNext={(recentHistory?.length ?? 0) >= PAGE_SIZE}
                onPageChange={setPage}
                className="mt-4"
              />
            </>
          )}
        </TabsContent>

        <TabsContent value="live">
          <TransactionsFeed />
        </TabsContent>
        <TabsContent value="pending">
          <PendingTransactions />
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
