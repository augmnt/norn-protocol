"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useEscrow } from "@/hooks/use-escrow";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount, formatTimestamp } from "@/lib/format";
import {
  ArrowLeft,
  ShieldCheck,
  Loader2,
  AlertCircle,
  CheckCircle2,
  XCircle,
  Clock,
  ArrowRight,
  Ban,
} from "lucide-react";
import { toast } from "sonner";
import type { Deal, DealStatus } from "@/lib/borsh-escrow";
import { cn } from "@/lib/utils";

const STATUS_VARIANT: Record<string, "norn" | "destructive" | "secondary"> = {
  Created: "norn",
  Funded: "norn",
  Delivered: "norn",
  Completed: "secondary",
  Disputed: "destructive",
  Cancelled: "destructive",
  Refunded: "secondary",
};

const STATUS_ICON: Record<string, React.ComponentType<{ className?: string }>> = {
  Created: Clock,
  Funded: ShieldCheck,
  Delivered: ArrowRight,
  Completed: CheckCircle2,
  Disputed: AlertCircle,
  Cancelled: XCircle,
  Refunded: Ban,
};

const STATUS_STEPS: DealStatus[] = [
  "Created",
  "Funded",
  "Delivered",
  "Completed",
];

function StatusTimeline({ status }: { status: DealStatus }) {
  const stepIndex = STATUS_STEPS.indexOf(status);
  const isTerminal = ["Disputed", "Cancelled", "Refunded"].includes(status);

  return (
    <div className="flex items-center gap-1">
      {STATUS_STEPS.map((step, i) => {
        const active = isTerminal ? false : i <= stepIndex;
        return (
          <div key={step} className="flex items-center gap-1">
            <div
              className={cn(
                "flex h-6 items-center justify-center rounded-full px-2 text-[10px] font-medium transition-colors",
                active
                  ? "bg-norn/10 text-norn"
                  : "bg-muted text-muted-foreground"
              )}
            >
              {step}
            </div>
            {i < STATUS_STEPS.length - 1 && (
              <div
                className={cn(
                  "h-px w-4",
                  active && i < stepIndex ? "bg-norn" : "bg-border"
                )}
              />
            )}
          </div>
        );
      })}
      {isTerminal && (
        <>
          <div className="h-px w-4 bg-border" />
          <div className="flex h-6 items-center justify-center rounded-full bg-destructive/10 px-2 text-[10px] font-medium text-destructive">
            {status}
          </div>
        </>
      )}
    </div>
  );
}

export default function DealDetailPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const id = params.id as string;
  const dealId = BigInt(id || "0");
  const { activeAddress } = useWallet();
  const {
    getDeal,
    fundDeal,
    markDelivered,
    confirmReceived,
    dispute,
    cancelDeal,
    refundExpired,
    loading,
  } = useEscrow(loomId);

  const [deal, setDeal] = useState<Deal | null>(null);
  const [fetching, setFetching] = useState(true);
  const hasLoadedRef = useRef(false);

  const fetchDeal = useCallback(async () => {
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const d = await getDeal(dealId);
      setDeal(d);
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getDeal, dealId]);

  useEffect(() => {
    fetchDeal();
  }, [fetchDeal]);

  useLoomRefresh(loomId, fetchDeal);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isBuyer = deal?.buyer.toLowerCase() === addr;
  const isSeller = deal?.seller.toLowerCase() === addr;
  const now = Math.floor(Date.now() / 1000);
  const isExpired = deal ? now >= Number(deal.deadline) : false;

  const handleAction = async (
    action: () => Promise<unknown>,
    successMsg: string
  ) => {
    try {
      await action();
      toast.success(successMsg);
      fetchDeal();
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

  if (!deal) {
    return (
      <PageContainer title="Deal Not Found">
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Deal #{dealId.toString()} was not found.
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  const Icon = STATUS_ICON[deal.status] ?? ShieldCheck;

  return (
    <PageContainer
      title={`Deal #${deal.id.toString()}`}
      action={
        <Link href={`/apps/escrow/${loomId}`}>
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-2xl space-y-4">
        {/* Status */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Icon className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Status</CardTitle>
              </div>
              <Badge variant={STATUS_VARIANT[deal.status] ?? "secondary"}>
                {deal.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <StatusTimeline status={deal.status} />
          </CardContent>
        </Card>

        {/* Deal Info */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Deal Details</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Description</span>
                <span className="text-right max-w-[60%] truncate">
                  {deal.description}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Buyer</span>
                <span className="font-mono text-xs">
                  {truncateAddress(deal.buyer)}
                  {isBuyer && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Seller</span>
                <span className="font-mono text-xs">
                  {truncateAddress(deal.seller)}
                  {isSeller && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(deal.amount.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token</span>
                <span className="font-mono text-xs truncate max-w-48">
                  {deal.tokenId}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span className="text-xs">
                  {deal.createdAt > 0n
                    ? formatTimestamp(Number(deal.createdAt))
                    : "\u2014"}
                </span>
              </div>
              {deal.fundedAt > 0n && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Funded</span>
                  <span className="text-xs">
                    {formatTimestamp(Number(deal.fundedAt))}
                  </span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-muted-foreground">Deadline</span>
                <span className={cn("text-xs", isExpired && "text-destructive")}>
                  {formatTimestamp(Number(deal.deadline))}
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
              {/* Buyer: Fund (when Created) */}
              {isBuyer && deal.status === "Created" && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => fundDeal(dealId),
                      "Deal funded successfully"
                    )
                  }
                  disabled={loading}
                >
                  {loading && <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />}
                  Fund Deal
                </Button>
              )}

              {/* Buyer: Cancel (when Created) */}
              {isBuyer && deal.status === "Created" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => cancelDeal(dealId),
                      "Deal cancelled"
                    )
                  }
                  disabled={loading}
                >
                  Cancel
                </Button>
              )}

              {/* Seller: Mark Delivered (when Funded) */}
              {isSeller && deal.status === "Funded" && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => markDelivered(dealId),
                      "Marked as delivered"
                    )
                  }
                  disabled={loading}
                >
                  {loading && <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />}
                  Mark Delivered
                </Button>
              )}

              {/* Buyer: Confirm Received (when Delivered) */}
              {isBuyer && deal.status === "Delivered" && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => confirmReceived(dealId),
                      "Funds released to seller"
                    )
                  }
                  disabled={loading}
                >
                  {loading && <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />}
                  Confirm Received
                </Button>
              )}

              {/* Buyer: Dispute (when Funded or Delivered) */}
              {isBuyer &&
                (deal.status === "Funded" || deal.status === "Delivered") && (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => dispute(dealId),
                        "Dispute filed"
                      )
                    }
                    disabled={loading}
                  >
                    Dispute
                  </Button>
                )}

              {/* Anyone: Refund expired */}
              {isExpired &&
                (deal.status === "Funded" ||
                  deal.status === "Delivered" ||
                  deal.status === "Disputed") && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      handleAction(
                        () => refundExpired(dealId),
                        "Deal refunded"
                      )
                    }
                    disabled={loading}
                  >
                    Refund (Expired)
                  </Button>
                )}

              {/* No actions for terminal states */}
              {["Completed", "Cancelled", "Refunded"].includes(deal.status) && (
                <p className="text-xs text-muted-foreground py-1">
                  This deal is finalized. No further actions available.
                </p>
              )}

              {/* Not buyer or seller */}
              {!isBuyer &&
                !isSeller &&
                !isExpired &&
                !["Completed", "Cancelled", "Refunded"].includes(
                  deal.status
                ) && (
                  <p className="text-xs text-muted-foreground py-1">
                    You are not a party to this deal.
                  </p>
                )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
