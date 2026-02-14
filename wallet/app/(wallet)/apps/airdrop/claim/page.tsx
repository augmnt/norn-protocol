"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { AIRDROP_LOOM_ID } from "@/lib/apps-config";
import { useAirdrop } from "@/hooks/use-airdrop";
import { useWallet } from "@/hooks/use-wallet";
import { formatAmount } from "@/lib/format";
import {
  ArrowLeft,
  Download,
  Gift,
  Loader2,
  CheckCircle,
} from "lucide-react";
import { toast } from "sonner";

export default function ClaimAirdropPage() {
  const { activeAddress } = useWallet();
  const { getConfig, getAllocation, isClaimed, claim, loading } =
    useAirdrop(AIRDROP_LOOM_ID);

  const [allocation, setAllocation] = useState<bigint>(0n);
  const [claimed, setClaimed] = useState(false);
  const [finalized, setFinalized] = useState(false);
  const [fetching, setFetching] = useState(true);

  const fetchData = useCallback(async () => {
    if (!activeAddress) return;
    setFetching(true);
    try {
      const [cfg, alloc, cl] = await Promise.all([
        getConfig(),
        getAllocation(activeAddress),
        isClaimed(activeAddress),
      ]);
      setFinalized(cfg?.finalized ?? false);
      setAllocation(alloc);
      setClaimed(cl);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getConfig, getAllocation, isClaimed, activeAddress]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleClaim = async () => {
    try {
      await claim();
      toast.success("Airdrop claimed successfully");
      fetchData();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to claim");
    }
  };

  return (
    <PageContainer
      title="Claim Airdrop"
      action={
        <Link href="/apps/airdrop">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
                <Gift className="h-4 w-4 text-norn" />
              </div>
              <CardTitle className="text-base">Your Airdrop</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            {fetching ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            ) : allocation === 0n ? (
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">
                  You do not have an allocation in this airdrop.
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="flex items-center justify-between rounded-lg border border-norn/20 bg-norn/5 p-4">
                  <div>
                    <p className="text-xs text-muted-foreground">
                      Your Allocation
                    </p>
                    <p className="mt-1 font-mono text-2xl tabular-nums text-norn">
                      {formatAmount(allocation.toString())}
                    </p>
                  </div>
                  <div>
                    {claimed ? (
                      <Badge variant="secondary" className="text-xs">
                        <CheckCircle className="mr-1 h-3 w-3" />
                        Claimed
                      </Badge>
                    ) : !finalized ? (
                      <Badge variant="outline" className="text-xs">
                        Not Finalized
                      </Badge>
                    ) : null}
                  </div>
                </div>

                {claimed ? (
                  <p className="text-sm text-muted-foreground text-center">
                    You have already claimed your airdrop allocation.
                  </p>
                ) : !finalized ? (
                  <p className="text-sm text-muted-foreground text-center">
                    The airdrop has not been finalized yet. You can claim once
                    the creator finalizes it.
                  </p>
                ) : (
                  <Button
                    onClick={handleClaim}
                    disabled={loading}
                    className="w-full"
                  >
                    {loading ? (
                      <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <Download className="mr-2 h-3.5 w-3.5" />
                    )}
                    Claim Airdrop
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
