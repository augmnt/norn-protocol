"use client";

import { useState, useEffect, useCallback } from "react";
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
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { CROWDFUND_LOOM_ID } from "@/lib/apps-config";
import { useCrowdfund } from "@/hooks/use-crowdfund";
import { useWallet } from "@/hooks/use-wallet";
import {
  truncateAddress,
  formatAmount,
  formatTimestamp,
} from "@/lib/format";
import {
  HandCoins,
  ArrowLeft,
  AlertCircle,
  Loader2,
  Coins,
  Undo2,
  CheckCircle2,
} from "lucide-react";
import { toast } from "sonner";
import type { CrowdfundConfig, CampaignStatus } from "@/lib/borsh-crowdfund";

const NATIVE_TOKEN_ID = "0".repeat(64);

const STATUS_VARIANT: Record<
  CampaignStatus,
  "norn" | "secondary" | "destructive"
> = {
  Active: "norn",
  Succeeded: "secondary",
  Failed: "destructive",
};

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useCrowdfund(loomId);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [goal, setGoal] = useState("");
  const [deadlineHours, setDeadlineHours] = useState("168");

  const canSubmit =
    title.trim().length > 0 &&
    description.trim().length > 0 &&
    parseFloat(goal) > 0 &&
    parseFloat(deadlineHours) > 0;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const goalRaw = BigInt(Math.floor(parseFloat(goal) * 1e12));
      const deadline =
        BigInt(Math.floor(Date.now() / 1000)) +
        BigInt(Math.floor(parseFloat(deadlineHours) * 3600));

      await initialize(
        title.trim(),
        description.trim(),
        tokenId,
        goalRaw,
        deadline
      );
      toast.success("Crowdfund campaign created successfully");
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
          <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
            <HandCoins className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Create Campaign</CardTitle>
            <CardDescription>
              Launch an all-or-nothing crowdfunding campaign. If the goal
              is met by the deadline, funds go to you. Otherwise,
              contributors can claim refunds.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Title</Label>
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Campaign title"
            maxLength={128}
            className="text-sm"
          />
        </div>

        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Description</Label>
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe what this campaign is for..."
            className="text-sm min-h-[80px] resize-y"
            maxLength={512}
            rows={3}
          />
          <p className="text-[10px] text-muted-foreground text-right">
            {description.length}/512
          </p>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Goal (NORN)
            </Label>
            <Input
              type="number"
              value={goal}
              onChange={(e) => setGoal(e.target.value)}
              placeholder="0.00"
              min="0"
              step="any"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Deadline (hours)
            </Label>
            <Input
              type="number"
              value={deadlineHours}
              onChange={(e) => setDeadlineHours(e.target.value)}
              placeholder="168"
              min="1"
              className="font-mono text-sm tabular-nums"
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Token ID</Label>
          <Input
            value={tokenId}
            onChange={(e) => setTokenId(e.target.value)}
            placeholder="64 hex chars (native = all zeros)"
            className="font-mono text-xs"
          />
          <p className="text-[10px] text-muted-foreground">
            Leave default for native NORN token.
          </p>
        </div>

        <Button
          onClick={handleSubmit}
          disabled={!canSubmit || loading}
          className="w-full"
        >
          {loading ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <HandCoins className="mr-2 h-3.5 w-3.5" />
          )}
          Create Campaign
        </Button>
      </CardContent>
    </Card>
  );
}

export default function CrowdfundDashboardPage() {
  const { activeAddress } = useWallet();
  const {
    getConfig,
    getTotalRaised,
    getContribution,
    getContributorCount,
    finalize,
    loading,
  } = useCrowdfund(CROWDFUND_LOOM_ID);
  const [config, setConfig] = useState<CrowdfundConfig | null>(null);
  const [totalRaised, setTotalRaised] = useState<bigint>(0n);
  const [contributorCount, setContributorCount] = useState<bigint>(0n);
  const [myContribution, setMyContribution] = useState<bigint>(0n);
  const [fetching, setFetching] = useState(false);

  const fetchData = useCallback(async () => {
    if (!CROWDFUND_LOOM_ID) return;
    setFetching(true);
    try {
      const [cfg, raised, count] = await Promise.all([
        getConfig(),
        getTotalRaised(),
        getContributorCount(),
      ]);
      setConfig(cfg);
      setTotalRaised(raised);
      setContributorCount(count);

      if (activeAddress && cfg) {
        const contrib = await getContribution(activeAddress);
        setMyContribution(contrib);
      }
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [
    getConfig,
    getTotalRaised,
    getContributorCount,
    getContribution,
    activeAddress,
  ]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isCreator = config?.creator.toLowerCase() === addr;
  const now = Math.floor(Date.now() / 1000);
  const hasEnded = config ? now >= Number(config.deadline) : false;
  const isActive = config?.status === "Active";
  const raisedPct =
    config && config.goal > 0n
      ? Number((totalRaised * 100n) / config.goal)
      : 0;

  if (!CROWDFUND_LOOM_ID) {
    return (
      <PageContainer title="Crowdfund">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              Crowdfund contract not configured. Set{" "}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                NEXT_PUBLIC_CROWDFUND_LOOM_ID
              </code>{" "}
              in your environment.
            </div>
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  // Not yet initialized
  if (!fetching && !config && CROWDFUND_LOOM_ID) {
    return (
      <PageContainer
        title="Crowdfund"
        action={
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
        }
      >
        <InitializeForm loomId={CROWDFUND_LOOM_ID} onSuccess={fetchData} />
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="Crowdfund"
      description="All-or-nothing fundraising with goal and deadline"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          {isActive && !hasEnded && (
            <Link href="/apps/crowdfund/contribute">
              <Button size="sm">
                <Coins className="mr-1.5 h-3.5 w-3.5" />
                Contribute
              </Button>
            </Link>
          )}
          {config?.status === "Failed" && myContribution > 0n && (
            <Link href="/apps/crowdfund/refund">
              <Button variant="outline" size="sm">
                <Undo2 className="mr-1.5 h-3.5 w-3.5" />
                Refund
              </Button>
            </Link>
          )}
        </div>
      }
    >
      {fetching || loading ? (
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      ) : config ? (
        <div className="max-w-2xl space-y-4">
          {/* Campaign status */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <HandCoins className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-sm">{config.title}</CardTitle>
                </div>
                <Badge variant={STATUS_VARIANT[config.status] ?? "secondary"}>
                  {config.status}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="pt-0 space-y-3">
              {config.description && (
                <p className="text-sm text-muted-foreground">
                  {config.description}
                </p>
              )}

              {/* Progress bar */}
              <div>
                <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                  <span>Raised / Goal</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(totalRaised.toString())} /{" "}
                    {formatAmount(config.goal.toString())}
                  </span>
                </div>
                <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                  <div
                    className="h-full rounded-full bg-norn transition-all"
                    style={{ width: `${Math.min(raisedPct, 100)}%` }}
                  />
                </div>
                <p className="mt-1 text-right text-xs text-muted-foreground font-mono tabular-nums">
                  {raisedPct}%
                </p>
              </div>

              {/* My contribution highlight */}
              {myContribution > 0n && (
                <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-3">
                  <div>
                    <p className="text-xs text-muted-foreground">
                      Your contribution
                    </p>
                    <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                      {formatAmount(myContribution.toString())}
                    </p>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Campaign Details */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Campaign Details</CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Creator</span>
                  <span className="font-mono text-xs">
                    {truncateAddress(config.creator)}
                    {isCreator && (
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
                  <span className="text-muted-foreground">Goal</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.goal.toString())} NORN
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Total Raised</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(totalRaised.toString())} NORN
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Contributors</span>
                  <span className="font-mono tabular-nums">
                    {contributorCount.toString()}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Deadline</span>
                  <span className="text-xs">
                    {formatTimestamp(Number(config.deadline))}
                    {hasEnded && (
                      <span className="ml-1 text-muted-foreground">
                        (ended)
                      </span>
                    )}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Token ID</span>
                  <span className="font-mono text-xs">
                    {config.tokenId.slice(0, 8)}...{config.tokenId.slice(-8)}
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
                {isActive && !hasEnded && (
                  <Link href="/apps/crowdfund/contribute">
                    <Button size="sm">
                      <Coins className="mr-1.5 h-3.5 w-3.5" />
                      Contribute
                    </Button>
                  </Link>
                )}

                {isCreator && hasEnded && isActive && (
                  <Button
                    size="sm"
                    onClick={async () => {
                      try {
                        await finalize();
                        toast.success("Campaign finalized");
                        fetchData();
                      } catch (e) {
                        toast.error(
                          e instanceof Error
                            ? e.message
                            : "Finalization failed"
                        );
                      }
                    }}
                    disabled={loading}
                  >
                    {loading && (
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    )}
                    <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                    Finalize
                  </Button>
                )}

                {config.status === "Failed" && myContribution > 0n && (
                  <Link href="/apps/crowdfund/refund">
                    <Button variant="outline" size="sm">
                      <Undo2 className="mr-1.5 h-3.5 w-3.5" />
                      Claim Refund
                    </Button>
                  </Link>
                )}

                {config.status === "Succeeded" && (
                  <p className="text-xs text-muted-foreground py-1">
                    Campaign succeeded. Funds have been released to the
                    creator.
                  </p>
                )}

                {config.status === "Failed" && myContribution === 0n && (
                  <p className="text-xs text-muted-foreground py-1">
                    Campaign failed to reach its goal. You have no
                    contribution to refund.
                  </p>
                )}

                {isActive && hasEnded && !isCreator && (
                  <p className="text-xs text-muted-foreground py-1">
                    Campaign has ended. Waiting for the creator to finalize.
                  </p>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      ) : null}
    </PageContainer>
  );
}
