"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { FeedCard } from "@/components/feed-card";
import { useDiscoverFeed } from "@/hooks/use-discover-feed";
import { useWallet } from "@/hooks/use-wallet";
import { APPS } from "@/lib/apps-config";
import { cn } from "@/lib/utils";
import {
  Blocks,
  Search,
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
  ArrowRight,
  Plus,
  Terminal,
  type LucideIcon,
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

const INITIAL_COUNT = 12;
const LOAD_MORE_COUNT = 12;

export default function DiscoverPage() {
  const [filter, setFilter] = useState("all");
  const [search, setSearch] = useState("");
  const [visibleCount, setVisibleCount] = useState(INITIAL_COUNT);
  const { activeAccount } = useWallet();

  const { data: allItems, isLoading, error } = useDiscoverFeed();

  // Instance counts per app type
  const instanceCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const item of allItems ?? []) {
      counts.set(item.appType, (counts.get(item.appType) ?? 0) + 1);
    }
    return counts;
  }, [allItems]);

  // Filtered + searched items
  const filteredItems = useMemo(() => {
    if (!allItems) return [];
    let items = allItems;

    if (filter === "mine") {
      const pubKey = activeAccount?.publicKeyHex?.toLowerCase();
      if (pubKey) {
        items = items.filter((i) => i.operator.toLowerCase() === pubKey);
      } else {
        return [];
      }
    } else if (filter !== "all") {
      items = items.filter((i) => i.appType === filter);
    }

    if (search.trim()) {
      const q = search.trim().toLowerCase();
      items = items.filter(
        (i) =>
          (i.summary?.title ?? i.name).toLowerCase().includes(q) ||
          i.loomId.toLowerCase().includes(q) ||
          i.operator.toLowerCase().includes(q)
      );
    }

    return items;
  }, [allItems, filter, search, activeAccount]);

  const visibleItems = filteredItems.slice(0, visibleCount);
  const hasMore = visibleCount < filteredItems.length;

  const filterChips = [
    { id: "all", label: "All" },
    { id: "mine", label: "Mine" },
    ...APPS.map((app) => ({ id: app.id, label: app.name })),
  ];

  const totalDeployed = allItems?.length ?? 0;

  return (
    <PageContainer
      title="Apps"
      description="Browse app templates and deployed contracts"
      action={
        <Link href="/contracts">
          <Button variant="ghost" size="sm">
            <Terminal className="mr-1.5 h-3.5 w-3.5" />
            Dev Console
          </Button>
        </Link>
      }
    >
      {/* App Catalog section — always visible */}
      <div className="mb-8">
        <h2 className="mb-4 text-sm font-semibold text-muted-foreground uppercase tracking-wider">
          App Catalog
        </h2>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {APPS.map((app) => {
            const Icon = ICON_MAP[app.icon] ?? Blocks;
            const count = instanceCounts.get(app.id) ?? 0;
            return (
              <Link
                key={app.id}
                href={app.href}
                className="text-left"
              >
                <Card className="group h-full transition-colors hover:border-norn/40">
                  <CardContent className="p-6">
                    <div className="flex items-start justify-between">
                      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-norn/10">
                        <Icon className="h-5 w-5 text-norn" />
                      </div>
                      {count > 0 && (
                        <Badge variant="secondary" className="text-[10px]">
                          {count} deployed
                        </Badge>
                      )}
                    </div>
                    <h3 className="mt-4 text-sm font-semibold">{app.name}</h3>
                    <p className="mt-1.5 text-xs text-muted-foreground leading-relaxed">
                      {app.description}
                    </p>
                    <div className="mt-4 flex items-center gap-1 text-xs text-norn opacity-0 transition-opacity group-hover:opacity-100">
                      {count > 0 ? "View contracts" : "Deploy"}
                      <ArrowRight className="h-3 w-3" />
                    </div>
                  </CardContent>
                </Card>
              </Link>
            );
          })}
        </div>
      </div>

      {/* Deployed Contracts section */}
      <div>
        <h2 className="mb-4 text-sm font-semibold text-muted-foreground uppercase tracking-wider">
          Deployed Contracts{totalDeployed > 0 ? ` (${totalDeployed})` : ""}
        </h2>

        {/* Search bar */}
        <div className="relative mb-4">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search by name, loom ID, or creator..."
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setVisibleCount(INITIAL_COUNT);
            }}
            className="w-full rounded-lg border border-border bg-background pl-10 pr-4 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>

        {/* Filter chips */}
        <div className="mb-6 flex flex-wrap gap-2">
          {filterChips.map((f) => (
            <button
              key={f.id}
              onClick={() => {
                setFilter(f.id);
                setVisibleCount(INITIAL_COUNT);
              }}
              className={cn(
                "rounded-full border px-3 py-1.5 text-xs font-medium transition-colors",
                filter === f.id
                  ? "border-norn bg-norn/10 text-norn"
                  : "border-border text-muted-foreground hover:text-foreground hover:bg-accent/50"
              )}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* Loading state */}
        {isLoading ? (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Card key={i}>
                <CardContent className="p-6">
                  <div className="flex items-start justify-between">
                    <Skeleton className="h-10 w-10 rounded-lg" />
                    <Skeleton className="h-5 w-16 rounded-md" />
                  </div>
                  <Skeleton className="mt-4 h-4 w-32" />
                  <Skeleton className="mt-2 h-3 w-48" />
                  <Skeleton className="mt-3 h-3 w-24" />
                </CardContent>
              </Card>
            ))}
          </div>
        ) : error ? (
          <Card>
            <CardContent className="p-6">
              <p className="text-sm text-destructive">
                Failed to load feed: {error.message}
              </p>
            </CardContent>
          </Card>
        ) : visibleItems.length === 0 ? (
          <EmptyState
            icon={filter !== "all" && filter !== "mine" ? (ICON_MAP[APPS.find((a) => a.id === filter)?.icon ?? ""] ?? Blocks) : Blocks}
            title="No contracts found"
            description={
              filter === "mine"
                ? "You haven't deployed any contracts yet."
                : search.trim()
                  ? `No results for "${search.trim()}".`
                  : totalDeployed === 0
                    ? "No contracts deployed yet. Choose an app above to get started."
                    : `No ${filterChips.find((f) => f.id === filter)?.label ?? filter} contracts found.`
            }
            action={
              filter !== "all" && filter !== "mine" && !search.trim() ? (
                <Link href={`/apps/${filter}/deploy`}>
                  <Button size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    Deploy {filterChips.find((f) => f.id === filter)?.label ?? filter}
                  </Button>
                </Link>
              ) : undefined
            }
          />
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {visibleItems.map((item) => (
                <FeedCard
                  key={item.loomId}
                  item={item}
                  currentPubKey={activeAccount?.publicKeyHex}
                />
              ))}
            </div>

            {hasMore && (
              <div className="mt-6 flex justify-center">
                <Button
                  variant="outline"
                  onClick={() => setVisibleCount((c) => c + LOAD_MORE_COUNT)}
                >
                  Load more
                </Button>
              </div>
            )}
          </>
        )}
      </div>
    </PageContainer>
  );
}
