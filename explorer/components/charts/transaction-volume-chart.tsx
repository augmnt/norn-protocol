"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { BlockChartPoint } from "@/hooks/use-chart-data";

interface TransactionVolumeChartProps {
  data: BlockChartPoint[];
}

function CustomTooltip({
  active,
  payload,
}: {
  active?: boolean;
  payload?: Array<{ payload: BlockChartPoint }>;
}) {
  if (!active || !payload?.length) return null;
  const d = payload[0].payload;
  return (
    <div className="rounded-lg border bg-popover px-3 py-2 text-xs shadow-md">
      <p className="font-medium text-foreground">Block {d.label}</p>
      <p className="text-muted-foreground">
        Transfers: <span className="font-mono text-foreground">{d.transferCount}</span>
      </p>
      <p className="text-muted-foreground">
        Total activity: <span className="font-mono text-foreground">{d.totalActivity}</span>
      </p>
    </div>
  );
}

export function TransactionVolumeChart({ data }: TransactionVolumeChartProps) {
  const display = data.slice(-30);

  if (display.length < 2) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">
            Transaction Volume
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

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium">
          Transaction Volume
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[200px] animate-fade-in">
          <ResponsiveContainer width="100%" height="100%" minWidth={0}>
            <BarChart
              data={display}
              margin={{ top: 4, right: 4, bottom: 0, left: -20 }}
            >
              <defs>
                <linearGradient id="barGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="hsl(210, 12%, 49%)" stopOpacity={0.8} />
                  <stop offset="100%" stopColor="hsl(210, 12%, 49%)" stopOpacity={0.3} />
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
                allowDecimals={false}
              />
              <Tooltip content={<CustomTooltip />} cursor={{ fill: "hsl(240, 3.7%, 12%)" }} />
              <Bar
                dataKey="transferCount"
                fill="url(#barGradient)"
                radius={[3, 3, 0, 0]}
                maxBarSize={24}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}
