import { formatNorn, formatAmount } from "@/lib/format";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
import { cn } from "@/lib/utils";

interface AmountDisplayProps {
  amount: string;
  symbol?: string;
  decimals?: number;
  className?: string;
}

export function AmountDisplay({
  amount,
  symbol = "NORN",
  decimals = 12,
  className,
}: AmountDisplayProps) {
  const formatted =
    symbol === "NORN" ? formatNorn(amount) : formatAmount(amount, decimals);
  const fullAmount = formatAmount(amount, decimals, decimals);

  return (
    <TooltipProvider delayDuration={300}>
      <Tooltip>
        <TooltipTrigger asChild>
          <span className={cn("font-mono text-sm tabular-nums", className)}>
            {formatted} <span className="text-muted-foreground">{symbol}</span>
          </span>
        </TooltipTrigger>
        <TooltipContent>
          <span className="font-mono text-xs">
            {fullAmount} {symbol}
          </span>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
