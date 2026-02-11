"use client";

import { useMemo } from "react";

// Curated palette that works well on dark backgrounds
const COLORS = [
  "#6366f1", "#8b5cf6", "#a78bfa", "#c084fc",
  "#e879f9", "#f472b6", "#fb7185", "#f87171",
  "#fb923c", "#fbbf24", "#a3e635", "#4ade80",
  "#34d399", "#2dd4bf", "#22d3ee", "#38bdf8",
];

function hashBytes(address: string): number[] {
  const clean = address.replace(/^0x/i, "").toLowerCase();
  const bytes: number[] = [];
  for (let i = 0; i < clean.length && bytes.length < 20; i += 2) {
    bytes.push(parseInt(clean.slice(i, i + 2), 16) || 0);
  }
  return bytes;
}

interface IdenticonProps {
  address: string;
  size?: number;
  className?: string;
}

export function Identicon({ address, size = 32, className }: IdenticonProps) {
  const svg = useMemo(() => {
    const bytes = hashBytes(address);
    const color = COLORS[bytes[0] % COLORS.length];
    const bgColor = COLORS[(bytes[1] + 8) % COLORS.length];

    // Generate 5x5 symmetric grid (only need left half + center column = 15 bits)
    const cells: boolean[] = [];
    for (let i = 0; i < 15; i++) {
      cells.push(((bytes[2 + Math.floor(i / 8)] >> (i % 8)) & 1) === 1);
    }

    // Build rects for the 5x5 grid with horizontal symmetry
    const rects: { x: number; y: number }[] = [];
    for (let row = 0; row < 5; row++) {
      for (let col = 0; col < 3; col++) {
        const idx = row * 3 + col;
        if (cells[idx]) {
          rects.push({ x: col, y: row });
          if (col < 2) {
            rects.push({ x: 4 - col, y: row }); // mirror
          }
        }
      }
    }

    return { color, bgColor, rects };
  }, [address]);

  const cellSize = size / 5;

  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      className={className}
    >
      <defs>
        <clipPath id={`circle-${address.slice(-8)}`}>
          <circle cx={size / 2} cy={size / 2} r={size / 2} />
        </clipPath>
      </defs>
      <g clipPath={`url(#circle-${address.slice(-8)})`}>
        <rect width={size} height={size} fill={svg.bgColor} opacity={0.15} />
        {svg.rects.map((r, i) => (
          <rect
            key={i}
            x={r.x * cellSize}
            y={r.y * cellSize}
            width={cellSize}
            height={cellSize}
            fill={svg.color}
          />
        ))}
      </g>
    </svg>
  );
}
