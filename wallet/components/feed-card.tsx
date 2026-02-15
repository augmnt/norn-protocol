"use client";

import Link from "next/link";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { APPS } from "@/lib/apps-config";
import { truncateHash, timeAgo } from "@/lib/format";
import {
  ArrowRight,
  ShieldCheck,
  Vault,
  Hourglass,
  Rocket,
  GitFork,
  HandCoins,
  Vote,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
  Boxes,
  type LucideIcon,
} from "lucide-react";
import type { FeedItem } from "@/hooks/use-discover-feed";

const ICON_MAP: Record<string, LucideIcon> = {
  ShieldCheck,
  Vault,
  Hourglass,
  Rocket,
  GitFork,
  HandCoins,
  Vote,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
};

export function FeedCard({ item }: { item: FeedItem }) {
  const appConfig = APPS.find((a) => a.id === item.appType);
  const Icon = appConfig ? ICON_MAP[appConfig.icon] ?? Boxes : Boxes;
  const summary = item.summary;

  return (
    <Link href={`/apps/${item.appType}/${item.loomId}`}>
      <Card className="group h-full transition-colors hover:border-norn/40">
        <CardContent className="p-6">
          {/* Header: icon + app type badge */}
          <div className="flex items-start justify-between">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-norn/10">
              <Icon className="h-5 w-5 text-norn" />
            </div>
            <div className="flex items-center gap-1.5">
              {summary?.status && (
                <Badge
                  variant={summary.statusVariant ?? "secondary"}
                  className="text-[10px]"
                >
                  {summary.status}
                </Badge>
              )}
              <Badge variant="outline" className="text-[10px]">
                {appConfig?.name ?? item.appType}
              </Badge>
            </div>
          </div>

          {/* Title */}
          <h3 className="mt-4 text-sm font-semibold truncate">
            {summary?.title ?? item.name}
          </h3>

          {/* Subtitle */}
          {summary?.subtitle && (
            <p className="mt-1 text-xs text-muted-foreground line-clamp-2">
              {summary.subtitle}
            </p>
          )}

          {/* Progress bar */}
          {summary?.progress !== undefined && (
            <div className="mt-3">
              <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
                <div
                  className="h-full rounded-full bg-norn transition-all"
                  style={{ width: `${summary.progress}%` }}
                />
              </div>
              <p className="mt-1 text-right text-[10px] text-muted-foreground font-mono tabular-nums">
                {summary.progress}%
              </p>
            </div>
          )}

          {/* Stats */}
          {summary?.stats && summary.stats.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1">
              {summary.stats.map((stat) => (
                <div key={stat.label} className="text-xs">
                  <span className="text-muted-foreground">{stat.label}: </span>
                  <span className="font-mono tabular-nums">{stat.value}</span>
                </div>
              ))}
            </div>
          )}

          {/* Footer: loom ID + deployed time */}
          <div className="mt-4 flex items-center justify-between">
            <div className="space-y-0.5">
              <p className="text-[10px] text-muted-foreground font-mono">
                {truncateHash(item.loomId, 8)}
              </p>
              {item.deployedAt > 0 && (
                <p className="text-[10px] text-muted-foreground">
                  {timeAgo(item.deployedAt)}
                </p>
              )}
            </div>
            <div className="flex items-center gap-1 text-xs text-norn opacity-0 transition-opacity group-hover:opacity-100">
              Open
              <ArrowRight className="h-3 w-3" />
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
