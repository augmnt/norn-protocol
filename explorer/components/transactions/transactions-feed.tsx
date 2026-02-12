"use client";

import { ArrowRightLeft } from "lucide-react";
import { DataTable } from "@/components/ui/data-table";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { EmptyState } from "@/components/ui/empty-state";
import { useRealtimeStore } from "@/stores/realtime-store";
import type { TransferEvent } from "@/types";
import Link from "next/link";

const columns = [
  {
    header: "From",
    key: "from",
    render: (tx: TransferEvent) => <AddressDisplay address={tx.from} />,
  },
  {
    header: "To",
    key: "to",
    render: (tx: TransferEvent) => <AddressDisplay address={tx.to} />,
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (tx: TransferEvent) => (
      <AmountDisplay
        amount={tx.amount}
        symbol={tx.symbol ?? "NORN"}
      />
    ),
  },
  {
    header: "Block",
    key: "block",
    className: "text-right",
    render: (tx: TransferEvent) =>
      tx.block_height != null ? (
        <Link
          href={`/block/${tx.block_height}`}
          className="font-mono text-sm text-norn hover:underline tabular-nums"
        >
          #{tx.block_height}
        </Link>
      ) : (
        <span className="text-sm text-muted-foreground">--</span>
      ),
  },
];

export function TransactionsFeed() {
  const transfers = useRealtimeStore((s) => s.recentTransfers);

  if (transfers.length === 0) {
    return (
      <EmptyState
        icon={ArrowRightLeft}
        title="No transactions yet"
        description="Live transactions will appear here as they are confirmed on the network."
      />
    );
  }

  return (
    <DataTable
      columns={columns}
      data={transfers}
      keyExtractor={(tx, i) => `${tx.from}-${tx.to}-${tx.amount}-${i}`}
    />
  );
}
