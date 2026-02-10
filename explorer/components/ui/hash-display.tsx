"use client";

import Link from "next/link";
import { truncateHash } from "@/lib/format";
import { CopyButton } from "./copy-button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
import { cn } from "@/lib/utils";

interface HashDisplayProps {
  hash: string;
  href?: string;
  truncate?: boolean;
  chars?: number;
  copy?: boolean;
  className?: string;
}

export function HashDisplay({
  hash,
  href,
  truncate = true,
  chars = 6,
  copy = true,
  className,
}: HashDisplayProps) {
  const display = truncate ? truncateHash(hash, chars) : hash;

  const hashElement = (
    <span
      className={cn(
        "font-mono text-sm",
        href && "text-norn hover:underline",
        className
      )}
    >
      {display}
    </span>
  );

  return (
    <span className="inline-flex items-center gap-1">
      <TooltipProvider delayDuration={300}>
        <Tooltip>
          <TooltipTrigger asChild>
            {href ? <Link href={href}>{hashElement}</Link> : hashElement}
          </TooltipTrigger>
          <TooltipContent>
            <span className="font-mono text-xs break-all">{hash}</span>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      {copy && <CopyButton value={hash} />}
    </span>
  );
}
