"use client";

import { cloneElement, useRef, useState, useEffect } from "react";

interface ChartProps {
  width?: number;
  height?: number;
}

/**
 * Measures the parent container with ResizeObserver and renders the
 * chart with explicit pixel dimensions, bypassing Recharts'
 * ResponsiveContainer which warns on negative dimensions.
 */
export function ChartContainer({
  children,
}: {
  children: React.ReactElement<ChartProps>;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(
    null,
  );

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      if (width > 0 && height > 0) {
        setSize({ width, height });
      }
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} style={{ width: "100%", height: "100%" }}>
      {size && cloneElement(children, { width: size.width, height: size.height })}
    </div>
  );
}
