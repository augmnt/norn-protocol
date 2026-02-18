"use client";

import { PieChart, Pie, Cell, Tooltip } from "recharts";
import { ChartContainer } from "@/components/ui/chart-container";
import type { BlockInfo } from "@/types";

interface BlockCompositionChartProps {
  block: BlockInfo;
}

const CATEGORIES = [
  { key: "transfer_count", label: "Transfers", color: "hsl(210, 12%, 49%)" },
  { key: "name_registration_count", label: "Names", color: "hsl(200, 15%, 42%)" },
  { key: "token_definition_count", label: "Token Defs", color: "hsl(170, 12%, 49%)" },
  { key: "token_mint_count", label: "Mints", color: "hsl(250, 12%, 49%)" },
  { key: "token_burn_count", label: "Burns", color: "hsl(0, 12%, 49%)" },
  { key: "loom_deploy_count", label: "Looms", color: "hsl(30, 12%, 49%)" },
  { key: "stake_operation_count", label: "Staking", color: "hsl(280, 12%, 49%)" },
] as const;

function CustomTooltip({
  active,
  payload,
}: {
  active?: boolean;
  payload?: Array<{ name: string; value: number }>;
}) {
  if (!active || !payload?.length) return null;
  const d = payload[0];
  return (
    <div className="rounded-lg border bg-popover px-3 py-2 text-xs shadow-md">
      <p className="text-foreground">
        {d.name}: <span className="font-mono font-medium">{d.value}</span>
      </p>
    </div>
  );
}

export function BlockCompositionChart({ block }: BlockCompositionChartProps) {
  const data = CATEGORIES.map((cat) => ({
    name: cat.label,
    value: block[cat.key] as number,
    color: cat.color,
  })).filter((d) => d.value > 0);

  if (data.length === 0) {
    return (
      <div className="flex h-[160px] items-center justify-center text-sm text-muted-foreground">
        Empty block
      </div>
    );
  }

  return (
    <div className="h-[160px]">
      <ChartContainer>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={40}
            outerRadius={65}
            paddingAngle={2}
            dataKey="value"
          >
            {data.map((entry, i) => (
              <Cell key={i} fill={entry.color} stroke="none" />
            ))}
          </Pie>
          <Tooltip content={<CustomTooltip />} />
        </PieChart>
      </ChartContainer>
    </div>
  );
}
