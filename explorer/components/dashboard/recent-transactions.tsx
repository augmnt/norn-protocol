"use client";

import Link from "next/link";
import { ArrowRightLeft } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { EmptyState } from "@/components/ui/empty-state";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useRecentHistory } from "@/hooks/use-recent-history";
import type { TransferEvent, TransactionHistoryEntry } from "@/types";

/** Unified row for display — merges WS events and RPC history. */
interface TxRow {
  key: string;
  from: string;
  to: string;
  amount: string;
  block_height?: number;
  timestamp?: number;
}

export function RecentTransactions() {
  const wsTransfers = useRealtimeStore((s) => s.recentTransfers);
  const { data: history } = useRecentHistory(5);

  // Merge WS transfers with RPC-fetched history, deduplicate
  const transactions = mergeTransactions(wsTransfers, history ?? []);

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">
            Recent Transactions
          </CardTitle>
          <Link
            href="/transactions"
            className="text-xs text-norn hover:underline"
          >
            View all
          </Link>
        </div>
      </CardHeader>
      <CardContent className="px-0">
        {transactions.length === 0 ? (
          <EmptyState
            icon={ArrowRightLeft}
            title="No transactions yet"
            description="Waiting for transactions..."
          />
        ) : (
          <div className="space-y-0">
            {transactions.slice(0, 5).map((tx) => (
              <div
                key={tx.key}
                className="flex items-center justify-between px-6 py-2.5 border-b last:border-0 animate-slide-in"
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-md bg-muted">
                    <ArrowRightLeft className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-1 text-xs">
                      <AddressDisplay address={tx.from} copy={false} />
                      <span className="text-muted-foreground">&rarr;</span>
                      <AddressDisplay address={tx.to} copy={false} />
                    </div>
                    {tx.timestamp ? (
                      <TimeAgo timestamp={tx.timestamp} className="text-xs" />
                    ) : tx.block_height != null ? (
                      <p className="text-xs text-muted-foreground">
                        Block #{tx.block_height}
                      </p>
                    ) : null}
                  </div>
                </div>
                <AmountDisplay amount={tx.amount} />
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function mergeTransactions(
  wsTransfers: TransferEvent[],
  history: TransactionHistoryEntry[],
): TxRow[] {
  const seen = new Set<string>();
  const result: TxRow[] = [];

  // WS transfers first (newest)
  for (const tx of wsTransfers) {
    const key = `${tx.from}-${tx.to}-${tx.amount}-${tx.block_height}`;
    if (!seen.has(key)) {
      seen.add(key);
      result.push({
        key,
        from: tx.from,
        to: tx.to,
        amount: tx.amount,
        block_height: tx.block_height,
      });
    }
  }

  // Then RPC-fetched history
  for (const tx of history) {
    const key = tx.knot_id || `${tx.from}-${tx.to}-${tx.amount}-${tx.block_height}`;
    if (!seen.has(key)) {
      seen.add(key);
      result.push({
        key,
        from: tx.from,
        to: tx.to,
        amount: tx.amount,
        block_height: tx.block_height,
        timestamp: tx.timestamp,
      });
    }
  }

  // Sort by block_height descending (newest first), fall back to insertion order
  return result.sort((a, b) => (b.block_height ?? 0) - (a.block_height ?? 0));
}
