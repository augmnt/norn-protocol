import { ArrowUpRight, ArrowDownLeft } from "lucide-react";
import { truncateAddress, formatNorn, timeAgo } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { TransactionHistoryEntry } from "@/types";

interface ActivityRowProps {
  tx: TransactionHistoryEntry;
  currentAddress: string;
}

export function ActivityRow({ tx, currentAddress }: ActivityRowProps) {
  const isSent =
    tx.from?.toLowerCase() === currentAddress.toLowerCase();
  const counterparty = isSent ? tx.to : tx.from;

  return (
    <div className="flex items-center gap-3 py-2.5 transition-colors duration-150 hover:bg-muted/50 -mx-2 px-2 rounded-md">
      <div
        className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-full",
          isSent
            ? "bg-orange-500/10 text-orange-400"
            : "bg-emerald-500/10 text-emerald-400",
        )}
      >
        {isSent ? (
          <ArrowUpRight className="h-4 w-4" />
        ) : (
          <ArrowDownLeft className="h-4 w-4" />
        )}
      </div>

      <div className="flex flex-1 flex-col">
        <span className="text-sm font-medium">
          {isSent ? "Sent" : "Received"}
        </span>
        {counterparty && (
          <span className="font-mono text-xs text-muted-foreground">
            {isSent ? "to " : "from "}
            {truncateAddress(counterparty)}
          </span>
        )}
      </div>

      <div className="flex flex-col items-end">
        <span
          className={cn(
            "font-mono text-sm font-medium tabular-nums",
            isSent ? "text-orange-400" : "text-emerald-400",
          )}
        >
          {isSent ? "-" : "+"}
          {formatNorn(tx.amount)}
        </span>
        {tx.timestamp && (
          <span className="text-xs text-muted-foreground">
            {timeAgo(tx.timestamp)}
          </span>
        )}
      </div>
    </div>
  );
}
