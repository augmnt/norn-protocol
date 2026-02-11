"use client";

import { useState, useMemo, useCallback } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { useTxHistory } from "@/hooks/use-tx-history";
import { PageContainer } from "@/components/ui/page-container";
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { EmptyState } from "@/components/ui/empty-state";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { Badge } from "@/components/ui/badge";
import { CopyButton } from "@/components/ui/copy-button";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { PAGE_SIZE } from "@/lib/constants";
import { formatTimestamp } from "@/lib/format";
import {
  explorerAddressUrl,
  explorerTxUrl,
  explorerBlockUrl,
} from "@/lib/explorer";
import {
  ArrowRightLeft,
  ArrowDownLeft,
  ArrowUpRight,
  ExternalLink,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { TransactionHistoryEntry } from "@/types";

type Filter = "all" | "sent" | "received";

function isSent(tx: TransactionHistoryEntry, activeAddress: string): boolean {
  // Prefer the built-in direction field if available, otherwise compare addresses
  if (tx.direction) return tx.direction === "sent";
  return tx.from?.toLowerCase() === activeAddress.toLowerCase();
}

function DirectionBadge({
  tx,
  activeAddress,
}: {
  tx: TransactionHistoryEntry;
  activeAddress: string;
}) {
  const sent = isSent(tx, activeAddress);
  return (
    <span className="inline-flex items-center gap-1 rounded-md border border-border bg-secondary/50 px-2 py-0.5 text-xs font-medium text-muted-foreground">
      {sent ? (
        <ArrowUpRight className="h-3 w-3" />
      ) : (
        <ArrowDownLeft className="h-3 w-3" />
      )}
      {sent ? "Sent" : "Received"}
    </span>
  );
}

function DetailRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4 py-2.5 border-b border-border last:border-0">
      <span className="text-sm text-muted-foreground shrink-0">{label}</span>
      <div className="text-sm text-right min-w-0 break-all">{children}</div>
    </div>
  );
}

function TransactionDetailDialog({
  tx,
  open,
  onOpenChange,
  activeAddress,
}: {
  tx: TransactionHistoryEntry | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  activeAddress: string;
}) {
  if (!tx) return null;

  const sent = isSent(tx, activeAddress);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            Transaction Details
            <DirectionBadge tx={tx} activeAddress={activeAddress} />
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-0">
          <DetailRow label="Status">
            <Badge
              variant="secondary"
              className="border-border text-muted-foreground"
            >
              Confirmed
            </Badge>
          </DetailRow>

          <DetailRow label="Tx Hash">
            <span className="inline-flex items-center gap-1 font-mono text-xs">
              <span className="break-all">{tx.knot_id}</span>
              <CopyButton value={tx.knot_id} />
            </span>
          </DetailRow>

          <DetailRow label="From">
            <span className="inline-flex items-center gap-1 font-mono text-xs">
              <span className={cn("break-all", !sent && "text-muted-foreground")}>
                {tx.from}
              </span>
              <CopyButton value={tx.from} />
            </span>
          </DetailRow>

          <DetailRow label="To">
            <span className="inline-flex items-center gap-1 font-mono text-xs">
              <span className={cn("break-all", sent && "text-muted-foreground")}>
                {tx.to}
              </span>
              <CopyButton value={tx.to} />
            </span>
          </DetailRow>

          <DetailRow label="Amount">
            <AmountDisplay amount={tx.amount} />
          </DetailRow>

          <DetailRow label="Token ID">
            <span className="inline-flex items-center gap-1 font-mono text-xs">
              <span className="break-all">{tx.token_id}</span>
              <CopyButton value={tx.token_id} />
            </span>
          </DetailRow>

          {tx.memo && (
            <DetailRow label="Memo">
              <span className="text-sm">{tx.memo}</span>
            </DetailRow>
          )}

          {tx.block_height != null && (
            <DetailRow label="Block Height">
              <a
                href={explorerBlockUrl(tx.block_height)}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 font-mono text-sm hover:text-primary transition-colors underline-offset-4 hover:underline"
              >
                {tx.block_height.toLocaleString()}
                <ExternalLink className="h-3 w-3 text-muted-foreground" />
              </a>
            </DetailRow>
          )}

          <DetailRow label="Timestamp">
            <div className="flex flex-col items-end gap-0.5">
              <span className="text-sm">{formatTimestamp(tx.timestamp)}</span>
              <TimeAgo timestamp={tx.timestamp} className="text-xs" />
            </div>
          </DetailRow>
        </div>

        <DialogFooter>
          <Button variant="outline" size="sm" asChild>
            <a
              href={explorerTxUrl(tx.knot_id)}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5"
            >
              <ExternalLink className="h-3.5 w-3.5" />
              View on Explorer
            </a>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

const filterOptions: { value: Filter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "sent", label: "Sent" },
  { value: "received", label: "Received" },
];

export default function HistoryPage() {
  const { activeAddress } = useWallet();
  const [page, setPage] = useState(1);
  const [filter, setFilter] = useState<Filter>("all");
  const [selectedTx, setSelectedTx] =
    useState<TransactionHistoryEntry | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const {
    data: history,
    isLoading,
    error,
    refetch,
  } = useTxHistory(activeAddress ?? undefined, page);

  const filtered = useMemo(() => {
    if (!history || filter === "all" || !activeAddress) return history;
    const addr = activeAddress.toLowerCase();
    return history.filter((tx) =>
      filter === "sent"
        ? tx.from?.toLowerCase() === addr
        : tx.to?.toLowerCase() === addr
    );
  }, [history, filter, activeAddress]);

  const hasNext = (history?.length ?? 0) >= PAGE_SIZE;

  const handleRowClick = useCallback((tx: TransactionHistoryEntry) => {
    setSelectedTx(tx);
    setDialogOpen(true);
  }, []);

  const columns = useMemo(
    () => [
      {
        header: "Direction",
        key: "direction",
        render: (tx: TransactionHistoryEntry) =>
          activeAddress ? (
            <DirectionBadge tx={tx} activeAddress={activeAddress} />
          ) : null,
      },
      {
        header: "Tx Hash",
        key: "knot_id",
        render: (tx: TransactionHistoryEntry) => (
          <HashDisplay
            hash={tx.knot_id}
            href={explorerTxUrl(tx.knot_id)}
            chars={6}
            copy={false}
          />
        ),
      },
      {
        header: "From",
        key: "from",
        hideOnMobile: true,
        render: (tx: TransactionHistoryEntry) => (
          <AddressDisplay
            address={tx.from}
            href={explorerAddressUrl(tx.from)}
            copy={false}
          />
        ),
      },
      {
        header: "To",
        key: "to",
        hideOnMobile: true,
        render: (tx: TransactionHistoryEntry) => (
          <AddressDisplay
            address={tx.to}
            href={explorerAddressUrl(tx.to)}
            copy={false}
          />
        ),
      },
      {
        header: "Amount",
        key: "amount",
        className: "text-right",
        render: (tx: TransactionHistoryEntry) => (
          <div className="flex flex-col items-end">
            <AmountDisplay amount={tx.amount} />
            {tx.memo && (
              <span className="text-xs text-muted-foreground mt-0.5 max-w-[200px] truncate">
                {tx.memo}
              </span>
            )}
          </div>
        ),
      },
      {
        header: "Status",
        key: "status",
        hideOnMobile: true,
        render: () => (
          <Badge
            variant="secondary"
            className="border-border text-muted-foreground text-[10px] px-1.5 py-0"
          >
            Confirmed
          </Badge>
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
    ],
    [activeAddress]
  );

  return (
    <PageContainer
      title="Transactions"
      action={
        <div className="flex gap-1">
          {filterOptions.map((opt) => (
            <Button
              key={opt.value}
              variant="outline"
              size="sm"
              onClick={() => {
                setFilter(opt.value);
                setPage(1);
              }}
              className={cn(
                "text-xs h-7 px-2.5",
                filter === opt.value && "bg-accent text-accent-foreground"
              )}
            >
              {opt.label}
            </Button>
          ))}
        </div>
      }
    >
      {isLoading ? (
        <TableSkeleton rows={10} cols={7} />
      ) : error ? (
        <ErrorState
          message="Failed to load transactions"
          retry={() => refetch()}
        />
      ) : !filtered || filtered.length === 0 ? (
        <EmptyState
          icon={ArrowRightLeft}
          title="No transactions yet"
          description="Send or receive NORN to see your transaction history."
        />
      ) : (
        <>
          <DataTable
            columns={columns}
            data={filtered!}
            keyExtractor={(tx, i) => `${tx.knot_id}-${i}`}
            emptyMessage="No transactions found"
            onRowClick={handleRowClick}
          />
          <Pagination
            page={page}
            hasNext={hasNext}
            onPageChange={setPage}
            className="mt-4"
          />
        </>
      )}

      <TransactionDetailDialog
        tx={selectedTx}
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        activeAddress={activeAddress ?? ""}
      />
    </PageContainer>
  );
}
