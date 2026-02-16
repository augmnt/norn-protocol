"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/ui/empty-state";
import { FormButton } from "@/components/ui/form-button";
import { useDiscoverFeed } from "@/hooks/use-discover-feed";
import { useLoomOps } from "@/hooks/use-loom-ops";
import { useWallet } from "@/hooks/use-wallet";
import { useTokenBalances } from "@/hooks/use-token-balances";
import {
  encodeGetPoolCount,
  encodeGetPool,
  encodeGetQuote,
  encodeSwapNornForToken,
  encodeSwapTokenForNorn,
  decodeU64,
  decodePool,
  decodeQuote,
} from "@/lib/borsh-amm";
import type { AmmPool } from "@/lib/borsh-amm";
import { formatAmount, strip0x, truncateAddress } from "@/lib/format";
import { Waves, ArrowDownUp, Loader2 } from "lucide-react";
import { toast } from "sonner";

const DECIMALS = 12;
const SLIPPAGE_BPS = 50;
const NORN_TOKEN_ID = "0".repeat(64);

function parseTokenAmount(value: string): bigint {
  if (!value || value === "0") return 0n;
  const parts = value.split(".");
  const whole = parts[0] || "0";
  const frac = (parts[1] || "").padEnd(DECIMALS, "0").slice(0, DECIMALS);
  return BigInt(whole) * BigInt(10 ** DECIMALS) + BigInt(frac);
}

interface PoolWithLoom extends AmmPool {
  loomId: string;
}

export default function SwapLandingPage() {
  const { activeAddress } = useWallet();
  const { data: feedItems, isLoading: feedLoading } =
    useDiscoverFeed("amm-pool");
  const { queryLoom, executeLoom, loading: txLoading } = useLoomOps();
  const { data: threadState } = useTokenBalances(activeAddress ?? undefined);

  const [allPools, setAllPools] = useState<PoolWithLoom[]>([]);
  const [poolsLoading, setPoolsLoading] = useState(false);
  const [isNornInput, setIsNornInput] = useState(true);
  const [selectedPool, setSelectedPool] = useState<PoolWithLoom | null>(null);
  const [inputAmount, setInputAmount] = useState("");
  const [outputAmount, setOutputAmount] = useState<bigint>(0n);
  const [quoting, setQuoting] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Discover all pools across all AMM instances
  useEffect(() => {
    if (!feedItems || feedItems.length === 0) return;

    const discoverPools = async () => {
      setPoolsLoading(true);
      const pools: PoolWithLoom[] = [];

      for (const item of feedItems) {
        try {
          const countRes = await queryLoom(item.loomId, encodeGetPoolCount());
          if (!countRes?.output_hex) continue;
          const count = decodeU64(countRes.output_hex);
          const limit = count > 20n ? 20n : count;

          for (let i = 0n; i < limit; i++) {
            const poolRes = await queryLoom(
              item.loomId,
              encodeGetPool(i)
            );
            if (poolRes?.output_hex) {
              const pool = decodePool(poolRes.output_hex);
              pools.push({ ...pool, loomId: item.loomId });
            }
          }
        } catch {
          // skip
        }
      }

      setAllPools(pools);
      if (pools.length > 0 && !selectedPool) {
        setSelectedPool(pools[0]);
      }
      setPoolsLoading(false);
    };

    discoverPools();
  }, [feedItems, queryLoom]); // eslint-disable-line react-hooks/exhaustive-deps

  // Debounced quote
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);

    const parsed = parseTokenAmount(inputAmount);
    if (parsed <= 0n || !selectedPool) {
      setOutputAmount(0n);
      return;
    }

    debounceRef.current = setTimeout(async () => {
      setQuoting(true);
      try {
        const res = await queryLoom(
          selectedPool.loomId,
          encodeGetQuote(selectedPool.id, isNornInput, parsed)
        );
        if (res?.output_hex) {
          setOutputAmount(decodeQuote(res.output_hex));
        }
      } catch {
        setOutputAmount(0n);
      } finally {
        setQuoting(false);
      }
    }, 300);

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [inputAmount, isNornInput, selectedPool, queryLoom]);

  const handleFlip = () => {
    setIsNornInput(!isNornInput);
    setInputAmount("");
    setOutputAmount(0n);
  };

  const handleSwap = async () => {
    if (!selectedPool || !activeAddress) return;
    const parsed = parseTokenAmount(inputAmount);
    if (parsed <= 0n) return;

    const minOut = (outputAmount * BigInt(10000 - SLIPPAGE_BPS)) / 10000n;

    try {
      const input = isNornInput
        ? encodeSwapNornForToken(selectedPool.id, parsed, minOut)
        : encodeSwapTokenForNorn(selectedPool.id, parsed, minOut);
      await executeLoom(selectedPool.loomId, input);
      toast.success("Swap executed successfully");
      setInputAmount("");
      setOutputAmount(0n);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Swap failed");
    }
  };

  const tokenDisplay = selectedPool
    ? selectedPool.token === NORN_TOKEN_ID
      ? "NORN"
      : truncateAddress("0x" + selectedPool.token.slice(0, 40))
    : "Token";

  const fromLabel = isNornInput ? "NORN" : tokenDisplay;
  const toLabel = isNornInput ? tokenDisplay : "NORN";

  const canSwap =
    !txLoading &&
    !!selectedPool &&
    parseTokenAmount(inputAmount) > 0n &&
    outputAmount > 0n;

  // No AMM instances deployed
  if (!feedLoading && (!feedItems || feedItems.length === 0)) {
    return (
      <PageContainer title="Swap" description="Instant token swaps via AMM pools">
        <EmptyState
          icon={Waves}
          title="No AMM pools deployed"
          description="Deploy an AMM Pool instance to enable swaps."
          action={
            <Link href="/apps/amm-pool">
              <Button variant="outline" size="sm">
                Deploy AMM Pool
              </Button>
            </Link>
          }
        />
      </PageContainer>
    );
  }

  return (
    <PageContainer title="Swap" description="Instant token swaps via AMM pools">
      <div className="mx-auto max-w-md space-y-4">
        {/* Pool selector */}
        {allPools.length > 1 && (
          <div>
            <label className="text-xs text-muted-foreground">Pool</label>
            <select
              value={selectedPool ? `${selectedPool.loomId}:${selectedPool.id}` : ""}
              onChange={(e) => {
                const [lId, pId] = e.target.value.split(":");
                const pool = allPools.find(
                  (p) => p.loomId === lId && p.id === BigInt(pId)
                );
                if (pool) {
                  setSelectedPool(pool);
                  setInputAmount("");
                  setOutputAmount(0n);
                }
              }}
              className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm font-mono"
            >
              {allPools.map((p) => (
                <option
                  key={`${p.loomId}:${p.id}`}
                  value={`${p.loomId}:${p.id}`}
                >
                  Pool #{p.id.toString()} —{" "}
                  {p.token === NORN_TOKEN_ID
                    ? "NORN"
                    : truncateAddress("0x" + p.token.slice(0, 40))}
                </option>
              ))}
            </select>
          </div>
        )}

        {poolsLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : allPools.length === 0 ? (
          <EmptyState
            icon={Waves}
            title="No pools available"
            description="Create a pool in an AMM instance first."
            action={
              feedItems && feedItems.length > 0 ? (
                <Link href={`/apps/amm-pool/${feedItems[0].loomId}/create`}>
                  <Button variant="outline" size="sm">
                    Create Pool
                  </Button>
                </Link>
              ) : undefined
            }
          />
        ) : (
          <Card>
            <CardContent className="p-4 space-y-3">
              {/* From */}
              <div className="rounded-lg bg-muted p-3">
                <div className="flex items-center justify-between">
                  <span className="text-xs text-muted-foreground">From</span>
                  <span className="text-xs font-medium">{fromLabel}</span>
                </div>
                <Input
                  type="text"
                  inputMode="decimal"
                  placeholder="0.0"
                  value={inputAmount}
                  onChange={(e) => setInputAmount(e.target.value)}
                  className="mt-1.5 border-0 bg-transparent p-0 text-lg font-mono tabular-nums focus-visible:ring-0"
                />
              </div>

              {/* Flip */}
              <div className="flex justify-center">
                <Button
                  variant="outline"
                  size="icon"
                  className="h-8 w-8 rounded-full"
                  onClick={handleFlip}
                >
                  <ArrowDownUp className="h-3.5 w-3.5" />
                </Button>
              </div>

              {/* To */}
              <div className="rounded-lg bg-muted p-3">
                <div className="flex items-center justify-between">
                  <span className="text-xs text-muted-foreground">To</span>
                  <span className="text-xs font-medium">{toLabel}</span>
                </div>
                <div className="mt-1.5">
                  <span className="text-lg font-mono tabular-nums">
                    {quoting ? (
                      <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                    ) : outputAmount > 0n ? (
                      formatAmount(outputAmount.toString())
                    ) : (
                      <span className="text-muted-foreground">0.0</span>
                    )}
                  </span>
                </div>
              </div>

              {/* Info */}
              {outputAmount > 0n && (
                <div className="space-y-1 text-xs text-muted-foreground">
                  <div className="flex justify-between">
                    <span>Min received (0.5% slippage)</span>
                    <span className="font-mono tabular-nums">
                      {formatAmount(
                        (
                          (outputAmount * BigInt(10000 - SLIPPAGE_BPS)) /
                          10000n
                        ).toString()
                      )}{" "}
                      {toLabel}
                    </span>
                  </div>
                </div>
              )}

              <FormButton
                className="w-full"
                onClick={handleSwap}
                disabled={!canSwap}
                disabledReason={
                  !activeAddress
                    ? "Connect wallet"
                    : !selectedPool
                      ? "No pool selected"
                      : parseTokenAmount(inputAmount) <= 0n
                        ? "Enter an amount"
                        : outputAmount <= 0n
                          ? "Insufficient liquidity"
                          : undefined
                }
              >
                {txLoading ? (
                  <>
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    Swapping...
                  </>
                ) : (
                  "Swap"
                )}
              </FormButton>
            </CardContent>
          </Card>
        )}
      </div>
    </PageContainer>
  );
}
