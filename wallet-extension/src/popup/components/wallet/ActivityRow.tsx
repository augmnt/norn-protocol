import { ArrowUpRight, ArrowDownLeft } from "lucide-react";
import { truncateAddress, timeAgo } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { TransactionHistoryEntry } from "@/types";

interface ActivityRowProps {
  tx: TransactionHistoryEntry;
  currentAddress: string;
  onClick?: () => void;
}

const NATIVE_TOKEN_ID = "0".repeat(64);

export function ActivityRow({ tx, currentAddress, onClick }: ActivityRowProps) {
  const isSent =
    tx.from?.toLowerCase() === currentAddress.toLowerCase();
  const counterparty = isSent ? tx.to : tx.from;
  const isNative = !tx.token_id || tx.token_id === NATIVE_TOKEN_ID || tx.token_id === `0x${NATIVE_TOKEN_ID}`;
  const symbol = isNative ? "NORN" : (tx.symbol || "TOKEN");

  return (
    <div
      className={cn(
        "flex items-center gap-3 py-2.5 transition-colors duration-150 hover:bg-muted/50 -mx-2 px-2 rounded-md",
        onClick && "cursor-pointer",
      )}
      onClick={onClick}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === "Enter" || e.key === " ") onClick(); } : undefined}
    >
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
          {isSent ? "Sent" : "Received"} {symbol}
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
          {tx.human_readable}
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
