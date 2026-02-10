"use client";

import { useHealth } from "@/hooks/use-health";
import { useWeaveState } from "@/hooks/use-weave-state";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { useRealtimeStore } from "@/stores/realtime-store";
import { formatNorn } from "@/lib/format";

export function NetworkInfo() {
  const { data: health } = useHealth();
  const { data: weave } = useWeaveState();
  const connected = useRealtimeStore((s) => s.connected);

  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-2 rounded-lg border px-4 py-2.5 text-xs text-muted-foreground">
      <LiveIndicator active={connected} />
      {health && (
        <>
          <span>
            Network:{" "}
            <span className="text-foreground font-medium">
              {health.network}
            </span>
          </span>
          <span>
            Version:{" "}
            <span className="text-foreground font-mono">{health.version}</span>
          </span>
          <span>
            Chain:{" "}
            <span className="text-foreground font-mono">{health.chain_id}</span>
          </span>
        </>
      )}
      {weave && (
        <span>
          Base Fee:{" "}
          <span className="text-foreground font-mono">
            {formatNorn(weave.base_fee)} NORN
          </span>
        </span>
      )}
    </div>
  );
}
