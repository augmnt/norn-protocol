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
import { FormButton } from "@/components/ui/form-button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { AIRDROP_LOOM_ID } from "@/lib/apps-config";
import { useAirdrop } from "@/hooks/use-airdrop";
import { useWallet } from "@/hooks/use-wallet";
import { formatAmount, truncateAddress } from "@/lib/format";
import {
  Gift,
  ArrowLeft,
  AlertCircle,
  Loader2,
  Plus,
  Download,
  Lock,
  Undo2,
} from "lucide-react";
import { toast } from "sonner";
import type { AirdropConfig } from "@/lib/borsh-airdrop";

const NATIVE_TOKEN_ID = "0".repeat(64);

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useAirdrop(loomId);
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [totalAmount, setTotalAmount] = useState("");

  const canSubmit = tokenId.length === 64 && parseFloat(totalAmount) > 0;

  const disabledReason = tokenId.length !== 64
    ? "Token ID must be 64 characters"
    : parseFloat(totalAmount) <= 0
      ? "Total amount must be greater than 0"
      : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(totalAmount) * 1e12));
      await initialize(tokenId, amountRaw);
      toast.success("Airdrop initialized successfully");
      onSuccess();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Initialization failed");
    }
  };

  return (
    <Card className="max-w-lg">
      <CardHeader className="pb-4">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
            <Gift className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Initialize Airdrop</CardTitle>
            <CardDescription>
              Set the airdrop token and total amount. Tokens are transferred to
              the contract on initialization. This can only be done once.
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

        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            Total Amount (NORN)
          </Label>
          <Input
            type="number"
            value={totalAmount}
            onChange={(e) => setTotalAmount(e.target.value)}
            placeholder="0.00"
            min="0"
            step="any"
            className="font-mono text-sm tabular-nums"
          />
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
            <Gift className="mr-2 h-3.5 w-3.5" />
          )}
          Initialize Airdrop
        </FormButton>
      </CardContent>
    </Card>
  );
}

export default function AirdropDashboardPage() {
  const { activeAddress } = useWallet();
  const {
    getConfig,
    getAllocation,
    isClaimed,
    finalize,
    claim,
    reclaimRemaining,
    loading,
  } = useAirdrop(AIRDROP_LOOM_ID);

  const [config, setConfig] = useState<AirdropConfig | null>(null);
  const [myAllocation, setMyAllocation] = useState<bigint>(0n);
  const [myClaimed, setMyClaimed] = useState(false);
  const [fetching, setFetching] = useState(false);

  const fetchData = useCallback(async () => {
    if (!AIRDROP_LOOM_ID) return;
    setFetching(true);
    try {
      const cfg = await getConfig();
      setConfig(cfg);

      if (activeAddress) {
        const [alloc, claimed] = await Promise.all([
          getAllocation(activeAddress),
          isClaimed(activeAddress),
        ]);
        setMyAllocation(alloc);
        setMyClaimed(claimed);
      }
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getConfig, getAllocation, isClaimed, activeAddress]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

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
  const isCreator = config?.creator.toLowerCase() === addr;

  if (!AIRDROP_LOOM_ID) {
    return (
      <PageContainer title="Airdrop">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              Airdrop contract not configured. Set{" "}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                NEXT_PUBLIC_AIRDROP_LOOM_ID
              </code>{" "}
              in your environment.
            </div>
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  // Not yet initialized
  if (!fetching && !config && AIRDROP_LOOM_ID) {
    return (
      <PageContainer
        title="Airdrop"
        action={
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
        }
      >
        <InitializeForm loomId={AIRDROP_LOOM_ID} onSuccess={fetchData} />
      </PageContainer>
    );
  }

  const claimedPct =
    config && config.totalAmount > 0n
      ? Number((config.claimedAmount * 100n) / config.totalAmount)
      : 0;

  return (
    <PageContainer
      title="Airdrop"
      description="Distribute tokens to a list of recipients"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          {isCreator && !config?.finalized && (
            <Link href="/apps/airdrop/add">
              <Button variant="outline" size="sm">
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                Add Recipients
              </Button>
            </Link>
          )}
          <Link href="/apps/airdrop/claim">
            <Button size="sm">
              <Download className="mr-1.5 h-3.5 w-3.5" />
              Claim
            </Button>
          </Link>
        </div>
      }
    >
      {fetching || loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      ) : config ? (
        <div className="space-y-4">
          {/* Config overview */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Gift className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-sm">Airdrop Status</CardTitle>
                </div>
                <div className="flex items-center gap-1.5">
                  {config.finalized ? (
                    <Badge variant="norn">Finalized</Badge>
                  ) : (
                    <Badge variant="outline">Pending</Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0 space-y-4">
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
                    Recipients
                  </span>
                  <p className="mt-1 font-mono tabular-nums">
                    {config.recipientCount.toString()}
                  </p>
                </div>
                <div>
                  <span className="text-xs text-muted-foreground">
                    Token ID
                  </span>
                  <p className="mt-1 font-mono text-xs truncate">
                    {config.tokenId.slice(0, 16)}...
                  </p>
                </div>
              </div>

              {/* Progress */}
              <div>
                <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                  <span>Claimed / Total</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(config.claimedAmount.toString())} /{" "}
                    {formatAmount(config.totalAmount.toString())}
                  </span>
                </div>
                <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                  <div
                    className="h-full rounded-full bg-norn transition-all"
                    style={{ width: `${Math.min(claimedPct, 100)}%` }}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          {/* User allocation */}
          {myAllocation > 0n && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Your Allocation</CardTitle>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-3">
                  <div>
                    <p className="text-xs text-muted-foreground">
                      {myClaimed ? "Claimed" : "Available to claim"}
                    </p>
                    <p className="mt-0.5 font-mono text-lg tabular-nums text-norn">
                      {formatAmount(myAllocation.toString())}
                    </p>
                  </div>
                  {!myClaimed && config.finalized && (
                    <Button
                      size="sm"
                      onClick={() =>
                        handleAction(
                          () => claim(),
                          "Airdrop claimed successfully"
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
                  {myClaimed && (
                    <Badge variant="secondary">Claimed</Badge>
                  )}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Creator actions */}
          {isCreator && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Creator Actions</CardTitle>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="flex flex-wrap gap-2">
                  {!config.finalized && (
                    <>
                      <Link href="/apps/airdrop/add">
                        <Button variant="outline" size="sm">
                          <Plus className="mr-1.5 h-3.5 w-3.5" />
                          Add Recipients
                        </Button>
                      </Link>
                      <Button
                        size="sm"
                        onClick={() =>
                          handleAction(
                            () => finalize(),
                            "Airdrop finalized"
                          )
                        }
                        disabled={loading}
                      >
                        {loading && (
                          <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                        )}
                        <Lock className="mr-1.5 h-3.5 w-3.5" />
                        Finalize
                      </Button>
                    </>
                  )}
                  {config.finalized && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        handleAction(
                          () => reclaimRemaining(),
                          "Remaining tokens reclaimed"
                        )
                      }
                      disabled={loading}
                    >
                      {loading && (
                        <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      )}
                      <Undo2 className="mr-1.5 h-3.5 w-3.5" />
                      Reclaim Remaining
                    </Button>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      ) : null}
    </PageContainer>
  );
}
