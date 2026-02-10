"use client";

import { Suspense, useState } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { Badge } from "@/components/ui/badge";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { EmptyState } from "@/components/ui/empty-state";
import { TransactionsFeed } from "@/components/transactions/transactions-feed";
import { PendingTransactions } from "@/components/transactions/pending-transactions";
import { useRealtimeStore } from "@/stores/realtime-store";
import { usePendingTxSubscription } from "@/hooks/use-subscriptions";
import { useTxHistory } from "@/hooks/use-tx-history";
import { useRecentHistory } from "@/hooks/use-recent-history";
import { isValidAddress } from "@/lib/format";
import { PAGE_SIZE } from "@/lib/constants";
import { ArrowRightLeft, Search } from "lucide-react";
import type { TransactionHistoryEntry } from "@/types";

const historyColumns = [
  {
    header: "Direction",
    key: "direction",
    render: (tx: TransactionHistoryEntry) => (
      <Badge variant={tx.direction === "sent" ? "secondary" : "outline"}>
        {tx.direction === "sent" ? "Sent" : "Received"}
      </Badge>
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
      <AmountDisplay amount={tx.amount} />
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
  const connected = useRealtimeStore((s) => s.connected);
  const searchParams = useSearchParams();
  const router = useRouter();
  usePendingTxSubscription();

  const activeTab = searchParams.get("tab") || "history";
  const [searchAddress, setSearchAddress] = useState("");
  const [activeAddress, setActiveAddress] = useState<string | undefined>();
  const [txPage, setTxPage] = useState(1);

  const handleTabChange = (tab: string) => {
    const params = new URLSearchParams(searchParams);
    params.set("tab", tab);
    router.replace(`/transactions?${params.toString()}`);
  };

  const { data: txHistory, isLoading: txLoading } = useTxHistory(
    activeAddress,
    txPage
  );
  const { data: recentHistory } = useRecentHistory(20);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    const q = searchAddress.trim();
    if (isValidAddress(q)) {
      setActiveAddress(q);
      setTxPage(1);
    }
  };

  return (
    <PageContainer
      title="Transactions"
      action={<LiveIndicator active={connected} label="Live Feed" />}
    >
      <Tabs value={activeTab} onValueChange={handleTabChange}>
        <TabsList>
          <TabsTrigger value="history">History</TabsTrigger>
          <TabsTrigger value="live">Live Feed</TabsTrigger>
          <TabsTrigger value="pending">Pending</TabsTrigger>
        </TabsList>

        <TabsContent value="history">
          <div className="space-y-4">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-medium">
                  Transaction History
                </CardTitle>
              </CardHeader>
              <CardContent>
                <form onSubmit={handleSearch} className="flex gap-2">
                  <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                      type="text"
                      value={searchAddress}
                      onChange={(e) => setSearchAddress(e.target.value)}
                      placeholder="Enter address (0x...) to view transaction history"
                      className="w-full rounded-md border bg-transparent px-3 py-2 pl-9 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                  </div>
                  <button
                    type="submit"
                    className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
                  >
                    Search
                  </button>
                </form>
              </CardContent>
            </Card>

            {activeAddress ? (
              <>
                <Card>
                  <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-medium">
                        Transactions for
                      </CardTitle>
                      <AddressDisplay address={activeAddress} />
                    </div>
                  </CardHeader>
                  <CardContent className="px-0">
                    {txLoading ? (
                      <div className="py-8 text-center text-sm text-muted-foreground">
                        Loading...
                      </div>
                    ) : (
                      <DataTable
                        columns={historyColumns}
                        data={txHistory ?? []}
                        keyExtractor={(tx) => tx.knot_id}
                        emptyMessage="No transactions found for this address"
                      />
                    )}
                  </CardContent>
                </Card>
                <Pagination
                  page={txPage}
                  hasNext={(txHistory?.length ?? 0) >= PAGE_SIZE}
                  onPageChange={setTxPage}
                />
              </>
            ) : recentHistory && recentHistory.length > 0 ? (
              <Card>
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm font-medium">
                    Recent Network Activity
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={historyColumns}
                    data={recentHistory}
                    keyExtractor={(tx) => tx.knot_id}
                    emptyMessage="No recent transactions"
                  />
                </CardContent>
              </Card>
            ) : (
              <EmptyState
                icon={ArrowRightLeft}
                title="Search for transactions"
                description="Enter an address above to view its transaction history."
              />
            )}
          </div>
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
