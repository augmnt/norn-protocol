"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useSwap } from "@/hooks/use-swap";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount } from "@/lib/format";
import {
  Plus,
  ArrowLeftRight,
  ArrowLeft,
  Loader2,
} from "lucide-react";
import type { SwapOrder, OrderStatus } from "@/lib/borsh-swap";

const STATUS_VARIANT: Record<
  OrderStatus,
  "norn" | "secondary" | "destructive"
> = {
  Open: "norn",
  Filled: "secondary",
  Cancelled: "destructive",
};

function OrderCard({ order, loomId }: { order: SwapOrder; loomId: string }) {
  return (
    <Link href={`/apps/swap/${loomId}/${order.id.toString()}`}>
      <Card className="transition-colors hover:border-norn/30">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Order #{order.id.toString()}
            </span>
            <Badge variant={STATUS_VARIANT[order.status]}>
              {order.status}
            </Badge>
          </div>

          <div className="mt-3 flex items-center gap-2">
            <div className="flex-1 rounded-lg bg-muted p-2">
              <p className="text-[10px] text-muted-foreground">Selling</p>
              <p className="mt-0.5 font-mono text-sm tabular-nums">
                {formatAmount(order.sellAmount.toString())}
              </p>
              <p className="font-mono text-[10px] text-muted-foreground truncate">
                {truncateAddress("0x" + order.sellToken.slice(0, 40))}
              </p>
            </div>
            <ArrowLeftRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
            <div className="flex-1 rounded-lg bg-muted p-2">
              <p className="text-[10px] text-muted-foreground">Buying</p>
              <p className="mt-0.5 font-mono text-sm tabular-nums">
                {formatAmount(order.buyAmount.toString())}
              </p>
              <p className="font-mono text-[10px] text-muted-foreground truncate">
                {truncateAddress("0x" + order.buyToken.slice(0, 40))}
              </p>
            </div>
          </div>

          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <span>
              Creator:{" "}
              <span className="font-mono">
                {truncateAddress(order.creator)}
              </span>
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function SwapDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getOrder, getOrderCount, loading } = useSwap(loomId);
  const [orders, setOrders] = useState<SwapOrder[]>([]);
  const [fetching, setFetching] = useState(false);

  const fetchOrders = useCallback(async () => {
    if (!loomId) return;
    setFetching(true);
    try {
      const count = await getOrderCount();
      const fetched: SwapOrder[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const o = await getOrder(i);
        if (o) fetched.push(o);
      }
      setOrders(fetched);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getOrder, getOrderCount, loomId]);

  useEffect(() => {
    fetchOrders();
  }, [fetchOrders]);

  useLoomRefresh(loomId, fetchOrders);

  const addr = activeAddress?.toLowerCase() ?? "";
  const myOrders = orders.filter((o) => o.creator.toLowerCase() === addr);
  const openOrders = orders.filter((o) => o.status === "Open");

  return (
    <PageContainer
      title="OTC Swap"
      description="Post and fill token swap orders at fixed rates"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/swap">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href={`/apps/swap/${loomId}/create`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Create Order
            </Button>
          </Link>
        </div>
      }
    >
      <Tabs defaultValue="open" className="space-y-4">
        <TabsList>
          <TabsTrigger value="open">Open ({openOrders.length})</TabsTrigger>
          <TabsTrigger value="mine">My Orders ({myOrders.length})</TabsTrigger>
          <TabsTrigger value="all">All ({orders.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="open" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : openOrders.length === 0 ? (
            <EmptyState
              icon={ArrowLeftRight}
              title="No open orders"
              description="Create an order to start trading."
              action={
                <Link href={`/apps/swap/${loomId}/create`}>
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    Create Order
                  </Button>
                </Link>
              }
            />
          ) : (
            openOrders
              .slice()
              .reverse()
              .map((o) => <OrderCard key={o.id.toString()} order={o} loomId={loomId} />)
          )}
        </TabsContent>

        <TabsContent value="mine" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : myOrders.length === 0 ? (
            <EmptyState
              icon={ArrowLeftRight}
              title="No orders created"
              description="Create a swap order to get started."
              action={
                <Link href={`/apps/swap/${loomId}/create`}>
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    Create Order
                  </Button>
                </Link>
              }
            />
          ) : (
            myOrders
              .slice()
              .reverse()
              .map((o) => <OrderCard key={o.id.toString()} order={o} loomId={loomId} />)
          )}
        </TabsContent>

        <TabsContent value="all" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : orders.length === 0 ? (
            <EmptyState
              icon={ArrowLeftRight}
              title="No orders found"
              description="Create a swap order to get started."
            />
          ) : (
            orders
              .slice()
              .reverse()
              .map((o) => <OrderCard key={o.id.toString()} order={o} loomId={loomId} />)
          )}
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
