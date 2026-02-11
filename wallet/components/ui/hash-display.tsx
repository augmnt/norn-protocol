"use client";

import { truncateHash } from "@/lib/format";
import { CopyButton } from "./copy-button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
import { cn } from "@/lib/utils";
import { ExternalLink } from "lucide-react";

interface HashDisplayProps {
  hash: string;
  truncate?: boolean;
  chars?: number;
  copy?: boolean;
  /** External URL (e.g. explorer link). Opens in a new tab. */
  href?: string;
  className?: string;
}

export function HashDisplay({
  hash,
  truncate = true,
  chars = 6,
  copy = true,
  href,
  className,
}: HashDisplayProps) {
  const display = truncate ? truncateHash(hash, chars) : hash;

  const inner = (
    <span className={cn("font-mono text-sm", className)}>
      {display}
    </span>
  );

  const hashElement = href ? (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 text-foreground hover:text-primary transition-colors underline-offset-4 hover:underline"
    >
      {inner}
      <ExternalLink className="h-3 w-3 text-muted-foreground" />
    </a>
  ) : (
    inner
  );

  return (
    <span className="inline-flex items-center gap-1">
      <TooltipProvider delayDuration={300}>
        <Tooltip>
          <TooltipTrigger asChild>
            {hashElement}
          </TooltipTrigger>
          <TooltipContent>
            <span className="font-mono text-xs break-all">{hash}</span>
            {href && (
              <span className="block text-xs text-muted-foreground mt-0.5">
                View on Explorer
              </span>
            )}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      {copy && <CopyButton value={hash} />}
    </span>
  );
}
