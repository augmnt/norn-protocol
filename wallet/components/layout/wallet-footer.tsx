"use client";

import { useRealtimeStore } from "@/stores/realtime-store";
import { useNetwork } from "@/hooks/use-network";
import { useHealth } from "@/hooks/use-health";
import { explorerBlockUrl } from "@/lib/explorer";
import { cn } from "@/lib/utils";

export function WalletFooter() {
  const connected = useRealtimeStore((s) => s.connected);
  const latestBlock = useRealtimeStore((s) => s.latestBlock);
  const { network } = useNetwork();
  const { data: health } = useHealth();
  const blockHeight = latestBlock?.height ?? health?.height ?? 0;

  return (
    <footer className="hidden md:block border-t">
      <div className="flex h-14 items-center justify-between px-4 sm:px-6 lg:px-8">
        <p className="text-xs text-muted-foreground">
          <span className="font-mono font-bold tracking-[-0.02em]">norn</span>{" "}
          wallet
        </p>

        <div className="flex items-center gap-4">
          {blockHeight > 0 && (
            <a
              href={explorerBlockUrl(blockHeight)}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-muted-foreground font-mono transition-colors hover:text-foreground"
            >
              Block #{blockHeight.toLocaleString()}
            </a>
          )}

          <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
            <span
              className={cn(
                "h-1.5 w-1.5 rounded-full",
                connected ? "bg-green-500" : "bg-zinc-500"
              )}
            />
            {connected ? network.name : "Disconnected"}
          </span>
        </div>
      </div>
    </footer>
  );
}
