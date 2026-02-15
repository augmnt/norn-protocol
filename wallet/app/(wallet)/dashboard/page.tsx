"use client";

import { useMemo } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { useBalance } from "@/hooks/use-balance";
import { useTokenBalances } from "@/hooks/use-token-balances";
import { useTxHistory } from "@/hooks/use-tx-history";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { BalanceHistoryChart } from "@/components/charts/balance-history-chart";
import { ActivityChart } from "@/components/charts/activity-chart";
import { formatNorn, truncateAddress, truncateHash } from "@/lib/format";
import { NATIVE_TOKEN_ID, QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { explorerAddressUrl, explorerTokenUrl, explorerTxUrl } from "@/lib/explorer";
import { useQueries, useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { formatAmount, strip0x } from "@/lib/format";
import Link from "next/link";
import { ArrowUpRight, QrCode, Coins, ArrowRightLeft, Copy } from "lucide-react";
import type { TokenInfo, TransactionHistoryEntry } from "@/types";
import { toast } from "sonner";

const NORN_DECIMALS = 12;

/** Format a timestamp as a chart label based on the overall time span. */
function chartLabel(ts: number, spanMs: number): string {
  const d = new Date(ts);
  if (spanMs < 24 * 60 * 60 * 1000) {
    // < 1 day: show hours
    return d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  }
  if (spanMs < 7 * 24 * 60 * 60 * 1000) {
    // < 1 week: show weekday + day
    return d.toLocaleDateString(undefined, { weekday: "short", day: "numeric" });
  }
  // >= 1 week: show month + day
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

/** Key for grouping activity by time bucket (day or hour). */
function activityBucketKey(ts: number, spanMs: number): string {
  const d = new Date(ts * 1000);
  if (spanMs < 24 * 60 * 60 * 1000) {
    return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}-${d.getHours()}`;
  }
  return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
}

function buildChartData(
  history: TransactionHistoryEntry[],
  address: string,
  currentBalance: string
) {
  // Sort oldest first — no knot_id dedup (a single knot can produce multiple records)
  const sorted = [...history].sort((a, b) => a.timestamp - b.timestamp);

  // Walk backwards from current balance to reconstruct balance at each NORN tx
  let bal = Number(currentBalance) / 10 ** NORN_DECIMALS;
  const reversed = [...sorted].reverse();
  const snapshots: { balance: number; timestamp: number }[] = [
    { balance: bal, timestamp: Date.now() },
  ];
  for (const tx of reversed) {
    if (tx.token_id !== NATIVE_TOKEN_ID) continue;
    const amt = Number(tx.amount) / 10 ** NORN_DECIMALS;
    // Use the direction field instead of manual address comparison
    bal = tx.direction === "sent" ? bal + amt : bal - amt;
    snapshots.push({ balance: Math.max(0, bal), timestamp: tx.timestamp * 1000 });
  }

  // If only 1 snapshot (no NORN txs), synthesize a point 7 days ago with same balance
  if (snapshots.length === 1) {
    snapshots.push({ balance: bal, timestamp: Date.now() - 7 * 24 * 60 * 60 * 1000 });
  }

  // Reverse back to chronological order
  snapshots.reverse();
  const balSpanMs = snapshots[snapshots.length - 1].timestamp - snapshots[0].timestamp;
  const balancePoints = snapshots.map((s) => ({
    balance: Math.round(s.balance * 100) / 100,
    label: chartLabel(s.timestamp, balSpanMs),
    timestamp: s.timestamp,
  }));

  // Activity: group all txs by time bucket (not just NORN)
  const actSpanMs = sorted.length >= 2
    ? (sorted[sorted.length - 1].timestamp - sorted[0].timestamp) * 1000
    : 0;
  const bucketMap = new Map<string, { label: string; sent: number; received: number }>();
  for (const tx of sorted) {
    const key = activityBucketKey(tx.timestamp, actSpanMs);
    const entry = bucketMap.get(key) ?? {
      label: chartLabel(tx.timestamp * 1000, actSpanMs),
      sent: 0,
      received: 0,
    };
    if (tx.direction === "sent") entry.sent++;
    else entry.received++;
    bucketMap.set(key, entry);
  }
  const activityPoints = Array.from(bucketMap.values());

  return { balancePoints, activityPoints };
}

export default function DashboardPage() {
  const { activeAddress, activeAccount } = useWallet();
  const { data: balance, isLoading: balanceLoading } = useBalance(activeAddress ?? undefined);
  const { data: threadState } = useTokenBalances(activeAddress ?? undefined);
  const { data: history } = useTxHistory(activeAddress ?? undefined, 1);

  // Fetch more history for charts (200 entries vs 20 for recent txs)
  const { data: chartHistory } = useQuery({
    queryKey: ["chartHistory", activeAddress],
    queryFn: () =>
      rpcCall<TransactionHistoryEntry[]>("norn_getTransactionHistory", [
        strip0x(activeAddress!),
        200,
        0,
      ]),
    staleTime: STALE_TIMES.dynamic,
    refetchInterval: 30_000,
    enabled: !!activeAddress,
  });

  const tokenBalances = threadState?.balances?.filter(
    (b) => b.token_id !== NATIVE_TOKEN_ID && BigInt(b.amount || "0") > 0n
  ) ?? [];

  const tokenInfoQueries = useQueries({
    queries: tokenBalances.map((b) => ({
      queryKey: QUERY_KEYS.tokenInfo(b.token_id),
      queryFn: () => rpcCall<TokenInfo>("norn_getTokenInfo", [b.token_id]),
      staleTime: STALE_TIMES.semiStatic,
    })),
  });
  const tokenInfoMap = new Map<string, TokenInfo>();
  tokenBalances.forEach((b, i) => {
    const data = tokenInfoQueries[i]?.data;
    if (data) tokenInfoMap.set(b.token_id, data);
  });

  const recentTxs = (history ?? [])
    .filter((tx, i, arr) => arr.findIndex((t) => t.knot_id === tx.knot_id) === i)
    .slice(0, 5);

  const { balancePoints, activityPoints } = useMemo(
    () =>
      chartHistory && activeAddress && balance
        ? buildChartData(chartHistory, activeAddress, balance.balance ?? "0")
        : { balancePoints: [], activityPoints: [] },
    [chartHistory, activeAddress, balance]
  );

  return (
    <PageContainer>
      {/* Hero Balance Section */}
      <div className="mb-6 py-8">
        <div className="flex flex-col items-center text-center gap-1">
          <p className="text-xs text-muted-foreground uppercase tracking-widest font-medium">
            {activeAccount?.label ?? "Wallet"} Balance
          </p>
          {balanceLoading ? (
            <Skeleton className="h-12 w-56 mt-2" />
          ) : (
            <button
              className="group flex items-center gap-2 mt-1"
              onClick={() => {
                navigator.clipboard.writeText(formatNorn(balance?.balance ?? "0"));
                toast.success("Balance copied");
              }}
              title="Click to copy balance"
            >
              <p className="text-4xl font-bold tabular-nums tracking-tight">
                {formatNorn(balance?.balance ?? "0")}
                <span className="text-base font-medium text-muted-foreground ml-2">NORN</span>
              </p>
              <Copy className="h-3.5 w-3.5 text-muted-foreground opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity" />
            </button>
          )}
          {activeAddress && (
            <div className="mt-2">
              <AddressDisplay address={activeAddress} href={explorerAddressUrl(activeAddress)} />
            </div>
          )}

          {/* Quick Actions */}
          <div className="flex gap-3 mt-6">
            <Button asChild className="rounded-full px-6">
              <Link href="/send">
                <ArrowUpRight className="mr-1.5 h-3.5 w-3.5" />
                Send
              </Link>
            </Button>
            <Button asChild variant="outline" className="rounded-full px-6">
              <Link href="/receive">
                <QrCode className="mr-1.5 h-3.5 w-3.5" />
                Receive
              </Link>
            </Button>
          </div>
        </div>
      </div>

      {/* Charts */}
      <div className="grid gap-4 md:grid-cols-2 mb-4">
        <BalanceHistoryChart data={balancePoints} />
        <ActivityChart data={activityPoints} />
      </div>

      {/* Two-Column Grid */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* Token Balances */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Token Balances</CardTitle>
            <Button asChild variant="ghost" size="sm" className="text-xs text-muted-foreground">
              <Link href="/tokens">View all</Link>
            </Button>
          </CardHeader>
          <CardContent>
            {tokenBalances.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-10 text-center">
                <Coins className="h-8 w-8 text-muted-foreground/30 mb-3" />
                <p className="text-sm font-medium text-muted-foreground">No tokens yet</p>
                <Link
                  href="/tokens"
                  className="text-xs text-norn hover:underline mt-1.5 underline-offset-4"
                >
                  Browse tokens
                </Link>
              </div>
            ) : (
              <div className="space-y-0.5">
                {tokenBalances.map((b) => {
                  const info = tokenInfoMap.get(b.token_id);
                  return (
                    <div
                      key={b.token_id}
                      className="flex items-center justify-between py-2 px-2 -mx-2 rounded-md hover:bg-muted/50 transition-colors group"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <a
                          href={explorerTokenUrl(b.token_id)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-sm font-medium text-foreground hover:text-norn transition-colors truncate"
                        >
                          {info?.symbol ?? b.token_id.slice(0, 8) + "..."}
                        </a>
                        {info?.name && (
                          <span className="text-xs text-muted-foreground truncate hidden sm:inline">
                            {info.name}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-1.5">
                        <span className="font-mono text-sm tabular-nums text-muted-foreground">
                          {formatAmount(b.amount, info?.decimals ?? 12)}
                        </span>
                        <Link
                          href={`/send?token=${b.token_id}`}
                          className="opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-foreground"
                          title="Send"
                        >
                          <ArrowUpRight className="h-3 w-3" />
                        </Link>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Recent Transactions */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Recent Transactions</CardTitle>
              <Link href="/history" className="text-xs text-norn hover:underline">
                View all
              </Link>
            </div>
          </CardHeader>
          <CardContent className="px-0 pb-2">
            {recentTxs.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <ArrowRightLeft className="h-10 w-10 text-muted-foreground/50 mb-4" />
                <p className="text-sm font-medium text-foreground">No transactions yet</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  Your activity will appear here
                </p>
              </div>
            ) : (
              <div>
                {recentTxs.map((tx) => (
                  <div key={tx.knot_id} className="px-6 py-3">
                    <div className="flex items-center justify-between">
                      {tx.knot_id ? (
                        <a
                          href={explorerTxUrl(tx.knot_id)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-sm font-mono text-norn hover:underline"
                        >
                          {truncateHash(tx.knot_id, 5)}
                        </a>
                      ) : (
                        <span className="text-sm text-muted-foreground">Pending</span>
                      )}
                      {tx.timestamp ? (
                        <TimeAgo timestamp={tx.timestamp} className="text-xs" />
                      ) : null}
                    </div>
                    <div className="flex items-center justify-between mt-0.5">
                      <span className="text-xs font-mono">
                        <a
                          href={explorerAddressUrl(tx.from)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-muted-foreground hover:text-norn"
                        >
                          {truncateAddress(tx.from)}
                        </a>
                        <span className="mx-1 text-muted-foreground">&rarr;</span>
                        <a
                          href={explorerAddressUrl(tx.to)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-muted-foreground hover:text-norn"
                        >
                          {truncateAddress(tx.to)}
                        </a>
                      </span>
                      <AmountDisplay
                        amount={tx.amount}
                        humanReadable={tx.human_readable}
                        symbol={tx.symbol}
                        className="text-xs text-muted-foreground"
                      />
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
