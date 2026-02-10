"use client";

import {
  AreaChart,
  Area,
  ResponsiveContainer,
} from "recharts";

interface SparklineProps {
  data: number[];
  color?: string;
  height?: number;
}

export function Sparkline({
  data,
  color = "hsl(210, 12%, 49%)",
  height = 32,
}: SparklineProps) {
  if (data.length < 2) return null;

  const chartData = data.map((v, i) => ({ v, i }));
  const gradientId = `sparkline-${Math.random().toString(36).slice(2, 8)}`;

  return (
    <div style={{ width: "100%", height }} className="opacity-30">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={chartData}
          margin={{ top: 0, right: 0, bottom: 0, left: 0 }}
        >
          <defs>
            <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity={0.3} />
              <stop offset="100%" stopColor={color} stopOpacity={0} />
            </linearGradient>
          </defs>
          <Area
            type="monotone"
            dataKey="v"
            stroke={color}
            strokeWidth={1.5}
            fill={`url(#${gradientId})`}
            dot={false}
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
