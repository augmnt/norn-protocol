"use client";

import { useState } from "react";
import { PageContainer } from "@/components/ui/page-container";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { FeedCard } from "@/components/feed-card";
import { useDiscoverFeed } from "@/hooks/use-discover-feed";
import { APPS } from "@/lib/apps-config";
import { cn } from "@/lib/utils";
import { Compass, Loader2 } from "lucide-react";

const INITIAL_COUNT = 12;
const LOAD_MORE_COUNT = 12;

const APP_TYPE_FILTERS = [
  { id: "all", label: "All" },
  ...APPS.map((app) => ({ id: app.id, label: app.name })),
];

export default function DiscoverPage() {
  const [filter, setFilter] = useState("all");
  const [visibleCount, setVisibleCount] = useState(INITIAL_COUNT);

  const {
    data: items,
    isLoading,
    error,
  } = useDiscoverFeed(filter === "all" ? undefined : filter);

  const visibleItems = items?.slice(0, visibleCount);
  const hasMore = items ? visibleCount < items.length : false;

  return (
    <PageContainer
      title="Discover"
      description="Explore deployed contracts and on-chain activity"
    >
      {/* Filter chips */}
      <div className="mb-6 flex flex-wrap gap-2">
        {APP_TYPE_FILTERS.map((f) => (
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
      ) : !visibleItems || visibleItems.length === 0 ? (
        <EmptyState
          icon={Compass}
          title="No contracts found"
          description={
            filter === "all"
              ? "No deployed contract instances yet."
              : `No ${APP_TYPE_FILTERS.find((f) => f.id === filter)?.label ?? filter} instances found.`
          }
        />
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {visibleItems.map((item) => (
              <FeedCard key={item.loomId} item={item} />
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
    </PageContainer>
  );
}
