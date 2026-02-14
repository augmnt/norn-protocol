"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { TREASURY_LOOM_ID } from "@/lib/apps-config";
import { useTreasury } from "@/hooks/use-treasury";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount } from "@/lib/format";
import {
  Plus,
  Vault,
  ArrowLeft,
  AlertCircle,
  Loader2,
  Download,
} from "lucide-react";
import type { Proposal, TreasuryConfig, ProposalStatus } from "@/lib/borsh-treasury";

const STATUS_VARIANT: Record<
  ProposalStatus,
  "norn" | "destructive" | "secondary" | "outline"
> = {
  Proposed: "norn",
  Executed: "secondary",
  Rejected: "destructive",
  Expired: "secondary",
};

function ProposalCard({
  proposal,
  requiredApprovals,
}: {
  proposal: Proposal;
  requiredApprovals: bigint;
}) {
  return (
    <Link href={`/apps/treasury/${proposal.id}`}>
      <Card className="transition-colors hover:border-norn/30">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Proposal #{proposal.id.toString()}
            </span>
            <Badge variant={STATUS_VARIANT[proposal.status] ?? "secondary"}>
              {proposal.status}
            </Badge>
          </div>
          <p className="mt-2 text-sm truncate">{proposal.description}</p>
          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <div className="flex items-center gap-3">
              <span>
                To:{" "}
                <span className="font-mono">
                  {truncateAddress(proposal.to)}
                </span>
              </span>
              <span>
                Approvals:{" "}
                <span className="font-mono tabular-nums">
                  {proposal.approvalCount.toString()}/
                  {requiredApprovals.toString()}
                </span>
              </span>
            </div>
            <span className="font-mono tabular-nums">
              {formatAmount(proposal.amount.toString())}
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function TreasuryDashboardPage() {
  const { activeAddress } = useWallet();
  const { getConfig, getProposal, getProposalCount, loading } =
    useTreasury(TREASURY_LOOM_ID);
  const [config, setConfig] = useState<TreasuryConfig | null>(null);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [fetching, setFetching] = useState(false);

  const fetchData = useCallback(async () => {
    if (!TREASURY_LOOM_ID) return;
    setFetching(true);
    try {
      const [cfg, count] = await Promise.all([getConfig(), getProposalCount()]);
      setConfig(cfg);

      const fetched: Proposal[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const p = await getProposal(i);
        if (p) fetched.push(p);
      }
      setProposals(fetched);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getConfig, getProposal, getProposalCount]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isOwner = config?.owners.some((o) => o.toLowerCase() === addr) ?? false;

  if (!TREASURY_LOOM_ID) {
    return (
      <PageContainer title="Multisig Treasury">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              Treasury contract not configured. Set{" "}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                NEXT_PUBLIC_TREASURY_LOOM_ID
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
      title="Multisig Treasury"
      description="Shared treasury with multi-signature approval"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href="/apps/treasury/deposit">
            <Button variant="outline" size="sm">
              <Download className="mr-1.5 h-3.5 w-3.5" />
              Deposit
            </Button>
          </Link>
          {isOwner && (
            <Link href="/apps/treasury/create">
              <Button size="sm">
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                New Proposal
              </Button>
            </Link>
          )}
        </div>
      }
    >
      {/* Config overview */}
      {config && (
        <Card className="mb-6">
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <Vault className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-sm">{config.name}</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-3">
              <div>
                <span className="text-xs text-muted-foreground">Owners</span>
                <div className="mt-1 space-y-1">
                  {config.owners.map((owner) => (
                    <div key={owner} className="flex items-center gap-1">
                      <span className="font-mono text-xs">
                        {truncateAddress(owner)}
                      </span>
                      {owner.toLowerCase() === addr && (
                        <Badge
                          variant="outline"
                          className="text-[9px] py-0"
                        >
                          You
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">Threshold</span>
                <p className="mt-1 font-mono tabular-nums">
                  {config.requiredApprovals.toString()} of{" "}
                  {config.owners.length}
                </p>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">
                  Proposals
                </span>
                <p className="mt-1 font-mono tabular-nums">
                  {proposals.length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Proposals */}
      <div className="space-y-3">
        {fetching || loading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : proposals.length === 0 ? (
          <EmptyState
            icon={Vault}
            title="No proposals yet"
            description="Create a proposal to request a transfer from the treasury."
            action={
              isOwner ? (
                <Link href="/apps/treasury/create">
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    New Proposal
                  </Button>
                </Link>
              ) : undefined
            }
          />
        ) : (
          proposals
            .slice()
            .reverse()
            .map((p) => (
              <ProposalCard
                key={p.id.toString()}
                proposal={p}
                requiredApprovals={config?.requiredApprovals ?? 1n}
              />
            ))
        )}
      </div>
    </PageContainer>
  );
}
