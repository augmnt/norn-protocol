"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useGovernance } from "@/hooks/use-governance";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatTimestamp } from "@/lib/format";
import {
  ArrowLeft,
  Vote,
  Loader2,
  ThumbsUp,
  ThumbsDown,
  CheckCircle2,
} from "lucide-react";
import { toast } from "sonner";
import type { GovProposal, ProposalStatus } from "@/lib/borsh-governance";

const STATUS_VARIANT: Record<
  ProposalStatus,
  "norn" | "secondary" | "destructive"
> = {
  Active: "norn",
  Passed: "secondary",
  Rejected: "destructive",
  Expired: "secondary",
};

export default function ProposalDetailPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const id = params.id as string;
  const proposalId = BigInt(id || "0");
  const { activeAddress } = useWallet();
  const { getProposal, getVote, vote, finalize, loading } =
    useGovernance(loomId);

  const [proposal, setProposal] = useState<GovProposal | null>(null);
  const [hasVoted, setHasVoted] = useState<boolean | null>(null);
  const [fetching, setFetching] = useState(true);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const p = await getProposal(proposalId);
      setProposal(p);

      if (activeAddress && p) {
        const v = await getVote(proposalId, activeAddress);
        setHasVoted(v);
      }
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getProposal, getVote, proposalId, activeAddress]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

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

  if (!proposal) {
    return (
      <PageContainer title="Proposal Not Found">
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Proposal #{proposalId.toString()} was not found.
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  const addr = activeAddress?.toLowerCase() ?? "";
  const isProposer = proposal.proposer.toLowerCase() === addr;
  const now = Math.floor(Date.now() / 1000);
  const endTime = Number(proposal.endTime);
  const isEnded = now >= endTime;
  const isActive = proposal.status === "Active";
  const canVote = isActive && !isEnded && hasVoted === null;
  const canFinalize = isActive && isEnded;

  const totalVotes = proposal.forVotes + proposal.againstVotes;
  const forPct =
    totalVotes > 0n ? Number((proposal.forVotes * 100n) / totalVotes) : 0;
  const againstPct = totalVotes > 0n ? 100 - forPct : 0;

  const timeRemaining = endTime - now;

  return (
    <PageContainer
      title={`Proposal #${proposal.id.toString()}`}
      action={
        <Link href={`/apps/governance/${loomId}`}>
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-2xl space-y-4">
        {/* Title and status */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Vote className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Proposal Details</CardTitle>
              </div>
              <Badge
                variant={STATUS_VARIANT[proposal.status] ?? "secondary"}
              >
                {proposal.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-0 space-y-3">
            <div>
              <h3 className="text-base font-semibold">{proposal.title}</h3>
              {proposal.description && (
                <p className="mt-2 text-sm text-muted-foreground whitespace-pre-wrap">
                  {proposal.description}
                </p>
              )}
            </div>

            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Proposer</span>
                <span className="font-mono text-xs">
                  {truncateAddress(proposal.proposer)}
                  {isProposer && (
                    <Badge
                      variant="outline"
                      className="ml-2 text-[9px] py-0"
                    >
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Start Time</span>
                <span className="text-xs">
                  {formatTimestamp(Number(proposal.startTime))}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">End Time</span>
                <span className="text-xs">
                  {formatTimestamp(endTime)}
                  {isEnded ? (
                    <span className="ml-1 text-muted-foreground">(ended)</span>
                  ) : (
                    <span className="ml-1 text-muted-foreground">
                      ({Math.floor(timeRemaining / 3600)}h{" "}
                      {Math.floor((timeRemaining % 3600) / 60)}m remaining)
                    </span>
                  )}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Vote Tally */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Vote Tally</CardTitle>
          </CardHeader>
          <CardContent className="pt-0 space-y-4">
            {/* For / Against bar */}
            <div>
              <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                <div className="flex items-center gap-1">
                  <ThumbsUp className="h-3 w-3" />
                  <span>
                    For: {proposal.forVotes.toString()} ({forPct}%)
                  </span>
                </div>
                <div className="flex items-center gap-1">
                  <span>
                    Against: {proposal.againstVotes.toString()} ({againstPct}
                    %)
                  </span>
                  <ThumbsDown className="h-3 w-3" />
                </div>
              </div>
              <div className="h-3 w-full rounded-full bg-muted overflow-hidden flex">
                {totalVotes > 0n ? (
                  <>
                    <div
                      className="h-full bg-norn transition-all"
                      style={{ width: `${forPct}%` }}
                    />
                    <div
                      className="h-full bg-destructive transition-all"
                      style={{ width: `${againstPct}%` }}
                    />
                  </>
                ) : (
                  <div className="h-full w-full bg-muted" />
                )}
              </div>
              {totalVotes === 0n && (
                <p className="mt-1.5 text-xs text-muted-foreground text-center">
                  No votes yet
                </p>
              )}
            </div>

            {/* Your vote status */}
            {hasVoted !== null && (
              <div className="flex items-center gap-2 rounded-lg border border-norn/20 bg-norn/5 p-3">
                <CheckCircle2 className="h-4 w-4 text-norn" />
                <p className="text-sm text-muted-foreground">
                  You voted{" "}
                  <span className="font-semibold text-foreground">
                    {hasVoted ? "For" : "Against"}
                  </span>{" "}
                  this proposal.
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Actions */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Actions</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="flex flex-wrap gap-2">
              {canVote && (
                <>
                  <Button
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => vote(proposalId, true),
                        "Voted For successfully"
                      )
                    }
                    disabled={loading}
                  >
                    {loading && (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    )}
                    <ThumbsUp className="mr-1.5 h-3.5 w-3.5" />
                    Vote For
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() =>
                      handleAction(
                        () => vote(proposalId, false),
                        "Voted Against successfully"
                      )
                    }
                    disabled={loading}
                  >
                    {loading && (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    )}
                    <ThumbsDown className="mr-1.5 h-3.5 w-3.5" />
                    Vote Against
                  </Button>
                </>
              )}

              {canFinalize && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => finalize(proposalId),
                      "Proposal finalized"
                    )
                  }
                  disabled={loading}
                >
                  {loading && (
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  )}
                  <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                  Finalize
                </Button>
              )}

              {hasVoted !== null && isActive && !isEnded && (
                <p className="text-xs text-muted-foreground py-1">
                  You have already voted on this proposal.
                </p>
              )}

              {proposal.status === "Passed" && (
                <p className="text-xs text-muted-foreground py-1">
                  This proposal has passed.
                </p>
              )}

              {proposal.status === "Rejected" && (
                <p className="text-xs text-muted-foreground py-1">
                  This proposal was rejected.
                </p>
              )}

              {proposal.status === "Expired" && (
                <p className="text-xs text-muted-foreground py-1">
                  This proposal has expired.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
