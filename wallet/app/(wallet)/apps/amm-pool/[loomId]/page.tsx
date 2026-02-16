"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { useAmm } from "@/hooks/use-amm";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { formatAmount, truncateAddress } from "@/lib/format";
import type { AmmPool } from "@/lib/borsh-amm";
import {
  Plus,
  Waves,
  ArrowLeft,
  ArrowLeftRight,
  Droplets,
  Loader2,
} from "lucide-react";

function PoolCard({ pool, loomId }: { pool: AmmPool; loomId: string }) {
  const tokenDisplay = pool.token === "0".repeat(64)
    ? "NORN"
    : truncateAddress("0x" + pool.token.slice(0, 40));

  const price =
    pool.reserveToken > 0n
      ? Number(pool.reserveNorn) / Number(pool.reserveToken)
      : 0;

  return (
    <Card className="transition-colors hover:border-norn/30">
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">
            Pool #{pool.id.toString()}
          </span>
          <span className="font-mono text-xs text-muted-foreground">
            {tokenDisplay}
          </span>
        </div>

        <div className="mt-3 flex items-center gap-2">
          <div className="flex-1 rounded-lg bg-muted p-2">
            <p className="text-[10px] text-muted-foreground">NORN Reserve</p>
            <p className="mt-0.5 font-mono text-sm tabular-nums">
              {formatAmount(pool.reserveNorn.toString())}
            </p>
          </div>
          <ArrowLeftRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <div className="flex-1 rounded-lg bg-muted p-2">
            <p className="text-[10px] text-muted-foreground">Token Reserve</p>
            <p className="mt-0.5 font-mono text-sm tabular-nums">
              {formatAmount(pool.reserveToken.toString())}
            </p>
          </div>
        </div>

        <div className="mt-3 flex items-center justify-between">
          <span className="text-xs text-muted-foreground">
            Price: <span className="font-mono tabular-nums">{price.toFixed(4)}</span> NORN/Token
          </span>
          <div className="flex items-center gap-1.5">
            <Link href={`/apps/amm-pool/${loomId}/swap/${pool.id.toString()}`}>
              <Button variant="outline" size="sm">
                <ArrowLeftRight className="mr-1 h-3 w-3" />
                Swap
              </Button>
            </Link>
            <Link href={`/apps/amm-pool/${loomId}/pool/${pool.id.toString()}`}>
              <Button variant="outline" size="sm">
                <Droplets className="mr-1 h-3 w-3" />
                Liquidity
              </Button>
            </Link>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export default function AmmPoolDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { getPool, getPoolCount } = useAmm(loomId);
  const [pools, setPools] = useState<AmmPool[]>([]);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchPools = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const count = await getPoolCount();
      const fetched: AmmPool[] = [];
      const limit = count > 50n ? 50n : count;
      for (let i = 0n; i < limit; i++) {
        const p = await getPool(i);
        if (p) fetched.push(p);
      }
      setPools(fetched);
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getPool, getPoolCount, loomId]);

  useEffect(() => {
    fetchPools();
  }, [fetchPools]);

  useLoomRefresh(loomId, fetchPools);

  return (
    <PageContainer
      title="AMM Pool"
      description="Automated market maker with constant-product liquidity pools"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/amm-pool">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          <Link href={`/apps/amm-pool/${loomId}/create`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Create Pool
            </Button>
          </Link>
        </div>
      }
    >
      <div className="space-y-3">
        {fetching ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : pools.length === 0 ? (
          <EmptyState
            icon={Waves}
            title="No pools yet"
            description="Create a liquidity pool to enable token swaps."
            action={
              <Link href={`/apps/amm-pool/${loomId}/create`}>
                <Button variant="outline" size="sm">
                  <Plus className="mr-1.5 h-3.5 w-3.5" />
                  Create Pool
                </Button>
              </Link>
            }
          />
        ) : (
          pools
            .slice()
            .reverse()
            .map((p) => (
              <PoolCard key={p.id.toString()} pool={p} loomId={loomId} />
            ))
        )}
      </div>
    </PageContainer>
  );
}
