"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { FormButton } from "@/components/ui/form-button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { EmptyState } from "@/components/ui/empty-state";
import { useGovernance } from "@/hooks/use-governance";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatTimestamp } from "@/lib/format";
import {
  Vote,
  Plus,
  ArrowLeft,
  Loader2,
} from "lucide-react";
import { toast } from "sonner";
import type { GovConfig, GovProposal, ProposalStatus } from "@/lib/borsh-governance";

const STATUS_VARIANT: Record<
  ProposalStatus,
  "norn" | "secondary" | "destructive"
> = {
  Active: "norn",
  Passed: "secondary",
  Rejected: "destructive",
  Expired: "secondary",
};

function ProposalCard({ proposal, loomId }: { proposal: GovProposal; loomId: string }) {
  const now = Math.floor(Date.now() / 1000);
  const endTime = Number(proposal.endTime);
  const timeRemaining = endTime - now;
  const isEnded = timeRemaining <= 0;
  const totalVotes = proposal.forVotes + proposal.againstVotes;
  const forPct =
    totalVotes > 0n ? Number((proposal.forVotes * 100n) / totalVotes) : 0;

  return (
    <Link href={`/apps/governance/${loomId}/${proposal.id.toString()}`}>
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
          <p className="mt-2 text-sm truncate">{proposal.title}</p>

          {/* Vote bar */}
          {totalVotes > 0n && (
            <div className="mt-3">
              <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
                <span>For: {forPct}%</span>
                <span>Against: {100 - forPct}%</span>
              </div>
              <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden flex">
                <div
                  className="h-full bg-norn transition-all"
                  style={{ width: `${forPct}%` }}
                />
                <div
                  className="h-full bg-destructive transition-all"
                  style={{ width: `${100 - forPct}%` }}
                />
              </div>
            </div>
          )}

          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <span>
              By:{" "}
              <span className="font-mono">
                {truncateAddress(proposal.proposer)}
              </span>
            </span>
            <span>
              {proposal.status === "Active" && !isEnded
                ? `${Math.floor(timeRemaining / 3600)}h ${Math.floor(
                    (timeRemaining % 3600) / 60
                  )}m remaining`
                : formatTimestamp(endTime)}
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useGovernance(loomId);
  const [name, setName] = useState("");
  const [votingPeriodHours, setVotingPeriodHours] = useState("72");
  const [quorum, setQuorum] = useState("1");

  const canSubmit =
    name.trim().length > 0 &&
    parseFloat(votingPeriodHours) > 0 &&
    parseInt(quorum) >= 1;

  const disabledReason = !name.trim()
    ? "Enter a governance name"
    : parseFloat(votingPeriodHours) <= 0
      ? "Voting period must be greater than 0"
      : parseInt(quorum) < 1
        ? "Quorum must be at least 1"
        : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const votingPeriod = BigInt(
        Math.floor(parseFloat(votingPeriodHours) * 3600)
      );
      const quorumBig = BigInt(parseInt(quorum));

      await initialize(name.trim(), votingPeriod, quorumBig);
      toast.success("Governance initialized successfully");
      onSuccess();
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Initialization failed"
      );
    }
  };

  return (
    <Card className="max-w-lg">
      <CardHeader className="pb-4">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
            <Vote className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Initialize Governance</CardTitle>
            <CardDescription>
              Set up on-chain governance with voting periods and quorum
              requirements. This can only be done once.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            Governance Name
          </Label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Community DAO"
            maxLength={64}
            className="text-sm"
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Voting Period (hours)
            </Label>
            <Input
              type="number"
              value={votingPeriodHours}
              onChange={(e) => setVotingPeriodHours(e.target.value)}
              placeholder="72"
              min="1"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Quorum (votes)
            </Label>
            <Input
              type="number"
              value={quorum}
              onChange={(e) => setQuorum(e.target.value)}
              placeholder="1"
              min="1"
              className="font-mono text-sm tabular-nums"
            />
          </div>
        </div>

        <FormButton
          onClick={handleSubmit}
          disabled={!canSubmit || loading}
          disabledReason={disabledReason}
          className="w-full"
        >
          {loading ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Vote className="mr-2 h-3.5 w-3.5" />
          )}
          Initialize Governance
        </FormButton>
      </CardContent>
    </Card>
  );
}

export default function GovernanceDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getConfig, getProposal, getProposalCount, loading } =
    useGovernance(loomId);
  const [config, setConfig] = useState<GovConfig | null>(null);
  const [proposals, setProposals] = useState<GovProposal[]>([]);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const [cfg, count] = await Promise.all([
        getConfig(),
        getProposalCount(),
      ]);
      setConfig(cfg);

      const fetched: GovProposal[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const p = await getProposal(i);
        if (p) fetched.push(p);
      }
      setProposals(fetched);
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getConfig, getProposal, getProposalCount, loomId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isCreator = config?.creator.toLowerCase() === addr;

  // Not yet initialized
  if (!fetching && !config && loomId) {
    return (
      <PageContainer
        title="DAO Governance"
        action={
          <Link href="/apps/governance">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
        }
      >
        <InitializeForm
          loomId={loomId}
          onSuccess={fetchData}
        />
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="DAO Governance"
      description="On-chain voting on proposals with quorum requirements"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/governance">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href={`/apps/governance/${loomId}/create`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              New Proposal
            </Button>
          </Link>
        </div>
      }
    >
      {/* Config overview */}
      {config && (
        <Card className="mb-6">
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <Vote className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-sm">{config.name}</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-3">
              <div>
                <span className="text-xs text-muted-foreground">Creator</span>
                <div className="mt-1 flex items-center gap-1">
                  <span className="font-mono text-xs">
                    {truncateAddress(config.creator)}
                  </span>
                  {isCreator && (
                    <Badge variant="outline" className="text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </div>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">
                  Voting Period
                </span>
                <p className="mt-1 font-mono tabular-nums">
                  {(Number(config.votingPeriod) / 3600).toFixed(1)}h
                </p>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">Quorum</span>
                <p className="mt-1 font-mono tabular-nums">
                  {config.quorum.toString()} vote
                  {config.quorum !== 1n ? "s" : ""}
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
            icon={Vote}
            title="No proposals yet"
            description="Create a proposal to start governance voting."
            action={
              <Link href={`/apps/governance/${loomId}/create`}>
                <Button variant="outline" size="sm">
                  <Plus className="mr-1.5 h-3.5 w-3.5" />
                  New Proposal
                </Button>
              </Link>
            }
          />
        ) : (
          proposals
            .slice()
            .reverse()
            .map((p) => (
              <ProposalCard key={p.id.toString()} proposal={p} loomId={loomId} />
            ))
        )}
      </div>
    </PageContainer>
  );
}
