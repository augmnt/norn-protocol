"use client";

import { use, useMemo } from "react";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { HashDisplay } from "@/components/ui/hash-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { LiveIndicator } from "@/components/ui/live-indicator";
import { DataTable } from "@/components/ui/data-table";
import { AddressDisplay } from "@/components/ui/address-display";
import { EmptyState } from "@/components/ui/empty-state";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { ContractInteract } from "@/components/contracts/contract-interact";
import { useLoomInfo } from "@/hooks/use-loom-info";
import { useLoomEventsSubscription } from "@/hooks/use-subscriptions";
import { useRealtimeStore } from "@/stores/realtime-store";
import { formatTimestamp, formatNumber } from "@/lib/format";
import { FileCode2 } from "lucide-react";
import type { LoomExecutionEvent } from "@/types";

const eventColumns = [
  {
    header: "Caller",
    key: "caller",
    render: (e: LoomExecutionEvent) => <AddressDisplay address={e.caller} />,
  },
  {
    header: "Gas Used",
    key: "gas",
    className: "text-right",
    render: (e: LoomExecutionEvent) => (
      <span className="font-mono text-sm tabular-nums">
        {formatNumber(e.gas_used)}
      </span>
    ),
  },
  {
    header: "Events",
    key: "events",
    className: "text-right",
    render: (e: LoomExecutionEvent) => (
      <Badge variant="outline">{e.events.length}</Badge>
    ),
  },
  {
    header: "Block",
    key: "block",
    className: "text-right",
    render: (e: LoomExecutionEvent) => (
      <span className="font-mono text-sm tabular-nums text-norn">
        #{e.block_height}
      </span>
    ),
  },
];

export default function ContractDetailPage({
  params,
}: {
  params: Promise<{ loomId: string }>;
}) {
  const { loomId } = use(params);
  const { data: loom, isLoading, error, refetch } = useLoomInfo(loomId);
  useLoomEventsSubscription(loomId);
  const allLoomEvents = useRealtimeStore((s) => s.loomEvents);
  const loomEvents = useMemo(
    () => allLoomEvents.filter((e) => e.loom_id === loomId),
    [allLoomEvents, loomId]
  );

  if (isLoading) {
    return (
      <PageContainer title="Contract">
        <CardSkeleton />
      </PageContainer>
    );
  }

  if (error || !loom) {
    return (
      <PageContainer title="Contract">
        <ErrorState message="Contract not found" retry={() => refetch()} />
      </PageContainer>
    );
  }

  return (
    <PageContainer title={loom.name}>
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Contracts", href: "/contracts" },
            { label: loom.name },
          ]}
        />
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">
                Contract Information
              </CardTitle>
              <Badge variant={loom.active ? "default" : "secondary"}>
                {loom.active ? "Active" : "Inactive"}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            <dl className="grid gap-4 sm:grid-cols-2">
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Name
                </dt>
                <dd className="text-sm font-medium">{loom.name}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Participants
                </dt>
                <dd className="font-mono text-sm">
                  {loom.participant_count}
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Loom ID
                </dt>
                <dd>
                  <HashDisplay hash={loom.loom_id} truncate={false} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Operator
                </dt>
                <dd>
                  <HashDisplay hash={loom.operator} chars={8} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Deployed
                </dt>
                <dd className="text-sm">
                  {formatTimestamp(loom.deployed_at)}
                  <span className="ml-2">
                    <TimeAgo timestamp={loom.deployed_at} />
                  </span>
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Bytecode
                </dt>
                <dd>
                  <Badge variant={loom.has_bytecode ? "outline" : "secondary"}>
                    {loom.has_bytecode ? "Deployed" : "None"}
                  </Badge>
                </dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        {loom.has_bytecode && <ContractInteract loomId={loomId} />}

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">
                Execution Events
              </CardTitle>
              <LiveIndicator active label="Live" />
            </div>
          </CardHeader>
          <CardContent className="px-0">
            {loomEvents.length === 0 ? (
              <EmptyState
                icon={FileCode2}
                title="No execution events yet"
                description="Live contract execution events will appear here."
              />
            ) : (
              <DataTable
                columns={eventColumns}
                data={loomEvents}
                keyExtractor={(e, i) =>
                  `${e.loom_id}-${e.block_height}-${e.caller}-${i}`
                }
              />
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
