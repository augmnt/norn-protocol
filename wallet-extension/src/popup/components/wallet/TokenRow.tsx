import { formatAmount } from "@/lib/format";

interface TokenRowProps {
  symbol: string;
  name: string;
  balance: string;
  decimals: number;
  onClick?: () => void;
}

export function TokenRow({ symbol, name, balance, decimals, onClick }: TokenRowProps) {
  return (
    <div
      className={`flex items-center gap-3 py-2.5 -mx-2 px-2 rounded-md transition-colors duration-150 hover:bg-muted/50${onClick ? " cursor-pointer" : ""}`}
      onClick={onClick}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === "Enter" || e.key === " ") onClick(); } : undefined}
    >
      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-norn/20 text-xs font-bold text-norn">
        {symbol.slice(0, 2)}
      </div>

      <div className="flex flex-1 flex-col">
        <span className="text-sm font-medium">{symbol}</span>
        <span className="text-xs text-muted-foreground">{name}</span>
      </div>

      <span className="font-mono text-sm font-medium tabular-nums">
        {formatAmount(balance, decimals)}
      </span>
    </div>
  );
}
