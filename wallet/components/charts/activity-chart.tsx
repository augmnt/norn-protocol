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
import { ArrowRightLeft } from "lucide-react";

export interface ActivityPoint {
  label: string;
  sent: number;
  received: number;
}

function CustomTooltip({
  active,
  payload,
}: {
  active?: boolean;
  payload?: Array<{ payload: ActivityPoint; dataKey: string; color: string }>;
}) {
  if (!active || !payload?.length) return null;
  const d = payload[0].payload;
  return (
    <div className="rounded-lg border bg-popover px-3 py-2 text-xs shadow-md">
      <p className="font-medium text-foreground">{d.label}</p>
      <div className="mt-1 space-y-0.5">
        <p className="text-muted-foreground">
          Sent: <span className="font-mono text-foreground">{d.sent}</span>
        </p>
        <p className="text-muted-foreground">
          Received:{" "}
          <span className="font-mono text-foreground">{d.received}</span>
        </p>
      </div>
    </div>
  );
}

interface ActivityChartProps {
  data: ActivityPoint[];
}

export function ActivityChart({ data }: ActivityChartProps) {
  if (data.length === 0) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <ArrowRightLeft className="h-3.5 w-3.5 text-muted-foreground" />
            Activity
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-[180px] items-center justify-center text-sm text-muted-foreground">
            Not enough data yet
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <ArrowRightLeft className="h-3.5 w-3.5 text-muted-foreground" />
          Activity
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[180px]">
          <ResponsiveContainer width="100%" height="100%" minWidth={0}>
            <BarChart
              data={data}
              margin={{ top: 4, right: 4, bottom: 0, left: -20 }}
            >
              <defs>
                <linearGradient
                  id="sentGradient"
                  x1="0"
                  y1="0"
                  x2="0"
                  y2="1"
                >
                  <stop
                    offset="0%"
                    stopColor="hsl(210, 12%, 49%)"
                    stopOpacity={0.8}
                  />
                  <stop
                    offset="100%"
                    stopColor="hsl(210, 12%, 49%)"
                    stopOpacity={0.3}
                  />
                </linearGradient>
                <linearGradient
                  id="receivedGradient"
                  x1="0"
                  y1="0"
                  x2="0"
                  y2="1"
                >
                  <stop
                    offset="0%"
                    stopColor="hsl(160, 30%, 45%)"
                    stopOpacity={0.8}
                  />
                  <stop
                    offset="100%"
                    stopColor="hsl(160, 30%, 45%)"
                    stopOpacity={0.3}
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
                allowDecimals={false}
              />
              <Tooltip
                content={<CustomTooltip />}
                cursor={{ fill: "hsl(240, 3.7%, 12%)" }}
              />
              <Bar
                dataKey="sent"
                fill="url(#sentGradient)"
                radius={[3, 3, 0, 0]}
                maxBarSize={16}
                stackId="activity"
              />
              <Bar
                dataKey="received"
                fill="url(#receivedGradient)"
                radius={[3, 3, 0, 0]}
                maxBarSize={16}
                stackId="activity"
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}
