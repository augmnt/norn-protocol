import { formatNorn } from "@/lib/format";
import { cn } from "@/lib/utils";

interface BalanceCardProps {
  balance: string;
  isLive?: boolean;
  className?: string;
}

export function BalanceCard({ balance, isLive, className }: BalanceCardProps) {
  return (
    <div className={cn("flex flex-col items-center gap-1 py-4", className)}>
      <div className="flex items-center gap-2">
        {isLive && (
          <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse-dot" />
        )}
        <span className="text-xs text-muted-foreground uppercase tracking-wider">
          Balance
        </span>
      </div>
      <div className="font-mono text-3xl font-bold tabular-nums animate-count-up">
        {formatNorn(balance)}
      </div>
      <span className="text-sm font-mono text-muted-foreground">NORN</span>
    </div>
  );
}
