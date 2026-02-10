import { ChevronDown } from "lucide-react";
import { truncateAddress } from "@/lib/format";
import { cn } from "@/lib/utils";
import { useNavigationStore } from "@/stores/navigation-store";
import { CopyButton } from "../ui/copy-button";

interface AccountPillProps {
  name: string;
  address: string;
  className?: string;
}

export function AccountPill({ name, address, className }: AccountPillProps) {
  const navigate = useNavigationStore((s) => s.navigate);

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <button
        onClick={() => navigate("accounts")}
        className="flex items-center gap-1.5 rounded-full border px-3 py-1.5 text-sm transition-colors duration-150 hover:bg-accent"
      >
        <div className="flex h-5 w-5 items-center justify-center rounded-full bg-norn text-[10px] font-bold text-norn-foreground">
          {name.charAt(0).toUpperCase()}
        </div>
        <span className="font-medium">{name}</span>
        <span className="font-mono text-xs text-muted-foreground">
          {truncateAddress(address)}
        </span>
        <ChevronDown className="h-3 w-3 text-muted-foreground" />
      </button>
      <CopyButton value={address} />
    </div>
  );
}
