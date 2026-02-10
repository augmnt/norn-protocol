"use client";

import { use } from "react";
import Link from "next/link";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { BlockCompositionChart } from "@/components/charts/block-composition-chart";
import { useBlock } from "@/hooks/use-block";
import { formatNumber, formatTimestamp } from "@/lib/format";

export default function BlockDetailPage({
  params,
}: {
  params: Promise<{ height: string }>;
}) {
  const { height: heightStr } = use(params);
  const height = parseInt(heightStr, 10);
  const { data: block, isLoading, error, refetch } = useBlock(height);

  if (isLoading) {
    return (
      <PageContainer title={`Block #${formatNumber(height)}`}>
        <CardSkeleton />
      </PageContainer>
    );
  }

  if (error || !block) {
    return (
      <PageContainer title={`Block #${formatNumber(height)}`}>
        <ErrorState
          message="Block not found or failed to load"
          retry={() => refetch()}
        />
      </PageContainer>
    );
  }

  const activityCounts = [
    { label: "Transfers", count: block.transfer_count },
    { label: "Names", count: block.name_registration_count },
    { label: "Token Defs", count: block.token_definition_count },
    { label: "Mints", count: block.token_mint_count },
    { label: "Burns", count: block.token_burn_count },
    { label: "Looms", count: block.loom_deploy_count },
    { label: "Stake Ops", count: block.stake_operation_count },
  ];

  return (
    <PageContainer
      title={`Block #${formatNumber(height)}`}
      action={
        <div className="flex items-center gap-2">
          <Button variant="outline" size="icon" asChild disabled={height <= 0}>
            <Link href={height > 0 ? `/block/${height - 1}` : "#"}>
              <ChevronLeft className="h-4 w-4" />
            </Link>
          </Button>
          <Button variant="outline" size="icon" asChild>
            <Link href={`/block/${height + 1}`}>
              <ChevronRight className="h-4 w-4" />
            </Link>
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Blocks", href: "/blocks" },
            { label: `Block #${formatNumber(height)}` },
          ]}
        />
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">
              Block Information
            </CardTitle>
          </CardHeader>
          <CardContent>
            <dl className="grid gap-4 sm:grid-cols-2">
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Height
                </dt>
                <dd className="font-mono text-sm tabular-nums">
                  {formatNumber(block.height)}
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Timestamp
                </dt>
                <dd className="text-sm">
                  {formatTimestamp(block.timestamp)}
                  <span className="ml-2">
                    <TimeAgo timestamp={block.timestamp} />
                  </span>
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Hash
                </dt>
                <dd>
                  <HashDisplay hash={block.hash} truncate={false} />
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Previous Hash
                </dt>
                <dd>
                  <HashDisplay
                    hash={block.prev_hash}
                    href={height > 0 ? `/block/${height - 1}` : undefined}
                    truncate={false}
                  />
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  State Root
                </dt>
                <dd>
                  <HashDisplay hash={block.state_root} truncate={false} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Proposer
                </dt>
                <dd>
                  <HashDisplay hash={block.proposer} chars={8} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Commitments
                </dt>
                <dd className="font-mono text-sm">
                  {block.commitment_count}
                </dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        <div className="grid gap-6 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">
                Block Activity
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {activityCounts.map((item) => (
                  <Badge
                    key={item.label}
                    variant={item.count > 0 ? "default" : "secondary"}
                  >
                    {item.label}: {item.count}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">
                Composition
              </CardTitle>
            </CardHeader>
            <CardContent>
              <BlockCompositionChart block={block} />
            </CardContent>
          </Card>
        </div>
      </div>
    </PageContainer>
  );
}
