"use client";

import Link from "next/link";
import { ArrowRightLeft } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TimeAgo } from "@/components/ui/time-ago";
import { EmptyState } from "@/components/ui/empty-state";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useRecentHistory } from "@/hooks/use-recent-history";
import { truncateAddress, truncateHash } from "@/lib/format";
import type { TransferEvent, TransactionHistoryEntry } from "@/types";

/** Unified row for display — merges WS events and RPC history. */
interface TxRow {
  key: string;
  knot_id?: string;
  from: string;
  to: string;
  amount: string;
  human_readable: string;
  symbol: string;
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
      <CardContent className="px-0 pb-2">
        {transactions.length === 0 ? (
          <EmptyState
            icon={ArrowRightLeft}
            title="No transactions yet"
            description="Waiting for transactions..."
          />
        ) : (
          <div>
            {transactions.slice(0, 5).map((tx) => (
              <div
                key={tx.key}
                className="px-6 py-2.5"
              >
                <div className="flex items-center justify-between">
                  {tx.knot_id ? (
                    <Link
                      href={`/tx/${tx.knot_id}`}
                      className="text-sm font-mono text-norn hover:underline"
                    >
                      {truncateHash(tx.knot_id, 5)}
                    </Link>
                  ) : (
                    <span className="text-sm text-muted-foreground">
                      Pending
                    </span>
                  )}
                  {tx.timestamp ? (
                    <TimeAgo timestamp={tx.timestamp} className="text-xs" />
                  ) : null}
                </div>
                <div className="flex items-center justify-between mt-0.5">
                  <span className="text-xs font-mono">
                    <Link
                      href={`/address/${tx.from}`}
                      className="text-muted-foreground hover:text-norn"
                    >
                      {truncateAddress(tx.from)}
                    </Link>
                    <span className="mx-1 text-muted-foreground">&rarr;</span>
                    <Link
                      href={`/address/${tx.to}`}
                      className="text-muted-foreground hover:text-norn"
                    >
                      {truncateAddress(tx.to)}
                    </Link>
                  </span>
                  <span className="text-xs font-mono tabular-nums text-muted-foreground">
                    {tx.human_readable} {tx.symbol}
                  </span>
                </div>
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
  const result: TxRow[] = [];

  // RPC history — deduplicate by knot_id (API can return both sent+received records)
  const seen = new Set<string>();
  for (const tx of history) {
    const key = tx.knot_id || `${tx.from}-${tx.to}-${tx.amount}-${tx.timestamp}`;
    if (seen.has(key)) continue;
    seen.add(key);
    result.push({
      key,
      knot_id: tx.knot_id,
      from: tx.from,
      to: tx.to,
      amount: tx.amount,
      human_readable: tx.human_readable,
      symbol: tx.symbol,
      block_height: tx.block_height,
      timestamp: tx.timestamp,
    });
  }

  // WS transfers — only add if not already in RPC results.
  // Match on from+to+amount as best-effort dedup against RPC entries.
  for (const tx of wsTransfers) {
    const matchKey = `${tx.from}-${tx.to}-${tx.amount}`;
    const alreadyInRpc = result.some(
      (r) => r.from === tx.from && r.to === tx.to && r.amount === tx.amount
    );
    if (!alreadyInRpc) {
      result.push({
        key: `ws-${matchKey}-${tx.block_height}`,
        from: tx.from,
        to: tx.to,
        amount: tx.amount,
        human_readable: tx.human_readable,
        symbol: tx.symbol ?? "NORN",
        block_height: tx.block_height ?? undefined,
      });
    }
  }

  // Sort by timestamp descending (newest first), fall back to block height
  return result.sort((a, b) => (b.timestamp ?? b.block_height ?? 0) - (a.timestamp ?? a.block_height ?? 0));
}
