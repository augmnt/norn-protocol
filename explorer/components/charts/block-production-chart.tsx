"use client";

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { BlockChartPoint } from "@/hooks/use-chart-data";

interface BlockProductionChartProps {
  data: BlockChartPoint[];
}

function formatProductionTime(ms: number): string {
  if (ms < 1) {
    const us = Math.round(ms * 1000);
    return `${us}µs`;
  }
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function CustomTooltip({
  active,
  payload,
  useProduction,
}: {
  active?: boolean;
  payload?: Array<{ payload: BlockChartPoint }>;
  useProduction: boolean;
}) {
  if (!active || !payload?.length) return null;
  const d = payload[0].payload;
  return (
    <div className="rounded-lg border bg-popover px-3 py-2 text-xs shadow-md">
      <p className="font-medium text-foreground">Block {d.label}</p>
      {useProduction && d.productionMs !== null && (
        <p className="text-muted-foreground">
          Production:{" "}
          <span className="font-mono text-foreground">
            {formatProductionTime(d.productionMs)}
          </span>
        </p>
      )}
      {!useProduction && d.cappedBlockTime !== null && (
        <p className="text-muted-foreground">
          Block gap:{" "}
          <span className="font-mono text-foreground">
            {d.cappedBlockTime}s
          </span>
        </p>
      )}
    </div>
  );
}

export function BlockProductionChart({ data }: BlockProductionChartProps) {
  const display = data.slice(-30);
  const hasProductionData = display.some((d) => d.productionMs !== null);

  if (display.length < 2) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">
            Block Production Time
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-[200px] items-center justify-center text-sm text-muted-foreground">
            Waiting for blocks...
          </div>
        </CardContent>
      </Card>
    );
  }

  // When real production data is available, show µs/ms scale.
  // Otherwise fall back to capped block gaps in seconds.
  const dataKey = hasProductionData ? "productionMs" : "cappedBlockTime";

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium">
          {hasProductionData ? "Block Production Time" : "Block Time"}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[200px] animate-fade-in">
          <ResponsiveContainer width="100%" height="100%" minWidth={0}>
            <AreaChart
              data={display}
              margin={{ top: 4, right: 4, bottom: 0, left: -20 }}
            >
              <defs>
                <linearGradient
                  id="blockTimeGradient"
                  x1="0"
                  y1="0"
                  x2="0"
                  y2="1"
                >
                  <stop
                    offset="0%"
                    stopColor="hsl(210, 12%, 49%)"
                    stopOpacity={0.25}
                  />
                  <stop
                    offset="100%"
                    stopColor="hsl(210, 12%, 49%)"
                    stopOpacity={0}
                  />
                </linearGradient>
              </defs>
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="hsl(240, 3.7%, 15%)"
                vertical={false}
              />
              <XAxis
                dataKey="label"
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 10, fill: "hsl(240, 5%, 50%)" }}
                interval="preserveStartEnd"
              />
              <YAxis
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 10, fill: "hsl(240, 5%, 50%)" }}
                tickFormatter={(v) =>
                  hasProductionData ? formatProductionTime(v) : `${v}s`
                }
                domain={[0, "auto"]}
              />
              <Tooltip
                content={
                  <CustomTooltip useProduction={hasProductionData} />
                }
              />
              <Area
                type="monotone"
                dataKey={dataKey}
                stroke="hsl(210, 12%, 49%)"
                strokeWidth={2}
                fill="url(#blockTimeGradient)"
                dot={false}
                activeDot={{
                  r: 4,
                  fill: "hsl(210, 12%, 49%)",
                  stroke: "hsl(240, 10%, 3.9%)",
                  strokeWidth: 2,
                }}
                connectNulls
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}
