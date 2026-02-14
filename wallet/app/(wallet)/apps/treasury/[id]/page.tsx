"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { TREASURY_LOOM_ID } from "@/lib/apps-config";
import { useTreasury } from "@/hooks/use-treasury";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount, formatTimestamp } from "@/lib/format";
import {
  ArrowLeft,
  Vault,
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
} from "lucide-react";
import { toast } from "sonner";
import type { Proposal, TreasuryConfig, ProposalStatus } from "@/lib/borsh-treasury";
import { cn } from "@/lib/utils";

const STATUS_VARIANT: Record<
  ProposalStatus,
  "norn" | "destructive" | "secondary" | "outline"
> = {
  Proposed: "norn",
  Executed: "secondary",
  Rejected: "destructive",
  Expired: "secondary",
};

export default function ProposalDetailPage() {
  const params = useParams();
  const proposalId = BigInt((params.id as string) || "0");
  const { activeAddress } = useWallet();
  const {
    getConfig,
    getProposal,
    approve,
    reject,
    revokeApproval,
    expireProposal,
    loading,
  } = useTreasury(TREASURY_LOOM_ID);

  const [proposal, setProposal] = useState<Proposal | null>(null);
  const [config, setConfig] = useState<TreasuryConfig | null>(null);
  const [fetching, setFetching] = useState(true);

  const fetchData = useCallback(async () => {
    setFetching(true);
    const [p, cfg] = await Promise.all([
      getProposal(proposalId),
      getConfig(),
    ]);
    setProposal(p);
    setConfig(cfg);
    setFetching(false);
  }, [getProposal, getConfig, proposalId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isOwner = config?.owners.some((o) => o.toLowerCase() === addr) ?? false;
  const now = Math.floor(Date.now() / 1000);
  const isExpired = proposal ? now >= Number(proposal.deadline) : false;

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

  const requiredApprovals = config?.requiredApprovals ?? 1n;
  const approvalPct =
    requiredApprovals > 0n
      ? Number((proposal.approvalCount * 100n) / requiredApprovals)
      : 0;

  return (
    <PageContainer
      title={`Proposal #${proposal.id.toString()}`}
      action={
        <Link href="/apps/treasury">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-2xl space-y-4">
        {/* Status + Approval Progress */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Vault className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Status</CardTitle>
              </div>
              <Badge variant={STATUS_VARIANT[proposal.status] ?? "secondary"}>
                {proposal.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-0 space-y-3">
            {/* Approval progress bar */}
            <div>
              <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                <span>Approvals</span>
                <span className="font-mono tabular-nums">
                  {proposal.approvalCount.toString()}/
                  {requiredApprovals.toString()}
                </span>
              </div>
              <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                <div
                  className="h-full rounded-full bg-norn transition-all"
                  style={{ width: `${Math.min(approvalPct, 100)}%` }}
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Proposal details */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Proposal Details</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Description</span>
                <span className="text-right max-w-[60%] truncate">
                  {proposal.description}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Proposer</span>
                <span className="font-mono text-xs">
                  {truncateAddress(proposal.proposer)}
                  {proposal.proposer.toLowerCase() === addr && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Recipient</span>
                <span className="font-mono text-xs">
                  {truncateAddress(proposal.to)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(proposal.amount.toString())} NORN
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span className="text-xs">
                  {proposal.createdAt > 0n
                    ? formatTimestamp(Number(proposal.createdAt))
                    : "\u2014"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Deadline</span>
                <span
                  className={cn("text-xs", isExpired && "text-destructive")}
                >
                  {formatTimestamp(Number(proposal.deadline))}
                  {isExpired && " (expired)"}
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
              {/* Owner: Approve (when Proposed and not expired) */}
              {isOwner &&
                proposal.status === "Proposed" &&
                !isExpired && (
                  <Button
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => approve(proposalId),
                        "Approval submitted"
                      )
                    }
                    disabled={loading}
                  >
                    {loading && (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    )}
                    <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                    Approve
                  </Button>
                )}

              {/* Owner: Reject (when Proposed) */}
              {isOwner && proposal.status === "Proposed" && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => reject(proposalId),
                      "Proposal rejected"
                    )
                  }
                  disabled={loading}
                >
                  <XCircle className="mr-1.5 h-3.5 w-3.5" />
                  Reject
                </Button>
              )}

              {/* Owner: Revoke own approval (when Proposed) */}
              {isOwner && proposal.status === "Proposed" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => revokeApproval(proposalId),
                      "Approval revoked"
                    )
                  }
                  disabled={loading}
                >
                  Revoke Approval
                </Button>
              )}

              {/* Anyone: Expire (when Proposed and expired) */}
              {isExpired && proposal.status === "Proposed" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => expireProposal(proposalId),
                      "Proposal expired"
                    )
                  }
                  disabled={loading}
                >
                  <Clock className="mr-1.5 h-3.5 w-3.5" />
                  Mark Expired
                </Button>
              )}

              {/* Terminal states */}
              {["Executed", "Rejected", "Expired"].includes(
                proposal.status
              ) && (
                <p className="text-xs text-muted-foreground py-1">
                  This proposal is finalized. No further actions available.
                </p>
              )}

              {/* Not an owner */}
              {!isOwner &&
                proposal.status === "Proposed" &&
                !isExpired && (
                  <p className="text-xs text-muted-foreground py-1">
                    Only treasury owners can approve or reject proposals.
                  </p>
                )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
