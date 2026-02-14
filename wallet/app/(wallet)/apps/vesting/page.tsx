"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { VESTING_LOOM_ID } from "@/lib/apps-config";
import { useVesting } from "@/hooks/use-vesting";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount } from "@/lib/format";
import {
  Plus,
  Timer,
  ArrowLeft,
  AlertCircle,
  Loader2,
} from "lucide-react";
import type { VestingSchedule } from "@/lib/borsh-vesting";

function ScheduleCard({ schedule }: { schedule: VestingSchedule }) {
  const pct =
    schedule.totalAmount > 0n
      ? Number((schedule.claimedAmount * 100n) / schedule.totalAmount)
      : 0;

  return (
    <Link href={`/apps/vesting/${schedule.id}`}>
      <Card className="transition-colors hover:border-norn/30">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Schedule #{schedule.id.toString()}
            </span>
            <div className="flex items-center gap-1.5">
              {schedule.revoked ? (
                <Badge variant="destructive">Revoked</Badge>
              ) : schedule.revocable ? (
                <Badge variant="outline">Revocable</Badge>
              ) : (
                <Badge variant="secondary">Locked</Badge>
              )}
            </div>
          </div>

          {/* Progress bar */}
          <div className="mt-3">
            <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
              <span>Claimed</span>
              <span className="font-mono tabular-nums">{pct}%</span>
            </div>
            <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
              <div
                className="h-full rounded-full bg-norn transition-all"
                style={{ width: `${Math.min(pct, 100)}%` }}
              />
            </div>
          </div>

          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <div className="flex items-center gap-3">
              <span>
                Creator:{" "}
                <span className="font-mono">
                  {truncateAddress(schedule.creator)}
                </span>
              </span>
              <span>
                Beneficiary:{" "}
                <span className="font-mono">
                  {truncateAddress(schedule.beneficiary)}
                </span>
              </span>
            </div>
            <span className="font-mono tabular-nums">
              {formatAmount(schedule.totalAmount.toString())}
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function VestingDashboardPage() {
  const { activeAddress } = useWallet();
  const { getSchedule, getScheduleCount, loading } =
    useVesting(VESTING_LOOM_ID);
  const [schedules, setSchedules] = useState<VestingSchedule[]>([]);
  const [fetching, setFetching] = useState(false);

  const fetchSchedules = useCallback(async () => {
    if (!VESTING_LOOM_ID) return;
    setFetching(true);
    try {
      const count = await getScheduleCount();
      const fetched: VestingSchedule[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const s = await getSchedule(i);
        if (s) fetched.push(s);
      }
      setSchedules(fetched);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getSchedule, getScheduleCount]);

  useEffect(() => {
    fetchSchedules();
  }, [fetchSchedules]);

  const addr = activeAddress?.toLowerCase() ?? "";
  const createdByMe = schedules.filter(
    (s) => s.creator.toLowerCase() === addr
  );
  const vestingToMe = schedules.filter(
    (s) => s.beneficiary.toLowerCase() === addr
  );

  if (!VESTING_LOOM_ID) {
    return (
      <PageContainer title="Token Vesting">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              Vesting contract not configured. Set{" "}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                NEXT_PUBLIC_VESTING_LOOM_ID
              </code>{" "}
              in your environment.
            </div>
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="Token Vesting"
      description="Time-locked token releases with cliff periods"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href="/apps/vesting/create">
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              New Schedule
            </Button>
          </Link>
        </div>
      }
    >
      <Tabs defaultValue="to_me" className="space-y-4">
        <TabsList>
          <TabsTrigger value="to_me">
            Vesting to Me ({vestingToMe.length})
          </TabsTrigger>
          <TabsTrigger value="created">
            Created by Me ({createdByMe.length})
          </TabsTrigger>
          <TabsTrigger value="all">All ({schedules.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="to_me" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : vestingToMe.length === 0 ? (
            <EmptyState
              icon={Timer}
              title="No vesting schedules"
              description="Schedules where you are the beneficiary will appear here."
            />
          ) : (
            vestingToMe.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} />
            ))
          )}
        </TabsContent>

        <TabsContent value="created" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : createdByMe.length === 0 ? (
            <EmptyState
              icon={Timer}
              title="No schedules created"
              description="Create a vesting schedule to get started."
              action={
                <Link href="/apps/vesting/create">
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    New Schedule
                  </Button>
                </Link>
              }
            />
          ) : (
            createdByMe.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} />
            ))
          )}
        </TabsContent>

        <TabsContent value="all" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : schedules.length === 0 ? (
            <EmptyState
              icon={Timer}
              title="No schedules found"
              description="Create a vesting schedule to get started."
            />
          ) : (
            schedules.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} />
            ))
          )}
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
