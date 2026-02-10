import { cn } from "@/lib/utils";

interface LiveIndicatorProps {
  active?: boolean;
  label?: string;
  className?: string;
}

export function LiveIndicator({
  active = true,
  label = "Live",
  className,
}: LiveIndicatorProps) {
  return (
    <span className={cn("inline-flex items-center gap-1.5 text-xs", className)}>
      <span
        className={cn(
          "h-2 w-2 rounded-full",
          active
            ? "bg-green-500 animate-pulse-dot"
            : "bg-zinc-500"
        )}
      />
      <span className="text-muted-foreground">{label}</span>
    </span>
  );
}
