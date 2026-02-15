"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useTimelock } from "@/hooks/use-timelock";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount, formatTimestamp } from "@/lib/format";
import {
  ArrowLeft,
  Clock,
  Loader2,
  Download,
} from "lucide-react";
import { toast } from "sonner";
import type { LockInfo } from "@/lib/borsh-timelock";

function formatCountdown(secondsLeft: number): string {
  if (secondsLeft <= 0) return "Unlocked";
  const days = Math.floor(secondsLeft / 86400);
  const hours = Math.floor((secondsLeft % 86400) / 3600);
  const mins = Math.floor((secondsLeft % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h ${mins}m`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

export default function LockDetailPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const lockId = BigInt((params.id as string) || "0");
  const { activeAddress } = useWallet();
  const { getLock, withdraw, loading } = useTimelock(loomId);

  const [lock, setLock] = useState<LockInfo | null>(null);
  const [fetching, setFetching] = useState(true);
  const hasLoadedRef = useRef(false);
  const [now, setNow] = useState(Math.floor(Date.now() / 1000));

  const fetchData = useCallback(async () => {
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const l = await getLock(lockId);
      setLock(l);
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getLock, lockId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  // Update countdown every minute
  useEffect(() => {
    const interval = setInterval(() => {
      setNow(Math.floor(Date.now() / 1000));
    }, 60_000);
    return () => clearInterval(interval);
  }, []);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isOwner = lock?.owner.toLowerCase() === addr;
  const unlocked = lock ? now >= Number(lock.unlockTime) : false;
  const secondsLeft = lock ? Number(lock.unlockTime) - now : 0;

  const handleWithdraw = async () => {
    try {
      await withdraw(lockId);
      toast.success("Tokens withdrawn successfully");
      fetchData();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to withdraw");
    }
  };

  if (fetching) {
    return (
      <PageContainer>
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      </PageContainer>
    );
  }

  if (!lock) {
    return (
      <PageContainer title="Lock Not Found">
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Lock #{lockId.toString()} was not found.
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title={`Lock #${lock.id.toString()}`}
      action={
        <Link href={`/apps/timelock/${loomId}`}>
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-2xl space-y-4">
        {/* Status + Countdown */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Lock Status</CardTitle>
              </div>
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
          </CardHeader>
          <CardContent className="pt-0 space-y-3">
            {/* Amount */}
            <div className="flex items-center justify-between rounded-lg border border-border p-3">
              <div>
                <p className="text-xs text-muted-foreground">Locked Amount</p>
                <p className="mt-0.5 font-mono text-2xl tabular-nums">
                  {formatAmount(lock.amount.toString())}
                </p>
              </div>
              {!lock.withdrawn && !unlocked && (
                <div className="text-right">
                  <p className="text-xs text-muted-foreground">Time Remaining</p>
                  <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                    {formatCountdown(secondsLeft)}
                  </p>
                </div>
              )}
            </div>

            {/* Withdraw action */}
            {isOwner && unlocked && !lock.withdrawn && (
              <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-3">
                <div>
                  <p className="text-xs text-muted-foreground">
                    Ready to withdraw
                  </p>
                  <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                    {formatAmount(lock.amount.toString())}
                  </p>
                </div>
                <Button
                  size="sm"
                  onClick={handleWithdraw}
                  disabled={loading}
                >
                  {loading && (
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  )}
                  <Download className="mr-1.5 h-3.5 w-3.5" />
                  Withdraw
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Lock Details */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Lock Details</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Owner</span>
                <span className="font-mono text-xs">
                  {truncateAddress(lock.owner)}
                  {isOwner && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(lock.amount.toString())} NORN
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token ID</span>
                <span className="font-mono text-xs truncate max-w-48">
                  {lock.tokenId}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Unlock Time</span>
                <span className="text-xs">
                  {formatTimestamp(Number(lock.unlockTime))}
                  {unlocked && (
                    <span className="ml-1 text-muted-foreground">(passed)</span>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span className="text-xs">
                  {lock.createdAt > 0n
                    ? formatTimestamp(Number(lock.createdAt))
                    : "\u2014"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Withdrawn</span>
                <span className="text-xs">
                  {lock.withdrawn ? "Yes" : "No"}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Actions */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Actions</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="flex flex-wrap gap-2">
              {isOwner && unlocked && !lock.withdrawn && (
                <Button
                  size="sm"
                  onClick={handleWithdraw}
                  disabled={loading}
                >
                  {loading && (
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  )}
                  <Download className="mr-1.5 h-3.5 w-3.5" />
                  Withdraw
                </Button>
              )}

              {lock.withdrawn && (
                <p className="text-xs text-muted-foreground py-1">
                  Tokens have been withdrawn from this lock.
                </p>
              )}

              {!unlocked && !lock.withdrawn && (
                <p className="text-xs text-muted-foreground py-1">
                  Tokens are still locked. Unlock in{" "}
                  <span className="font-mono tabular-nums">
                    {formatCountdown(secondsLeft)}
                  </span>
                  .
                </p>
              )}

              {!isOwner && unlocked && !lock.withdrawn && (
                <p className="text-xs text-muted-foreground py-1">
                  You are not the owner of this lock.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
