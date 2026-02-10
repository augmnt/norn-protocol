"use client";

import { useEffect, useState } from "react";
import { timeAgo, formatTimestamp } from "@/lib/format";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
import { cn } from "@/lib/utils";

interface TimeAgoProps {
  timestamp: number;
  className?: string;
}

export function TimeAgo({ timestamp, className }: TimeAgoProps) {
  const [relative, setRelative] = useState(
    timestamp > 0 ? timeAgo(timestamp) : "\u2014"
  );

  useEffect(() => {
    if (timestamp <= 0) {
      setRelative("\u2014");
      return;
    }
    setRelative(timeAgo(timestamp));
    const interval = setInterval(() => {
      setRelative(timeAgo(timestamp));
    }, 10_000);
    return () => clearInterval(interval);
  }, [timestamp]);

  if (timestamp <= 0) {
    return (
      <span className={cn("text-sm text-muted-foreground tabular-nums", className)}>
        {"\u2014"}
      </span>
    );
  }

  return (
    <TooltipProvider delayDuration={300}>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            className={cn("text-sm text-muted-foreground tabular-nums", className)}
          >
            {relative}
          </span>
        </TooltipTrigger>
        <TooltipContent>
          <span className="text-xs">{formatTimestamp(timestamp)}</span>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
