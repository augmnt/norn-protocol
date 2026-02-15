"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useVesting } from "@/hooks/use-vesting";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount } from "@/lib/format";
import {
  Plus,
  Hourglass,
  ArrowLeft,
  Loader2,
} from "lucide-react";
import type { VestingSchedule } from "@/lib/borsh-vesting";

function getVestingPercent(schedule: VestingSchedule): number {
  const now = Math.floor(Date.now() / 1000);
  const start = Number(schedule.startTime);
  const cliff = start + Number(schedule.cliffDuration);
  const end = start + Number(schedule.totalDuration);
  if (now < start || now < cliff) return 0;
  if (now >= end) return 100;
  const elapsed = now - start;
  const total = Number(schedule.totalDuration);
  return total > 0 ? Math.min(Math.floor((elapsed / total) * 100), 100) : 0;
}

function ScheduleCard({ schedule, loomId }: { schedule: VestingSchedule; loomId: string }) {
  const vestPct = getVestingPercent(schedule);

  return (
    <Link href={`/apps/vesting/${loomId}/${schedule.id}`}>
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

          {/* Vesting progress bar */}
          <div className="mt-3">
            <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
              <span>Vested</span>
              <span className="font-mono tabular-nums">{vestPct}%</span>
            </div>
            <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
              <div
                className="h-full rounded-full bg-norn transition-all"
                style={{ width: `${vestPct}%` }}
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
              {formatAmount(schedule.claimedAmount.toString())} /{" "}
              {formatAmount(schedule.totalAmount.toString())}
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function VestingDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getSchedule, getScheduleCount, loading } =
    useVesting(loomId);
  const [schedules, setSchedules] = useState<VestingSchedule[]>([]);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchSchedules = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
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
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getSchedule, getScheduleCount, loomId]);

  useEffect(() => {
    fetchSchedules();
  }, [fetchSchedules]);

  useLoomRefresh(loomId, fetchSchedules);

  const addr = activeAddress?.toLowerCase() ?? "";
  const createdByMe = schedules.filter(
    (s) => s.creator.toLowerCase() === addr
  );
  const vestingToMe = schedules.filter(
    (s) => s.beneficiary.toLowerCase() === addr
  );

  return (
    <PageContainer
      title="Token Vesting"
      description="Time-locked token releases with cliff periods"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/vesting">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href={`/apps/vesting/${loomId}/create`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              New Schedule
            </Button>
          </Link>
        </div>
      }
    >
      <Tabs defaultValue="my_vesting" className="space-y-4">
        <TabsList>
          <TabsTrigger value="my_vesting">
            My Vesting ({vestingToMe.length})
          </TabsTrigger>
          <TabsTrigger value="created">
            Created by Me ({createdByMe.length})
          </TabsTrigger>
          <TabsTrigger value="all">All ({schedules.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="my_vesting" className="space-y-3">
          {fetching ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : vestingToMe.length === 0 ? (
            <EmptyState
              icon={Hourglass}
              title="No vesting schedules"
              description="Schedules where you are the beneficiary will appear here."
            />
          ) : (
            vestingToMe.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} loomId={loomId} />
            ))
          )}
        </TabsContent>

        <TabsContent value="created" className="space-y-3">
          {fetching ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : createdByMe.length === 0 ? (
            <EmptyState
              icon={Hourglass}
              title="No schedules created"
              description="Create a vesting schedule to get started."
              action={
                <Link href={`/apps/vesting/${loomId}/create`}>
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    New Schedule
                  </Button>
                </Link>
              }
            />
          ) : (
            createdByMe.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} loomId={loomId} />
            ))
          )}
        </TabsContent>

        <TabsContent value="all" className="space-y-3">
          {fetching ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : schedules.length === 0 ? (
            <EmptyState
              icon={Hourglass}
              title="No schedules found"
              description="Create a vesting schedule to get started."
            />
          ) : (
            schedules.map((s) => (
              <ScheduleCard key={s.id.toString()} schedule={s} loomId={loomId} />
            ))
          )}
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
