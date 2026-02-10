"use client";

import { use, useState } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { StatCard } from "@/components/ui/stat-card";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { QRCodeDisplay } from "@/components/ui/qr-code";
import { useBalance } from "@/hooks/use-balance";
import { useTxHistory } from "@/hooks/use-tx-history";
import { useNames } from "@/hooks/use-names";
import { useThreadState } from "@/hooks/use-thread-state";
import { useAddressTransfersSubscription } from "@/hooks/use-subscriptions";
import { useFavoritesStore } from "@/stores/favorites-store";
import { formatNorn, strip0x } from "@/lib/format";
import { exportTransactionsCSV } from "@/lib/csv-export";
import { PAGE_SIZE } from "@/lib/constants";
import {
  Wallet,
  ArrowRightLeft,
  Tag,
  Coins,
  Star,
  Download,
  QrCode,
} from "lucide-react";
import type { TransactionHistoryEntry, NameInfo, BalanceEntry } from "@/types";

const txColumns = [
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
    header: "Counterparty",
    key: "counterparty",
    render: (tx: TransactionHistoryEntry) => (
      <AddressDisplay
        address={tx.direction === "sent" ? tx.to : tx.from}
      />
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

const balanceColumns = [
  {
    header: "Token",
    key: "token",
    render: (b: BalanceEntry) => (
      <Link
        href={`/token/${b.token_id}`}
        className="text-norn hover:underline"
      >
        <HashDisplay hash={b.token_id} chars={8} copy={false} />
      </Link>
    ),
  },
  {
    header: "Balance",
    key: "balance",
    className: "text-right",
    render: (b: BalanceEntry) => (
      <span className="font-mono text-sm tabular-nums">
        {b.human_readable}
      </span>
    ),
  },
];

export default function AddressPage({
  params,
}: {
  params: Promise<{ address: string }>;
}) {
  const { address } = use(params);
  const [txPage, setTxPage] = useState(1);
  const [showQR, setShowQR] = useState(false);

  const threadId = strip0x(address);
  const { data: balance, isLoading: balanceLoading } = useBalance(address);
  const { data: txHistory, isLoading: txLoading } = useTxHistory(
    address,
    txPage
  );
  const { data: names } = useNames(address);
  const { data: threadState } = useThreadState(threadId);

  useAddressTransfersSubscription(address);

  const isFavorite = useFavoritesStore((s) => s.isFavorite(address));
  const addFavorite = useFavoritesStore((s) => s.addFavorite);
  const removeFavorite = useFavoritesStore((s) => s.removeFavorite);

  const tokenBalances = threadState?.balances ?? [];

  return (
    <PageContainer title="Address">
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Addresses" },
            { label: address.slice(0, 10) + "..." },
          ]}
        />
        {/* Address Header */}
        <Card>
          <CardContent className="pt-6">
            <div className="flex flex-col gap-3">
              <div className="flex items-start justify-between">
                <div className="flex flex-col gap-2">
                  <p className="text-xs text-muted-foreground uppercase tracking-wider">
                    Address
                  </p>
                  <AddressDisplay
                    address={address}
                    link={false}
                    full
                    className="text-base"
                  />
                  {names && names.length > 0 && (
                    <div className="flex gap-2 mt-1">
                      {names.map((n) => (
                        <Badge key={n.name} variant="secondary">
                          {n.name}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-1.5">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={() => setShowQR(!showQR)}
                    title="Show QR code"
                  >
                    <QrCode className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={() =>
                      isFavorite
                        ? removeFavorite(address)
                        : addFavorite(address)
                    }
                    title={isFavorite ? "Remove from favorites" : "Add to favorites"}
                  >
                    <Star
                      className={`h-4 w-4 ${
                        isFavorite
                          ? "fill-yellow-500 text-yellow-500"
                          : "text-muted-foreground"
                      }`}
                    />
                  </Button>
                </div>
              </div>
              {showQR && (
                <div className="flex justify-center py-2">
                  <QRCodeDisplay value={address} />
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Balance */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            label="NORN Balance"
            value={
              balance ? `${formatNorn(balance.balance)} NORN` : "\u2014"
            }
            icon={Wallet}
            loading={balanceLoading}
          />
          <StatCard
            label="Tokens Held"
            value={tokenBalances.length}
            icon={Coins}
          />
          <StatCard
            label="Transactions"
            value={txHistory ? `${txHistory.length}+` : "\u2014"}
            icon={ArrowRightLeft}
            loading={txLoading}
          />
          <StatCard
            label="Names"
            value={names?.length ?? 0}
            icon={Tag}
          />
        </div>

        {/* Tabs */}
        <Tabs defaultValue="transactions">
          <div className="flex items-center justify-between">
            <TabsList>
              <TabsTrigger value="transactions">Transactions</TabsTrigger>
              <TabsTrigger value="tokens">
                Token Balances
                {tokenBalances.length > 0 && (
                  <Badge variant="secondary" className="ml-1.5 px-1.5 py-0 text-[10px]">
                    {tokenBalances.length}
                  </Badge>
                )}
              </TabsTrigger>
              <TabsTrigger value="names">Names</TabsTrigger>
            </TabsList>
            {txHistory && txHistory.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                className="hidden sm:flex items-center gap-1.5"
                onClick={() => exportTransactionsCSV(txHistory, address)}
              >
                <Download className="h-3.5 w-3.5" />
                Export CSV
              </Button>
            )}
          </div>
          <TabsContent value="transactions">
            {txLoading ? (
              <CardSkeleton />
            ) : (
              <>
                <DataTable
                  columns={txColumns}
                  data={txHistory ?? []}
                  keyExtractor={(tx, i) => `${tx.knot_id}-${i}`}
                  emptyMessage="No transactions found"
                />
                <Pagination
                  page={txPage}
                  hasNext={(txHistory?.length ?? 0) >= PAGE_SIZE}
                  onPageChange={setTxPage}
                  className="mt-4"
                />
              </>
            )}
          </TabsContent>
          <TabsContent value="tokens">
            {tokenBalances.length === 0 ? (
              <EmptyState
                icon={Coins}
                title="No token balances"
                description="This address holds no tokens."
              />
            ) : (
              <DataTable
                columns={balanceColumns}
                data={tokenBalances}
                keyExtractor={(b) => b.token_id}
              />
            )}
          </TabsContent>
          <TabsContent value="names">
            {!names || names.length === 0 ? (
              <EmptyState
                icon={Tag}
                title="No names registered"
                description="This address has no registered names."
              />
            ) : (
              <DataTable
                columns={[
                  {
                    header: "Name",
                    key: "name",
                    render: (n: NameInfo) => (
                      <span className="font-medium">{n.name}</span>
                    ),
                  },
                  {
                    header: "Registered",
                    key: "registered",
                    className: "text-right",
                    render: (n: NameInfo) => (
                      <TimeAgo timestamp={n.registered_at} />
                    ),
                  },
                ]}
                data={names}
                keyExtractor={(n) => n.name}
              />
            )}
          </TabsContent>
        </Tabs>
      </div>
    </PageContainer>
  );
}
