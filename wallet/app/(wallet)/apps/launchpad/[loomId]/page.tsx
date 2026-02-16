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
import { useLaunchpad } from "@/hooks/use-launchpad";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import {
  truncateAddress,
  truncateHash,
  formatAmount,
  formatTimestamp,
} from "@/lib/format";
import {
  Rocket,
  Loader2,
  Coins,
} from "lucide-react";
import { toast } from "sonner";
import type { LaunchConfig } from "@/lib/borsh-launchpad";

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useLaunchpad(loomId);
  const [tokenId, setTokenId] = useState("");
  const [price, setPrice] = useState("");
  const [hardCap, setHardCap] = useState("");
  const [maxPerWallet, setMaxPerWallet] = useState("");
  const [startDelayHours, setStartDelayHours] = useState("1");
  const [durationHours, setDurationHours] = useState("72");
  const [totalTokens, setTotalTokens] = useState("");

  const canSubmit =
    /^[a-fA-F0-9]{64}$/.test(tokenId) &&
    parseFloat(price) > 0 &&
    parseFloat(hardCap) > 0 &&
    parseFloat(maxPerWallet) > 0 &&
    parseFloat(startDelayHours) >= 0 &&
    parseFloat(durationHours) > 0 &&
    parseFloat(totalTokens) > 0;

  const disabledReason = !tokenId
    ? "Enter a token ID"
    : !/^[a-fA-F0-9]{64}$/.test(tokenId)
      ? "Token ID must be 64 hex characters"
      : parseFloat(price) <= 0
        ? "Price must be greater than 0"
        : parseFloat(hardCap) <= 0
          ? "Hard cap must be greater than 0"
          : parseFloat(maxPerWallet) <= 0
            ? "Max per wallet must be greater than 0"
            : parseFloat(totalTokens) <= 0
              ? "Total tokens must be greater than 0"
              : parseFloat(durationHours) <= 0
                ? "Duration must be greater than 0"
                : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const priceRaw = BigInt(Math.floor(parseFloat(price) * 1e12));
      const hardCapRaw = BigInt(Math.floor(parseFloat(hardCap) * 1e12));
      const maxPerWalletRaw = BigInt(
        Math.floor(parseFloat(maxPerWallet) * 1e12)
      );
      const now = BigInt(Math.floor(Date.now() / 1000));
      const startTime =
        now + BigInt(Math.floor(parseFloat(startDelayHours) * 3600));
      const endTime =
        startTime + BigInt(Math.floor(parseFloat(durationHours) * 3600));
      const totalTokensRaw = BigInt(
        Math.floor(parseFloat(totalTokens) * 1e12)
      );

      await initialize(
        tokenId,
        priceRaw,
        hardCapRaw,
        maxPerWalletRaw,
        startTime,
        endTime,
        totalTokensRaw
      );
      toast.success("Launchpad initialized successfully");
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
            <Rocket className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Initialize Launchpad</CardTitle>
            <CardDescription>
              Set up a fixed-price token sale. Deposit tokens and configure
              the sale parameters. This can only be done once.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            Token ID (64 hex chars)
          </Label>
          <Input
            value={tokenId}
            onChange={(e) => setTokenId(e.target.value)}
            placeholder="abcd1234...  (64 hex characters)"
            className="font-mono text-xs"
            maxLength={64}
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Price per Token
            </Label>
            <Input
              type="number"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="0.00"
              min="0"
              step="any"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">Hard Cap</Label>
            <Input
              type="number"
              value={hardCap}
              onChange={(e) => setHardCap(e.target.value)}
              placeholder="0.00"
              min="0"
              step="any"
              className="font-mono text-sm tabular-nums"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Max per Wallet
            </Label>
            <Input
              type="number"
              value={maxPerWallet}
              onChange={(e) => setMaxPerWallet(e.target.value)}
              placeholder="0.00"
              min="0"
              step="any"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Total Tokens for Sale
            </Label>
            <Input
              type="number"
              value={totalTokens}
              onChange={(e) => setTotalTokens(e.target.value)}
              placeholder="0.00"
              min="0"
              step="any"
              className="font-mono text-sm tabular-nums"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Start Delay (hours)
            </Label>
            <Input
              type="number"
              value={startDelayHours}
              onChange={(e) => setStartDelayHours(e.target.value)}
              placeholder="1"
              min="0"
              className="font-mono text-sm tabular-nums"
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Duration (hours)
            </Label>
            <Input
              type="number"
              value={durationHours}
              onChange={(e) => setDurationHours(e.target.value)}
              placeholder="72"
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
            <Rocket className="mr-2 h-3.5 w-3.5" />
          )}
          Initialize Launchpad
        </FormButton>
      </CardContent>
    </Card>
  );
}

export default function LaunchpadDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getConfig, getTotalRaised, getContribution, finalize, loading } =
    useLaunchpad(loomId);
  const [config, setConfig] = useState<LaunchConfig | null>(null);
  const [totalRaised, setTotalRaised] = useState<bigint>(0n);
  const [myContribution, setMyContribution] = useState<bigint>(0n);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const [cfg, raised] = await Promise.all([
        getConfig(),
        getTotalRaised(),
      ]);
      setConfig(cfg);
      setTotalRaised(raised);

      if (activeAddress && cfg) {
        const contrib = await getContribution(activeAddress);
        setMyContribution(contrib);
      }
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getConfig, getTotalRaised, getContribution, activeAddress, loomId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  const now = Math.floor(Date.now() / 1000);
  const isCreator =
    config?.creator.toLowerCase() === activeAddress?.toLowerCase();
  const hasStarted = config ? now >= Number(config.startTime) : false;
  const hasEnded = config ? now >= Number(config.endTime) : false;
  const isActive = hasStarted && !hasEnded && !config?.finalized;
  const raisedPct =
    config && config.hardCap > 0n
      ? Number((totalRaised * 100n) / config.hardCap)
      : 0;

  // Not yet initialized
  if (!fetching && !config && loomId) {
    return (
      <PageContainer
        title="Token Launchpad"
        breadcrumb={[{label: "Apps", href: "/discover"}, {label: "Token Launchpad", href: "/apps/launchpad"}, {label: truncateHash(loomId, 8)}]}
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
      title="Token Launchpad"
      description="Fixed-price token sale with hard cap"
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "Token Launchpad", href: "/apps/launchpad"}, {label: truncateHash(loomId, 8)}]}
      action={
        isActive ? (
          <Link href={`/apps/launchpad/${loomId}/contribute`}>
            <Button size="sm">
              <Coins className="mr-1.5 h-3.5 w-3.5" />
              Contribute
            </Button>
          </Link>
        ) : undefined
      }
    >
      {fetching ? (
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      ) : config ? (
        <div className="max-w-2xl space-y-4">
          {/* Sale status */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Rocket className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-sm">Sale Status</CardTitle>
                </div>
                <div className="flex items-center gap-1.5">
                  {config.finalized ? (
                    <Badge variant="secondary">Finalized</Badge>
                  ) : isActive ? (
                    <Badge variant="norn">Active</Badge>
                  ) : hasEnded ? (
                    <Badge variant="secondary">Ended</Badge>
                  ) : (
                    <Badge variant="outline">Not Started</Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0 space-y-3">
              {/* Progress bar */}
              <div>
                <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                  <span>Raised / Hard Cap</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(totalRaised.toString())} /{" "}
                    {formatAmount(config.hardCap.toString())}
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

          {/* Sale Details */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Sale Details</CardTitle>
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
                  <span className="text-muted-foreground">Token ID</span>
                  <span className="font-mono text-xs">
                    {config.tokenId.slice(0, 8)}...{config.tokenId.slice(-8)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Price</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.price.toString())}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Hard Cap</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.hardCap.toString())}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Max per Wallet</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.maxPerWallet.toString())}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Total Tokens</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.totalTokens.toString())}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Start Time</span>
                  <span className="text-xs">
                    {formatTimestamp(Number(config.startTime))}
                    {hasStarted && (
                      <span className="ml-1 text-muted-foreground">
                        (started)
                      </span>
                    )}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">End Time</span>
                  <span className="text-xs">
                    {formatTimestamp(Number(config.endTime))}
                    {hasEnded && (
                      <span className="ml-1 text-muted-foreground">
                        (ended)
                      </span>
                    )}
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
                {isActive && (
                  <Link href={`/apps/launchpad/${loomId}/contribute`}>
                    <Button size="sm">
                      <Coins className="mr-1.5 h-3.5 w-3.5" />
                      Contribute
                    </Button>
                  </Link>
                )}

                {isCreator && hasEnded && !config.finalized && (
                  <Button
                    size="sm"
                    onClick={async () => {
                      try {
                        await finalize();
                        toast.success("Launchpad finalized");
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
                    Finalize Sale
                  </Button>
                )}

                {config.finalized && (
                  <p className="text-xs text-muted-foreground py-1">
                    This sale has been finalized. No further actions available.
                  </p>
                )}

                {!isActive && !hasEnded && !config.finalized && (
                  <p className="text-xs text-muted-foreground py-1">
                    Sale has not started yet. Contributions open at{" "}
                    {formatTimestamp(Number(config.startTime))}.
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
