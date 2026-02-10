import { cn } from "@/lib/utils";
import { formatNorn } from "@/lib/format";

interface AmountDisplayProps {
  amount: string;
  symbol?: string;
  size?: "sm" | "md" | "lg";
  className?: string;
}

const sizeClasses = {
  sm: "text-sm",
  md: "text-lg",
  lg: "text-2xl",
};

export function AmountDisplay({
  amount,
  symbol = "NORN",
  size = "md",
  className,
}: AmountDisplayProps) {
  return (
    <span className={cn("font-mono font-semibold", sizeClasses[size], className)}>
      {formatNorn(amount)} <span className="text-muted-foreground">{symbol}</span>
    </span>
  );
}
