"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useVesting } from "@/hooks/use-vesting";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount, formatTimestamp } from "@/lib/format";
import {
  ArrowLeft,
  Hourglass,
  Loader2,
  Download,
  XCircle,
} from "lucide-react";
import { toast } from "sonner";
import type { VestingSchedule } from "@/lib/borsh-vesting";

export default function ScheduleDetailPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const scheduleId = BigInt((params.id as string) || "0");
  const { activeAddress } = useWallet();
  const { getSchedule, getClaimable, claim, revoke, loading } =
    useVesting(loomId);

  const [schedule, setSchedule] = useState<VestingSchedule | null>(null);
  const [claimable, setClaimable] = useState<bigint>(0n);
  const [fetching, setFetching] = useState(true);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const [s, c] = await Promise.all([
        getSchedule(scheduleId),
        getClaimable(scheduleId),
      ]);
      setSchedule(s);
      setClaimable(c);
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getSchedule, getClaimable, scheduleId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isBeneficiary = schedule?.beneficiary.toLowerCase() === addr;
  const isCreator = schedule?.creator.toLowerCase() === addr;

  const handleAction = async (
    action: () => Promise<unknown>,
    successMsg: string
  ) => {
    try {
      await action();
      toast.success(successMsg);
      fetchData();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Action failed");
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

  if (!schedule) {
    return (
      <PageContainer title="Schedule Not Found">
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Schedule #{scheduleId.toString()} was not found.
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  const cliffEnd = Number(schedule.startTime) + Number(schedule.cliffDuration);
  const vestingEnd =
    Number(schedule.startTime) + Number(schedule.totalDuration);
  const now = Math.floor(Date.now() / 1000);
  const cliffPassed = now >= cliffEnd;
  const fullyVested = now >= vestingEnd;

  // Time-based vesting percentage
  const vestedPct = (() => {
    const start = Number(schedule.startTime);
    if (now < start || now < cliffEnd) return 0;
    if (now >= vestingEnd) return 100;
    const elapsed = now - start;
    const total = Number(schedule.totalDuration);
    return total > 0 ? Math.min(Math.floor((elapsed / total) * 100), 100) : 0;
  })();

  return (
    <PageContainer
      title={`Schedule #${schedule.id.toString()}`}
      action={
        <Link href={`/apps/vesting/${loomId}`}>
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-2xl space-y-4">
        {/* Status + Vesting Progress */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Hourglass className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Vesting Progress</CardTitle>
              </div>
              <div className="flex items-center gap-1.5">
                {schedule.revoked ? (
                  <Badge variant="destructive">Revoked</Badge>
                ) : fullyVested ? (
                  <Badge variant="secondary">Fully Vested</Badge>
                ) : cliffPassed ? (
                  <Badge variant="norn">Vesting</Badge>
                ) : (
                  <Badge variant="outline">Cliff</Badge>
                )}
                {schedule.revocable && !schedule.revoked && (
                  <Badge variant="outline">Revocable</Badge>
                )}
                {!schedule.revocable && (
                  <Badge variant="secondary">Locked</Badge>
                )}
              </div>
            </div>
          </CardHeader>
          <CardContent className="pt-0 space-y-3">
            {/* Vesting progress bar */}
            <div>
              <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                <span>Vested</span>
                <span className="font-mono tabular-nums">{vestedPct}%</span>
              </div>
              <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                <div
                  className="h-full rounded-full bg-norn transition-all"
                  style={{ width: `${vestedPct}%` }}
                />
              </div>
              <div className="flex items-center justify-between text-[10px] text-muted-foreground mt-1">
                <span>
                  Claimed:{" "}
                  <span className="font-mono tabular-nums">
                    {formatAmount(schedule.claimedAmount.toString())}
                  </span>
                </span>
                <span>
                  Total:{" "}
                  <span className="font-mono tabular-nums">
                    {formatAmount(schedule.totalAmount.toString())}
                  </span>
                </span>
              </div>
            </div>

            {/* Claimable highlight */}
            {claimable > 0n && !schedule.revoked && (
              <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-3">
                <div>
                  <p className="text-xs text-muted-foreground">
                    Available to claim
                  </p>
                  <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                    {formatAmount(claimable.toString())}
                  </p>
                </div>
                {isBeneficiary && (
                  <Button
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => claim(scheduleId),
                        "Tokens claimed successfully"
                      )
                    }
                    disabled={loading}
                  >
                    {loading && (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    )}
                    <Download className="mr-1.5 h-3.5 w-3.5" />
                    Claim
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Schedule Details */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Schedule Details</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Creator</span>
                <span className="font-mono text-xs">
                  {truncateAddress(schedule.creator)}
                  {isCreator && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Beneficiary</span>
                <span className="font-mono text-xs">
                  {truncateAddress(schedule.beneficiary)}
                  {isBeneficiary && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token</span>
                <span className="font-mono text-xs truncate max-w-48">
                  {schedule.tokenId}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(schedule.totalAmount.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Claimed Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(schedule.claimedAmount.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Claimable Now</span>
                <span className="font-mono tabular-nums text-norn">
                  {formatAmount(claimable.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Start Time</span>
                <span className="text-xs">
                  {formatTimestamp(Number(schedule.startTime))}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Cliff End</span>
                <span className="text-xs">
                  {formatTimestamp(cliffEnd)}
                  {cliffPassed && (
                    <span className="ml-1 text-muted-foreground">(passed)</span>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Vesting End</span>
                <span className="text-xs">
                  {formatTimestamp(vestingEnd)}
                  {fullyVested && (
                    <span className="ml-1 text-muted-foreground">
                      (complete)
                    </span>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span className="text-xs">
                  {schedule.createdAt > 0n
                    ? formatTimestamp(Number(schedule.createdAt))
                    : "\u2014"}
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
              {/* Beneficiary: Claim */}
              {isBeneficiary && claimable > 0n && !schedule.revoked && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => claim(scheduleId),
                      "Tokens claimed successfully"
                    )
                  }
                  disabled={loading}
                >
                  {loading && (
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  )}
                  <Download className="mr-1.5 h-3.5 w-3.5" />
                  Claim {formatAmount(claimable.toString())}
                </Button>
              )}

              {/* Creator: Revoke (if revocable and not already revoked) */}
              {isCreator &&
                schedule.revocable &&
                !schedule.revoked && (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => revoke(scheduleId),
                        "Schedule revoked"
                      )
                    }
                    disabled={loading}
                  >
                    <XCircle className="mr-1.5 h-3.5 w-3.5" />
                    Revoke
                  </Button>
                )}

              {/* Revoked state */}
              {schedule.revoked && (
                <p className="text-xs text-muted-foreground py-1">
                  This schedule has been revoked. No further actions available.
                </p>
              )}

              {/* Fully claimed */}
              {!schedule.revoked &&
                schedule.claimedAmount >= schedule.totalAmount && (
                  <p className="text-xs text-muted-foreground py-1">
                    All tokens have been claimed.
                  </p>
                )}

              {/* Not a party */}
              {!isBeneficiary && !isCreator && !schedule.revoked && (
                <p className="text-xs text-muted-foreground py-1">
                  You are not a party to this vesting schedule.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
