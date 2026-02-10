"use client";

import { useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";

interface RefreshIndicatorProps {
  intervalMs: number;
  label?: string;
}

export function RefreshIndicator({
  intervalMs,
  label = "Refreshing",
}: RefreshIndicatorProps) {
  const [secondsLeft, setSecondsLeft] = useState(
    Math.ceil(intervalMs / 1000)
  );

  useEffect(() => {
    const totalSeconds = Math.ceil(intervalMs / 1000);
    setSecondsLeft(totalSeconds);

    const timer = setInterval(() => {
      setSecondsLeft((prev) => {
        if (prev <= 1) return totalSeconds;
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [intervalMs]);

  return (
    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
      <RefreshCw className="h-3 w-3" />
      <span>
        {label} in {secondsLeft}s
      </span>
    </div>
  );
}
