"use client";

import { Blocks, Layers, Users, Activity, Timer } from "lucide-react";
import { StatCard } from "@/components/ui/stat-card";
import { useWeaveState } from "@/hooks/use-weave-state";
import { useValidatorSet } from "@/hooks/use-validator-set";
import { useHealth } from "@/hooks/use-health";
import { useChartData } from "@/hooks/use-chart-data";
import { formatNumber } from "@/lib/format";

export function StatsBar() {
  const { data: weave, isLoading: weaveLoading } = useWeaveState();
  const { data: validators, isLoading: validatorsLoading } = useValidatorSet();
  const { data: health, isLoading: healthLoading } = useHealth();
  const {
    avgBlockTime,
    totalTxs,
    sparklineHeights,
    sparklineTxs,
    sparklineBlockTimes,
  } = useChartData();

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
        label="Threads"
        value={weave ? formatNumber(weave.thread_count) : "—"}
        icon={Layers}
        loading={weaveLoading}
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
        label="Avg Block Time"
        value={avgBlockTime !== null ? `${avgBlockTime}s` : "—"}
        icon={Timer}
        sparklineData={sparklineBlockTimes}
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
