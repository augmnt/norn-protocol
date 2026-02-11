"use client";

import { truncateAddress } from "@/lib/format";
import { getAddressLabel } from "@/lib/address-labels";
import { CopyButton } from "./copy-button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
import { cn } from "@/lib/utils";
import { ExternalLink } from "lucide-react";

interface AddressDisplayProps {
  address: string;
  copy?: boolean;
  full?: boolean;
  /** External URL (e.g. explorer link). Opens in a new tab. */
  href?: string;
  className?: string;
}

export function AddressDisplay({
  address,
  copy = true,
  full = false,
  href,
  className,
}: AddressDisplayProps) {
  const label = getAddressLabel(address);
  const display = full ? address : truncateAddress(address);

  const inner = (
    <span className={cn("font-mono text-sm", className)}>
      {label ? (
        <>
          <span className="font-sans text-xs text-muted-foreground mr-1.5">
            {label}
          </span>
          {display}
        </>
      ) : (
        display
      )}
    </span>
  );

  const addressElement = href ? (
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
            {addressElement}
          </TooltipTrigger>
          <TooltipContent>
            <span className="font-mono text-xs break-all">{address}</span>
            {label && (
              <span className="block text-xs text-muted-foreground">{label}</span>
            )}
            {href && (
              <span className="block text-xs text-muted-foreground mt-0.5">
                View on Explorer
              </span>
            )}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      {copy && <CopyButton value={address} />}
    </span>
  );
}
