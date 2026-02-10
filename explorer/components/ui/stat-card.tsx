"use client";

import { Card } from "./card";
import { Skeleton } from "./skeleton";
import { Sparkline } from "./sparkline";
import { AnimatedNumber } from "./animated-number";
import { cn } from "@/lib/utils";
import type { LucideIcon } from "lucide-react";

interface StatCardProps {
  label: string;
  value: string | number;
  icon?: LucideIcon;
  loading?: boolean;
  className?: string;
  sparklineData?: number[];
  animateNumber?: boolean;
  suffix?: string;
}

export function StatCard({
  label,
  value,
  icon: Icon,
  loading = false,
  className,
  sparklineData,
  animateNumber = false,
  suffix,
}: StatCardProps) {
  return (
    <Card className={cn("p-4 overflow-hidden relative", className)}>
      <div className="flex items-center justify-between relative z-10 drop-shadow-[0_1px_2px_hsl(var(--card))]">
        <p className="text-xs text-muted-foreground uppercase tracking-wider">
          {label}
        </p>
        {Icon && <Icon className="h-4 w-4 text-muted-foreground" />}
      </div>
      <div className="mt-2 relative z-10 drop-shadow-[0_1px_2px_hsl(var(--card))]">
        {loading ? (
          <Skeleton className="h-7 w-24" />
        ) : (
          <p className="text-2xl font-semibold tabular-nums animate-count-up">
            {animateNumber && typeof value === "number" ? (
              <>
                <AnimatedNumber value={value} />
                {suffix && <span className="text-sm font-normal text-muted-foreground ml-1">{suffix}</span>}
              </>
            ) : (
              <>
                {typeof value === "number" ? value.toLocaleString() : value}
                {suffix && <span className="text-sm font-normal text-muted-foreground ml-1">{suffix}</span>}
              </>
            )}
          </p>
        )}
      </div>
      {sparklineData && sparklineData.length >= 2 && (
        <div className="absolute bottom-0 left-0 right-0">
          <Sparkline data={sparklineData} height={36} />
        </div>
      )}
    </Card>
  );
}
