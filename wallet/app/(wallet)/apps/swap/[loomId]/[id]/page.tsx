"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useSwap } from "@/hooks/use-swap";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, truncateHash, formatAmount, formatTimestamp } from "@/lib/format";
import {
  ArrowLeftRight,
  Loader2,
  CheckCircle,
  XCircle,
} from "lucide-react";
import { toast } from "sonner";
import type { SwapOrder, OrderStatus } from "@/lib/borsh-swap";

const STATUS_VARIANT: Record<
  OrderStatus,
  "norn" | "secondary" | "destructive"
> = {
  Open: "norn",
  Filled: "secondary",
  Cancelled: "destructive",
};

export default function OrderDetailPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const orderId = BigInt((params.id as string) || "0");
  const { activeAddress } = useWallet();
  const { getOrder, fillOrder, cancelOrder, loading } = useSwap(loomId);

  const [order, setOrder] = useState<SwapOrder | null>(null);
  const [fetching, setFetching] = useState(true);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const o = await getOrder(orderId);
      setOrder(o);
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getOrder, orderId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isCreator = order?.creator.toLowerCase() === addr;

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

  if (!order) {
    return (
      <PageContainer title="Order Not Found">
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Order #{orderId.toString()} was not found.
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title={`Order #${order.id.toString()}`}
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "OTC Swap", href: "/apps/swap"}, {label: truncateHash(loomId, 8), href: `/apps/swap/${loomId}`}, {label: `Order #${params.id}`}]}
    >
      <div className="max-w-2xl space-y-4">
        {/* Swap visualization */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <ArrowLeftRight className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-sm">Swap Details</CardTitle>
              </div>
              <Badge variant={STATUS_VARIANT[order.status]}>
                {order.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="flex items-center gap-3">
              <div className="flex-1 rounded-lg border border-border p-3">
                <p className="text-xs text-muted-foreground">Selling</p>
                <p className="mt-1 font-mono text-lg tabular-nums">
                  {formatAmount(order.sellAmount.toString())}
                </p>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground truncate">
                  {truncateAddress("0x" + order.sellToken.slice(0, 40))}
                </p>
              </div>
              <ArrowLeftRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              <div className="flex-1 rounded-lg border border-border p-3">
                <p className="text-xs text-muted-foreground">Buying</p>
                <p className="mt-1 font-mono text-lg tabular-nums">
                  {formatAmount(order.buyAmount.toString())}
                </p>
                <p className="mt-0.5 font-mono text-xs text-muted-foreground truncate">
                  {truncateAddress("0x" + order.buyToken.slice(0, 40))}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Order info */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Order Information</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Creator</span>
                <span className="font-mono text-xs">
                  {truncateAddress(order.creator)}
                  {isCreator && (
                    <Badge variant="outline" className="ml-2 text-[9px] py-0">
                      You
                    </Badge>
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Status</span>
                <Badge variant={STATUS_VARIANT[order.status]}>
                  {order.status}
                </Badge>
              </div>
              {order.status === "Filled" && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Filled By</span>
                  <span className="font-mono text-xs">
                    {truncateAddress(order.filledBy)}
                  </span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-muted-foreground">Sell Token</span>
                <span className="font-mono text-xs truncate max-w-48">
                  {order.sellToken}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Sell Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(order.sellAmount.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Buy Token</span>
                <span className="font-mono text-xs truncate max-w-48">
                  {order.buyToken}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Buy Amount</span>
                <span className="font-mono tabular-nums">
                  {formatAmount(order.buyAmount.toString())}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span className="text-xs">
                  {order.createdAt > 0n
                    ? formatTimestamp(Number(order.createdAt))
                    : "\u2014"}
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
              {/* Fill order (if open & not creator) */}
              {order.status === "Open" && !isCreator && (
                <Button
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => fillOrder(orderId),
                      "Order filled successfully"
                    )
                  }
                  disabled={loading}
                >
                  {loading && (
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  )}
                  <CheckCircle className="mr-1.5 h-3.5 w-3.5" />
                  Fill Order
                </Button>
              )}

              {/* Cancel (if open & creator) */}
              {order.status === "Open" && isCreator && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() =>
                    handleAction(
                      () => cancelOrder(orderId),
                      "Order cancelled"
                    )
                  }
                  disabled={loading}
                >
                  <XCircle className="mr-1.5 h-3.5 w-3.5" />
                  Cancel Order
                </Button>
              )}

              {/* Filled */}
              {order.status === "Filled" && (
                <p className="text-xs text-muted-foreground py-1">
                  This order has been filled.
                </p>
              )}

              {/* Cancelled */}
              {order.status === "Cancelled" && (
                <p className="text-xs text-muted-foreground py-1">
                  This order has been cancelled.
                </p>
              )}

              {/* Open but creator - can't fill own order */}
              {order.status === "Open" && isCreator && (
                <p className="text-xs text-muted-foreground py-1">
                  You cannot fill your own order.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
