"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { useDiscoverFeed, type FeedItem } from "@/hooks/use-discover-feed";
import { useWallet } from "@/hooks/use-wallet";
import { APPS } from "@/lib/apps-config";
import { truncateHash, timeAgo } from "@/lib/format";
import { cn } from "@/lib/utils";
import {
  ArrowRight,
  Plus,
  Boxes,
  type LucideIcon,
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
  Waves,
} from "lucide-react";

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
  Waves,
};

interface AppInstanceListProps {
  appType: string;
}

export function AppInstanceList({ appType }: AppInstanceListProps) {
  const { data: allItems, isLoading, error } = useDiscoverFeed(appType);
  const { activeAccount } = useWallet();
  const [filter, setFilter] = useState<"all" | "mine">("all");

  const appConfig = APPS.find((a) => a.id === appType);
  const Icon = appConfig ? ICON_MAP[appConfig.icon] ?? Boxes : Boxes;
  const appName = appConfig?.name ?? appType;

  const items = useMemo(() => {
    if (!allItems) return [];
    if (filter === "mine") {
      const pubKey = activeAccount?.publicKeyHex?.toLowerCase();
      if (!pubKey) return [];
      return allItems.filter((i) => i.operator.toLowerCase() === pubKey);
    }
    return allItems;
  }, [allItems, filter, activeAccount]);

  return (
    <PageContainer
      title={appName}
      description={appConfig?.description}
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: appName },
      ]}
      action={
        <Link href={`/apps/${appType}/deploy`}>
          <Button size="sm">
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            Deploy New
          </Button>
        </Link>
      }
    >
      {/* Mine / All filter chips */}
      <div className="mb-6 flex flex-wrap gap-2">
        {(["all", "mine"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={cn(
              "rounded-full border px-3 py-1.5 text-xs font-medium transition-colors",
              filter === f
                ? "border-norn bg-norn/10 text-norn"
                : "border-border text-muted-foreground hover:text-foreground hover:bg-accent/50"
            )}
          >
            {f === "all" ? "All" : "Mine"}
          </button>
        ))}
      </div>

      {isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Card key={i}>
              <CardContent className="p-6">
                <Skeleton className="h-4 w-32 mb-3" />
                <Skeleton className="h-3 w-48 mb-2" />
                <Skeleton className="h-3 w-24" />
              </CardContent>
            </Card>
          ))}
        </div>
      ) : error ? (
        <Card>
          <CardContent className="p-6">
            <p className="text-sm text-destructive">
              Failed to load contracts: {error.message}
            </p>
          </CardContent>
        </Card>
      ) : items.length === 0 ? (
        <EmptyState
          icon={Icon}
          title={
            filter === "mine"
              ? `You haven't deployed any ${appName} contracts yet.`
              : `No ${appName} contracts found`
          }
          description={
            filter === "mine"
              ? "Deploy a new contract to get started."
              : "Deploy a new contract to get started."
          }
          action={
            <Link href={`/apps/${appType}/deploy`}>
              <Button size="sm">
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                Deploy New
              </Button>
            </Link>
          }
        />
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {items.map((item) => (
            <InstanceCard
              key={item.loomId}
              item={item}
              appType={appType}
              Icon={Icon}
              currentPubKey={activeAccount?.publicKeyHex}
            />
          ))}
        </div>
      )}
    </PageContainer>
  );
}

function InstanceCard({
  item,
  appType,
  Icon,
  currentPubKey,
}: {
  item: FeedItem;
  appType: string;
  Icon: LucideIcon;
  currentPubKey?: string;
}) {
  const isYours =
    currentPubKey &&
    item.operator.toLowerCase() === currentPubKey.toLowerCase();
  const summary = item.summary;

  return (
    <Link href={`/apps/${appType}/${item.loomId}`}>
      <Card className="group h-full transition-colors hover:border-norn/40">
        <CardContent className="p-6">
          <div className="flex items-start justify-between">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-norn/10">
              <Icon className="h-5 w-5 text-norn" />
            </div>
            <div className="flex items-center gap-1.5">
              {isYours && (
                <Badge variant="norn" className="text-[10px]">
                  Yours
                </Badge>
              )}
              {summary?.status && (
                <Badge
                  variant={summary.statusVariant ?? "secondary"}
                  className="text-[10px]"
                >
                  {summary.status}
                </Badge>
              )}
              <Badge
                variant={item.active ? "norn" : "secondary"}
                className="text-[10px]"
              >
                {item.active ? "Active" : "Inactive"}
              </Badge>
            </div>
          </div>

          <h3 className="mt-4 text-sm font-semibold truncate">
            {summary?.title ?? item.name}
          </h3>

          {summary?.subtitle && (
            <p className="mt-1 text-xs text-muted-foreground line-clamp-2">
              {summary.subtitle}
            </p>
          )}

          {/* Stats (first 2) */}
          {summary?.stats && summary.stats.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1">
              {summary.stats.slice(0, 2).map((stat) => (
                <div key={stat.label} className="text-xs">
                  <span className="text-muted-foreground">{stat.label}: </span>
                  <span className="font-mono tabular-nums">{stat.value}</span>
                </div>
              ))}
            </div>
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

          <div className="mt-4 flex items-center justify-between">
            <div className="space-y-0.5">
              <p className="text-[10px] text-muted-foreground font-mono">
                {truncateHash(item.loomId, 8)}
              </p>
              <p className="text-[10px] text-muted-foreground">
                by{" "}
                <span className="font-mono">
                  {truncateHash(item.operator, 6)}
                </span>
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
