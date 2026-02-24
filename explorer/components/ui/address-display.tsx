"use client";

import Link from "next/link";
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

interface AddressDisplayProps {
  address: string;
  link?: boolean;
  copy?: boolean;
  full?: boolean;
  className?: string;
}

export function AddressDisplay({
  address,
  link = true,
  copy = true,
  full = false,
  className,
}: AddressDisplayProps) {
  const label = getAddressLabel(address);
  const display = full ? address : truncateAddress(address);

  const addressElement = (
    <span
      className={cn(
        "font-mono text-sm",
        full && "break-all",
        link && "text-norn hover:underline",
        className
      )}
    >
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

  return (
    <span className="inline-flex items-center gap-1">
      <TooltipProvider delayDuration={300}>
        <Tooltip>
          <TooltipTrigger asChild>
            {link ? (
              <Link href={`/address/${address}`}>{addressElement}</Link>
            ) : (
              addressElement
            )}
          </TooltipTrigger>
          <TooltipContent>
            <span className="font-mono text-xs">{address}</span>
            {label && (
              <span className="block text-xs text-muted-foreground">{label}</span>
            )}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      {copy && <CopyButton value={address} />}
    </span>
  );
}
