"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { ESCROW_LOOM_ID } from "@/lib/apps-config";
import { useEscrow } from "@/hooks/use-escrow";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, formatAmount } from "@/lib/format";
import {
  Plus,
  ShieldCheck,
  ArrowLeft,
  AlertCircle,
  Loader2,
} from "lucide-react";
import type { Deal } from "@/lib/borsh-escrow";

const STATUS_VARIANT: Record<string, "norn" | "destructive" | "secondary"> = {
  Created: "norn",
  Funded: "norn",
  Delivered: "norn",
  Completed: "secondary",
  Disputed: "destructive",
  Cancelled: "destructive",
  Refunded: "secondary",
};

function DealCard({ deal }: { deal: Deal }) {
  return (
    <Link href={`/apps/escrow/${deal.id}`}>
      <Card className="transition-colors hover:border-norn/30">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Deal #{deal.id.toString()}
            </span>
            <Badge variant={STATUS_VARIANT[deal.status] ?? "secondary"}>
              {deal.status}
            </Badge>
          </div>
          <p className="mt-2 text-sm truncate">{deal.description}</p>
          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <div className="flex items-center gap-3">
              <span>
                Buyer:{" "}
                <span className="font-mono">{truncateAddress(deal.buyer)}</span>
              </span>
              <span>
                Seller:{" "}
                <span className="font-mono">
                  {truncateAddress(deal.seller)}
                </span>
              </span>
            </div>
            <span className="font-mono tabular-nums">
              {formatAmount(deal.amount.toString())}
            </span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}

export default function EscrowDashboardPage() {
  const { activeAddress } = useWallet();
  const { getDeal, getDealCount, loading } = useEscrow(ESCROW_LOOM_ID);
  const [deals, setDeals] = useState<Deal[]>([]);
  const [fetching, setFetching] = useState(false);

  const fetchDeals = useCallback(async () => {
    if (!ESCROW_LOOM_ID) return;
    setFetching(true);
    try {
      const count = await getDealCount();
      const fetched: Deal[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const deal = await getDeal(i);
        if (deal) fetched.push(deal);
      }
      setDeals(fetched);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getDeal, getDealCount]);

  useEffect(() => {
    fetchDeals();
  }, [fetchDeals]);

  useLoomRefresh(ESCROW_LOOM_ID, fetchDeals);

  const addr = activeAddress?.toLowerCase() ?? "";
  const activeDeals = deals.filter(
    (d) =>
      d.status === "Created" || d.status === "Funded" || d.status === "Delivered"
  );
  const myDeals = deals.filter(
    (d) => d.buyer.toLowerCase() === addr || d.seller.toLowerCase() === addr
  );

  if (!ESCROW_LOOM_ID) {
    return (
      <PageContainer title="P2P Escrow">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              Escrow contract not configured. Set{" "}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                NEXT_PUBLIC_ESCROW_LOOM_ID
              </code>{" "}
              in your environment.
            </div>
          </CardContent>
        </Card>
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="P2P Escrow"
      description="Secure peer-to-peer deals with on-chain escrow"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href="/apps/escrow/create">
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Create Deal
            </Button>
          </Link>
        </div>
      }
    >
      <Tabs defaultValue="active" className="space-y-4">
        <TabsList>
          <TabsTrigger value="active">
            Active ({activeDeals.length})
          </TabsTrigger>
          <TabsTrigger value="mine">
            My Deals ({myDeals.length})
          </TabsTrigger>
          <TabsTrigger value="all">All ({deals.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="active" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : activeDeals.length === 0 ? (
            <EmptyState
              icon={ShieldCheck}
              title="No active deals"
              description="Create a deal to get started."
              action={
                <Link href="/apps/escrow/create">
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    Create Deal
                  </Button>
                </Link>
              }
            />
          ) : (
            activeDeals
              .slice()
              .reverse()
              .map((deal) => <DealCard key={deal.id.toString()} deal={deal} />)
          )}
        </TabsContent>

        <TabsContent value="mine" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : myDeals.length === 0 ? (
            <EmptyState
              icon={ShieldCheck}
              title="No deals found"
              description="Deals where you are the buyer or seller will appear here."
              action={
                <Link href="/apps/escrow/create">
                  <Button variant="outline" size="sm">
                    <Plus className="mr-1.5 h-3.5 w-3.5" />
                    Create Deal
                  </Button>
                </Link>
              }
            />
          ) : (
            myDeals
              .slice()
              .reverse()
              .map((deal) => <DealCard key={deal.id.toString()} deal={deal} />)
          )}
        </TabsContent>

        <TabsContent value="all" className="space-y-3">
          {fetching || loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : deals.length === 0 ? (
            <EmptyState
              icon={ShieldCheck}
              title="No deals found"
              description="Create a deal to get started."
            />
          ) : (
            deals
              .slice()
              .reverse()
              .map((deal) => <DealCard key={deal.id.toString()} deal={deal} />)
          )}
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
