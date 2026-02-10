"use client";

import { Clock } from "lucide-react";
import { DataTable } from "@/components/ui/data-table";
import { AddressDisplay } from "@/components/ui/address-display";
import { HashDisplay } from "@/components/ui/hash-display";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { useRealtimeStore } from "@/stores/realtime-store";
import { timeAgo } from "@/lib/format";
import type { PendingTransactionEvent } from "@/types";

const columns = [
  {
    header: "Type",
    key: "type",
    render: (tx: PendingTransactionEvent) => (
      <Badge variant="outline" className="font-mono text-xs">
        {tx.tx_type}
      </Badge>
    ),
  },
  {
    header: "Hash",
    key: "hash",
    render: (tx: PendingTransactionEvent) => (
      <HashDisplay hash={tx.hash} chars={6} />
    ),
  },
  {
    header: "From",
    key: "from",
    render: (tx: PendingTransactionEvent) => (
      <AddressDisplay address={tx.from} />
    ),
  },
  {
    header: "Time",
    key: "time",
    className: "text-right",
    render: (tx: PendingTransactionEvent) => (
      <span className="text-sm text-muted-foreground">
        {timeAgo(tx.timestamp)}
      </span>
    ),
  },
];

export function PendingTransactions() {
  const pendingTxs = useRealtimeStore((s) => s.pendingTxs);

  if (pendingTxs.length === 0) {
    return (
      <EmptyState
        icon={Clock}
        title="No pending transactions"
        description="Pending transactions will appear here in real-time."
      />
    );
  }

  return (
    <DataTable
      columns={columns}
      data={pendingTxs}
      keyExtractor={(tx) => tx.hash}
    />
  );
}
