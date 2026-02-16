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
import { useTimelock } from "@/hooks/use-timelock";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, truncateHash, formatAmount, formatTimestamp } from "@/lib/format";
import {
  Plus,
  Clock,
  Loader2,
} from "lucide-react";
import type { LockInfo } from "@/lib/borsh-timelock";

function LockCard({ lock, loomId }: { lock: LockInfo; loomId: string }) {
  const now = Math.floor(Date.now() / 1000);
  const unlocked = now >= Number(lock.unlockTime);

  return (
    <Link href={`/apps/timelock/${loomId}/${lock.id.toString()}`}>
      <Card className="transition-colors hover:border-norn/30">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Lock #{lock.id.toString()}
            </span>
            <div className="flex items-center gap-1.5">
              {lock.withdrawn ? (
                <Badge variant="secondary">Withdrawn</Badge>
              ) : unlocked ? (
                <Badge variant="norn">Unlocked</Badge>
              ) : (
                <Badge variant="outline">Locked</Badge>
              )}
            </div>
          </div>

          <div className="mt-3 flex items-center justify-between">
            <div>
              <p className="text-xs text-muted-foreground">Amount</p>
              <p className="mt-0.5 font-mono text-sm tabular-nums">
                {formatAmount(lock.amount.toString())}
              </p>
            </div>
            <div className="text-right">
              <p className="text-xs text-muted-foreground">Unlock Date</p>
              <p className="mt-0.5 text-xs">
                {formatTimestamp(Number(lock.unlockTime))}
              </p>
            </div>
          </div>

          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <span>
              Owner:{" "}
              <span className="font-mono">
                {truncateAddress(lock.owner)}
              </span>
            </span>
            <span className="font-mono text-[10px] truncate max-w-24">
              {lock.tokenId.slice(0, 12)}...
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function TimelockDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getLock, getLockCount, loading } = useTimelock(loomId);
  const [locks, setLocks] = useState<LockInfo[]>([]);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchLocks = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const count = await getLockCount();
      const fetched: LockInfo[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const l = await getLock(i);
        if (l) fetched.push(l);
      }
      setLocks(fetched);
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getLock, getLockCount, loomId]);

  useEffect(() => {
    fetchLocks();
  }, [fetchLocks]);

  useLoomRefresh(loomId, fetchLocks);

  const addr = activeAddress?.toLowerCase() ?? "";
  const myLocks = locks.filter((l) => l.owner.toLowerCase() === addr);

  return (
    <PageContainer
      title="Time-locked Vault"
      description="Deposit tokens with a future unlock date"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Time-locked Vault", href: "/apps/timelock" },
        { label: truncateHash(loomId, 8) },
      ]}
      action={
        <Link href={`/apps/timelock/${loomId}/create`}>
          <Button size="sm">
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            New Lock
          </Button>
        </Link>
      }
    >
      <Tabs defaultValue="mine" className="space-y-4">
        <TabsList>
          <TabsTrigger value="mine">
            My Locks ({myLocks.length})
          </TabsTrigger>
          <TabsTrigger value="all">All ({locks.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="mine" className="space-y-3">
          {fetching ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : myLocks.length === 0 ? (
            <EmptyState
              icon={Clock}
              title="No locks found"
              description="Create a time-locked deposit to get started."
              action={
                <Link href={`/apps/timelock/${loomId}/create`}>
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    New Lock
                  </Button>
                </Link>
              }
            />
          ) : (
            myLocks
              .slice()
              .reverse()
              .map((l) => <LockCard key={l.id.toString()} lock={l} loomId={loomId} />)
          )}
        </TabsContent>

        <TabsContent value="all" className="space-y-3">
          {fetching ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : locks.length === 0 ? (
            <EmptyState
              icon={Clock}
              title="No locks found"
              description="Create a time-locked deposit to get started."
            />
          ) : (
            locks
              .slice()
              .reverse()
              .map((l) => <LockCard key={l.id.toString()} lock={l} loomId={loomId} />)
          )}
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
