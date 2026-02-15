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
import { useStaking } from "@/hooks/use-staking";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { formatAmount, truncateAddress } from "@/lib/format";
import {
  Landmark,
  ArrowLeft,
  Loader2,
  Plus,
  Download,
  Coins,
} from "lucide-react";
import { toast } from "sonner";
import type { StakingConfig, StakeInfo } from "@/lib/borsh-staking";

const NATIVE_TOKEN_ID = "0".repeat(64);

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useStaking(loomId);
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [rewardRate, setRewardRate] = useState("1000");
  const [minLockPeriodSeconds, setMinLockPeriodSeconds] = useState("86400");

  const canSubmit =
    tokenId.length === 64 &&
    parseInt(rewardRate) > 0 &&
    parseInt(minLockPeriodSeconds) >= 0;

  const disabledReason = tokenId.length !== 64
    ? "Token ID must be 64 characters"
    : parseInt(rewardRate) <= 0
      ? "Reward rate must be greater than 0"
      : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      await initialize(
        tokenId,
        BigInt(rewardRate),
        BigInt(minLockPeriodSeconds)
      );
      toast.success("Staking vault initialized successfully");
      onSuccess();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Initialization failed");
    }
  };

  return (
    <Card className="max-w-lg">
      <CardHeader className="pb-4">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
            <Landmark className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Initialize Staking Vault</CardTitle>
            <CardDescription>
              Configure the staking token, reward rate, and minimum lock period.
              This can only be done once.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
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

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Reward Rate (per second)
            </Label>
            <Input
              type="number"
              value={rewardRate}
              onChange={(e) => setRewardRate(e.target.value)}
              placeholder="1000"
              min="1"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Min Lock Period (seconds)
            </Label>
            <Input
              type="number"
              value={minLockPeriodSeconds}
              onChange={(e) => setMinLockPeriodSeconds(e.target.value)}
              placeholder="86400"
              min="0"
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
            <Landmark className="mr-2 h-3.5 w-3.5" />
          )}
          Initialize Staking Vault
        </FormButton>
      </CardContent>
    </Card>
  );
}

export default function StakingDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const {
    getConfig,
    getStake,
    getPendingRewards,
    getTotalStaked,
    getRewardPool,
    stake,
    unstake,
    claimRewards,
    loading,
  } = useStaking(loomId);

  const [config, setConfig] = useState<StakingConfig | null>(null);
  const [stakeInfo, setStakeInfo] = useState<StakeInfo | null>(null);
  const [pendingRewards, setPendingRewards] = useState<bigint>(0n);
  const [totalStaked, setTotalStaked] = useState<bigint>(0n);
  const [rewardPool, setRewardPool] = useState<bigint>(0n);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const [stakeAmount, setStakeAmount] = useState("");
  const [unstakeAmount, setUnstakeAmount] = useState("");

  const fetchData = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const [cfg, ts, rp] = await Promise.all([
        getConfig(),
        getTotalStaked(),
        getRewardPool(),
      ]);
      setConfig(cfg);
      setTotalStaked(ts);
      setRewardPool(rp);

      if (activeAddress) {
        const [si, pr] = await Promise.all([
          getStake(activeAddress),
          getPendingRewards(activeAddress),
        ]);
        setStakeInfo(si);
        setPendingRewards(pr);
      }
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [
    getConfig,
    getTotalStaked,
    getRewardPool,
    getStake,
    getPendingRewards,
    activeAddress,
    loomId,
  ]);

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

  const addr = activeAddress?.toLowerCase() ?? "";
  const isOperator = config?.operator.toLowerCase() === addr;

  // Not yet initialized
  if (!fetching && !config && loomId) {
    return (
      <PageContainer
        title="Staking Vault"
        action={
          <Link href="/apps/staking">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
        }
      >
        <InitializeForm loomId={loomId} onSuccess={fetchData} />
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="Staking Vault"
      description="Stake tokens and earn rewards over time"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/staking">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href={`/apps/staking/${loomId}/stake`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Stake
            </Button>
          </Link>
          <Link href={`/apps/staking/${loomId}/fund`}>
            <Button variant="outline" size="sm">
              <Coins className="mr-1.5 h-3.5 w-3.5" />
              Fund Rewards
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
              <Landmark className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-sm">Vault Configuration</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-3">
              <div>
                <span className="text-xs text-muted-foreground">Operator</span>
                <div className="mt-1 flex items-center gap-1">
                  <span className="font-mono text-xs">
                    {truncateAddress(config.operator)}
                  </span>
                  {isOperator && (
                    <Badge variant="outline" className="text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </div>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">
                  Reward Rate
                </span>
                <p className="mt-1 font-mono tabular-nums">
                  {config.rewardRate.toString()}/s
                </p>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">
                  Min Lock Period
                </span>
                <p className="mt-1 font-mono tabular-nums">
                  {(Number(config.minLockPeriod) / 86400).toFixed(1)}d
                </p>
              </div>
            </div>

            <div className="mt-4 grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-xs text-muted-foreground">
                  Total Staked
                </span>
                <p className="mt-1 font-mono tabular-nums text-lg">
                  {formatAmount(totalStaked.toString())}
                </p>
              </div>
              <div>
                <span className="text-xs text-muted-foreground">
                  Reward Pool
                </span>
                <p className="mt-1 font-mono tabular-nums text-lg">
                  {formatAmount(rewardPool.toString())}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* User Stake Info */}
      {fetching || loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Your Stake</CardTitle>
            </CardHeader>
            <CardContent className="pt-0 space-y-4">
              {stakeInfo && stakeInfo.amount > 0n ? (
                <>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-xs text-muted-foreground">
                        Staked Amount
                      </span>
                      <p className="mt-1 font-mono tabular-nums text-lg">
                        {formatAmount(stakeInfo.amount.toString())}
                      </p>
                    </div>
                    <div>
                      <span className="text-xs text-muted-foreground">
                        Pending Rewards
                      </span>
                      <p className="mt-1 font-mono tabular-nums text-lg text-norn">
                        {formatAmount(pendingRewards.toString())}
                      </p>
                    </div>
                  </div>

                  {/* Quick unstake */}
                  <div className="flex items-center gap-2">
                    <Input
                      type="number"
                      value={unstakeAmount}
                      onChange={(e) => setUnstakeAmount(e.target.value)}
                      placeholder="Amount to unstake"
                      min="0"
                      step="any"
                      className="font-mono text-sm tabular-nums"
                    />
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        handleAction(
                          () =>
                            unstake(
                              BigInt(
                                Math.floor(parseFloat(unstakeAmount) * 1e12)
                              )
                            ),
                          "Unstaked successfully"
                        )
                      }
                      disabled={
                        loading ||
                        !unstakeAmount ||
                        parseFloat(unstakeAmount) <= 0
                      }
                    >
                      {loading && (
                        <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      )}
                      Unstake
                    </Button>
                  </div>

                  {/* Claim rewards */}
                  {pendingRewards > 0n && (
                    <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-3">
                      <div>
                        <p className="text-xs text-muted-foreground">
                          Available to claim
                        </p>
                        <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                          {formatAmount(pendingRewards.toString())}
                        </p>
                      </div>
                      <Button
                        size="sm"
                        onClick={() =>
                          handleAction(
                            () => claimRewards(),
                            "Rewards claimed successfully"
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
                    </div>
                  )}
                </>
              ) : (
                <div className="text-center py-6">
                  <p className="text-sm text-muted-foreground">
                    You have no active stake. Stake tokens to start earning
                    rewards.
                  </p>
                  <div className="mt-3 flex items-center justify-center gap-2">
                    <Input
                      type="number"
                      value={stakeAmount}
                      onChange={(e) => setStakeAmount(e.target.value)}
                      placeholder="Amount to stake"
                      min="0"
                      step="any"
                      className="max-w-48 font-mono text-sm tabular-nums"
                    />
                    <Button
                      size="sm"
                      onClick={() =>
                        handleAction(
                          () =>
                            stake(
                              BigInt(
                                Math.floor(parseFloat(stakeAmount) * 1e12)
                              )
                            ),
                          "Staked successfully"
                        )
                      }
                      disabled={
                        loading ||
                        !stakeAmount ||
                        parseFloat(stakeAmount) <= 0
                      }
                    >
                      {loading && (
                        <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      )}
                      <Plus className="mr-1.5 h-3.5 w-3.5" />
                      Stake
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </PageContainer>
  );
}
