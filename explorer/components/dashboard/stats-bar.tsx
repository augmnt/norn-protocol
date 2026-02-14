"use client";

import { useState, useEffect } from "react";
import { Blocks, Clock, Users, Activity, Zap } from "lucide-react";
import { StatCard } from "@/components/ui/stat-card";
import { useWeaveState } from "@/hooks/use-weave-state";
import { useValidatorSet } from "@/hooks/use-validator-set";
import { useHealth } from "@/hooks/use-health";
import { useChartData } from "@/hooks/use-chart-data";
import { useLatestBlock } from "@/hooks/use-block";
import { useRealtimeStore } from "@/stores/realtime-store";
import { formatNumber } from "@/lib/format";

function useLastBlockAgo() {
  const wsBlock = useRealtimeStore((s) => s.latestBlock);
  const { data: polledBlock } = useLatestBlock();
  // Prefer WS block (freshest), fall back to polled block.
  const latestBlock = wsBlock ?? polledBlock ?? null;
  const [ago, setAgo] = useState<string>("—");

  useEffect(() => {
    if (!latestBlock) return;

    const update = () => {
      const diff = Math.max(0, Math.floor(Date.now() / 1000) - latestBlock.timestamp);
      if (diff < 1) setAgo("just now");
      else if (diff < 60) setAgo(`${diff}s ago`);
      else if (diff < 3600) setAgo(`${Math.floor(diff / 60)}m ago`);
      else setAgo(`${Math.floor(diff / 3600)}h ago`);
    };

    update();
    const id = setInterval(update, 1000);
    return () => clearInterval(id);
  }, [latestBlock]);

  return ago;
}

function formatProductionTime(us: number): string {
  if (us < 1000) return `${us}µs`;
  const ms = us / 1000;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  const secs = ms / 1000;
  return `${secs.toFixed(1)}s`;
}

function formatBlockTime(seconds: number): string {
  if (seconds < 1) return "< 1s";
  if (Number.isInteger(seconds)) return `${seconds}s`;
  return `${seconds.toFixed(1)}s`;
}

export function StatsBar() {
  const { data: weave, isLoading: weaveLoading } = useWeaveState();
  const { data: validators, isLoading: validatorsLoading } = useValidatorSet();
  const { data: health, isLoading: healthLoading } = useHealth();
  const blockTimeTarget = health?.block_time_target;
  const lastProductionUs = health?.last_block_production_us;
  const {
    blockProductionTime,
    totalTxs,
    sparklineHeights,
    sparklineTxs,
    sparklineBlockTimes,
  } = useChartData();
  const lastBlockAgo = useLastBlockAgo();

  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-5">
      <StatCard
        label="Block Height"
        value={weave ? weave.height : 0}
        icon={Blocks}
        loading={weaveLoading}
        animateNumber={!!weave}
        sparklineData={sparklineHeights}
      />
      <StatCard
        label="Block Speed"
        value={
          lastProductionUs != null
            ? formatProductionTime(lastProductionUs)
            : blockProductionTime !== null
              ? formatBlockTime(blockProductionTime)
              : blockTimeTarget
                ? `~${blockTimeTarget}s`
                : "~3s"
        }
        icon={Zap}
      />
      <StatCard
        label="Validators"
        value={
          validators ? formatNumber(validators.validators.length) : "—"
        }
        icon={Users}
        loading={validatorsLoading}
      />
      <StatCard
        label="Last Block"
        value={lastBlockAgo}
        icon={Clock}
      />
      <StatCard
        label="Transactions"
        value={totalTxs > 0 ? totalTxs : 0}
        icon={Activity}
        loading={healthLoading}
        animateNumber={totalTxs > 0}
        sparklineData={sparklineTxs}
        suffix={totalTxs > 0 ? "recent" : undefined}
      />
    </div>
  );
}
