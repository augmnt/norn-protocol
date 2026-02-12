"use client";

import { Github } from "lucide-react";
import { config } from "@/lib/config";
import { useRealtimeStore } from "@/stores/realtime-store";
import { cn } from "@/lib/utils";

export function Footer() {
  const connectionState = useRealtimeStore((s) => s.connectionState);

  const dotColor =
    connectionState === "connected"
      ? "bg-green-500"
      : connectionState === "connecting"
        ? "bg-amber-500 animate-pulse"
        : "bg-zinc-500";
  const label =
    connectionState === "connected"
      ? config.chainName
      : connectionState === "connecting"
        ? "Connecting\u2026"
        : "Disconnected";

  return (
    <footer className="border-t">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <p className="text-xs text-muted-foreground">
          <span className="font-mono font-bold tracking-[-0.02em]">norn</span>{" "}
          explorer
        </p>

        <div className="flex items-center gap-4">
          <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
            <span
              className={cn(
                "h-1.5 w-1.5 rounded-full",
                dotColor
              )}
            />
            {label}
          </span>

          <a
            href="https://github.com/augmnt/norn-protocol"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground transition-colors hover:text-foreground"
            aria-label="GitHub repository"
          >
            <Github className="h-4 w-4" />
          </a>
        </div>
      </div>
    </footer>
  );
}
