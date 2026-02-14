"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
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
import { CROWDFUND_LOOM_ID } from "@/lib/apps-config";
import { useCrowdfund } from "@/hooks/use-crowdfund";
import { useWallet } from "@/hooks/use-wallet";
import { formatAmount } from "@/lib/format";
import { ArrowLeft, Undo2, Loader2, AlertCircle } from "lucide-react";
import { toast } from "sonner";

export default function CrowdfundRefundPage() {
  const router = useRouter();
  const { activeAddress } = useWallet();
  const { refund, getContribution, getConfig, loading } =
    useCrowdfund(CROWDFUND_LOOM_ID);

  const [myContribution, setMyContribution] = useState<bigint>(0n);
  const [isFailed, setIsFailed] = useState(false);
  const [fetching, setFetching] = useState(true);

  const fetchData = useCallback(async () => {
    if (!activeAddress) return;
    setFetching(true);
    try {
      const [contrib, cfg] = await Promise.all([
        getContribution(activeAddress),
        getConfig(),
      ]);
      setMyContribution(contrib);
      setIsFailed(cfg?.status === "Failed");
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [activeAddress, getContribution, getConfig]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleRefund = async () => {
    try {
      await refund();
      toast.success("Refund claimed successfully");
      router.push("/apps/crowdfund");
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to claim refund"
      );
    }
  };

  return (
    <PageContainer
      title="Claim Refund"
      action={
        <Link href="/apps/crowdfund">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
                <Undo2 className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Claim Refund</CardTitle>
                <CardDescription>
                  The campaign did not reach its goal. You can reclaim your
                  contribution.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {fetching ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            ) : !isFailed ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <AlertCircle className="h-4 w-4" />
                Refunds are only available when the campaign has failed.
              </div>
            ) : myContribution === 0n ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <AlertCircle className="h-4 w-4" />
                You have no contribution to refund.
              </div>
            ) : (
              <>
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

                <Button
                  onClick={handleRefund}
                  disabled={loading}
                  className="w-full"
                >
                  {loading ? (
                    <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Undo2 className="mr-2 h-3.5 w-3.5" />
                  )}
                  Claim Refund
                </Button>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
