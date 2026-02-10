import { cn } from "@/lib/utils";
import { truncateAddress } from "@/lib/format";
import { CopyButton } from "./copy-button";

interface AddressDisplayProps {
  address: string;
  truncate?: boolean;
  copyable?: boolean;
  className?: string;
}

export function AddressDisplay({
  address,
  truncate = true,
  copyable = true,
  className,
}: AddressDisplayProps) {
  return (
    <span className={cn("inline-flex items-center gap-1", className)}>
      <span className="font-mono text-sm">
        {truncate ? truncateAddress(address) : address}
      </span>
      {copyable && <CopyButton value={address} />}
    </span>
  );
}
