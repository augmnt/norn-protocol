"use client";

import { use } from "react";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { HashDisplay } from "@/components/ui/hash-display";
import { AddressDisplay } from "@/components/ui/address-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { DataTable } from "@/components/ui/data-table";
import { EmptyState } from "@/components/ui/empty-state";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { useTokenInfo } from "@/hooks/use-token-info";
import { useTokenEventsSubscription } from "@/hooks/use-subscriptions";
import { useRealtimeStore } from "@/stores/realtime-store";
import { formatAmount, formatTimestamp } from "@/lib/format";
import { Coins } from "lucide-react";
import type { TokenEvent } from "@/types";

const eventColumns = [
  {
    header: "Event",
    key: "event",
    render: (e: TokenEvent) => (
      <Badge
        variant={
          e.event_type === "created"
            ? "default"
            : e.event_type === "minted"
              ? "secondary"
              : "destructive"
        }
      >
        {e.event_type}
      </Badge>
    ),
  },
  {
    header: "Actor",
    key: "actor",
    render: (e: TokenEvent) => <AddressDisplay address={e.actor} />,
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (e: TokenEvent) => (
      <span className="font-mono text-sm tabular-nums">
        {e.amount ?? "—"}
      </span>
    ),
  },
  {
    header: "Block",
    key: "block",
    className: "text-right",
    render: (e: TokenEvent) => (
      <span className="font-mono text-sm tabular-nums text-norn">
        #{e.block_height}
      </span>
    ),
  },
];

export default function TokenDetailPage({
  params,
}: {
  params: Promise<{ tokenId: string }>;
}) {
  const { tokenId } = use(params);
  const { data: token, isLoading, error, refetch } = useTokenInfo(tokenId);
  useTokenEventsSubscription(tokenId);
  const tokenEvents = useRealtimeStore((s) =>
    s.tokenEvents.filter((e) => e.token_id === tokenId)
  );

  if (isLoading) {
    return (
      <PageContainer title="Token">
        <CardSkeleton />
      </PageContainer>
    );
  }

  if (error || !token) {
    return (
      <PageContainer title="Token">
        <ErrorState
          message="Token not found"
          retry={() => refetch()}
        />
      </PageContainer>
    );
  }

  const supplyPercent =
    BigInt(token.max_supply) > 0n
      ? Number(
          (BigInt(token.current_supply) * 100n) / BigInt(token.max_supply)
        )
      : 0;

  return (
    <PageContainer title={`${token.name} (${token.symbol})`}>
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Tokens", href: "/tokens" },
            { label: `${token.name} (${token.symbol})` },
          ]}
        />
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">
                Token Information
              </CardTitle>
              <Badge variant="outline" className="font-mono">
                {token.symbol}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            <dl className="grid gap-4 sm:grid-cols-2">
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Name
                </dt>
                <dd className="text-sm font-medium">{token.name}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Decimals
                </dt>
                <dd className="font-mono text-sm">{token.decimals}</dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Token ID
                </dt>
                <dd>
                  <HashDisplay hash={token.token_id} truncate={false} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Creator
                </dt>
                <dd>
                  <AddressDisplay address={token.creator} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Created
                </dt>
                <dd className="text-sm">
                  {formatTimestamp(token.created_at)}
                  <span className="ml-2">
                    <TimeAgo timestamp={token.created_at} />
                  </span>
                </dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        {/* Supply Bar */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">Supply</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Current Supply</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(token.current_supply, token.decimals)}{" "}
                  {token.symbol}
                </span>
              </div>
              <div className="h-2 w-full rounded-full bg-muted">
                <div
                  className="h-2 rounded-full bg-norn transition-all"
                  style={{ width: `${Math.min(supplyPercent, 100)}%` }}
                />
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Max Supply</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(token.max_supply, token.decimals)}{" "}
                  {token.symbol}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Live Events */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">
                Token Events
              </CardTitle>
              <LiveIndicator active label="Live" />
            </div>
          </CardHeader>
          <CardContent className="px-0">
            {tokenEvents.length === 0 ? (
              <EmptyState
                icon={Coins}
                title="No events yet"
                description="Live token events will appear here."
              />
            ) : (
              <DataTable
                columns={eventColumns}
                data={tokenEvents}
                keyExtractor={(e, i) =>
                  `${e.token_id}-${e.event_type}-${e.block_height}-${i}`
                }
              />
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
