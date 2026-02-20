"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { FormButton } from "@/components/ui/form-button";
import { useAmm } from "@/hooks/use-amm";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { useTokenSymbol } from "@/hooks/use-token-symbol";
import { formatAmount, truncateHash } from "@/lib/format";
import type { AmmPool } from "@/lib/borsh-amm";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";

const DECIMALS = 12;

function parseTokenAmount(value: string): bigint {
  if (!value || value === "0") return 0n;
  const parts = value.split(".");
  const whole = parts[0] || "0";
  const frac = (parts[1] || "").padEnd(DECIMALS, "0").slice(0, DECIMALS);
  return BigInt(whole) * BigInt(10 ** DECIMALS) + BigInt(frac);
}

export default function LiquidityPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const poolId = BigInt(params.poolId as string);
  const { activeAddress } = useWallet();
  const { getPool, getLpBalance, addLiquidity, removeLiquidity, loading } =
    useAmm(loomId);

  const [pool, setPool] = useState<AmmPool | null>(null);
  const [lpBalance, setLpBalance] = useState<bigint>(0n);
  const [lpTotal, setLpTotal] = useState<bigint>(0n);

  // Add liquidity inputs
  const [nornAmount, setNornAmount] = useState("");
  const [tokenAmount, setTokenAmount] = useState("");

  // Remove liquidity input
  const [lpAmount, setLpAmount] = useState("");

  const tokenDisplay = useTokenSymbol(pool?.token);

  const fetchData = useCallback(async () => {
    const p = await getPool(poolId);
    if (p) setPool(p);
    if (activeAddress) {
      const bal = await getLpBalance(poolId, activeAddress);
      setLpBalance(bal);
    }
  }, [getPool, getLpBalance, poolId, activeAddress]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  // Auto-calculate second amount on add liquidity
  useEffect(() => {
    if (!pool || pool.reserveNorn === 0n || pool.reserveToken === 0n) return;
    const parsed = parseTokenAmount(nornAmount);
    if (parsed > 0n) {
      const calculated = (parsed * pool.reserveToken) / pool.reserveNorn;
      const whole = calculated / BigInt(10 ** DECIMALS);
      const frac = calculated % BigInt(10 ** DECIMALS);
      const fracStr = frac.toString().padStart(DECIMALS, "0").replace(/0+$/, "");
      setTokenAmount(fracStr ? `${whole}.${fracStr}` : whole.toString());
    }
  }, [nornAmount, pool]);

  const handleAdd = async () => {
    const norn = parseTokenAmount(nornAmount);
    const token = parseTokenAmount(tokenAmount);
    if (norn <= 0n || token <= 0n) return;

    try {
      await addLiquidity(poolId, norn, token);
      toast.success("Liquidity added successfully");
      setNornAmount("");
      setTokenAmount("");
      fetchData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to add liquidity");
    }
  };

  const handleRemove = async () => {
    const lp = parseTokenAmount(lpAmount);
    if (lp <= 0n) return;

    try {
      await removeLiquidity(poolId, lp);
      toast.success("Liquidity removed successfully");
      setLpAmount("");
      fetchData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to remove liquidity");
    }
  };

  const handleMax = () => {
    if (lpBalance > 0n) {
      const whole = lpBalance / BigInt(10 ** DECIMALS);
      const frac = lpBalance % BigInt(10 ** DECIMALS);
      const fracStr = frac.toString().padStart(DECIMALS, "0").replace(/0+$/, "");
      setLpAmount(fracStr ? `${whole}.${fracStr}` : whole.toString());
    }
  };

  // Position calculations
  const sharePercent =
    lpBalance > 0n && pool
      ? (() => {
          // Estimate total LP from pool reserves and user balance
          // We need total LP, approximate from user share
          const totalReserve = pool.reserveNorn + pool.reserveToken;
          return totalReserve > 0n ? Number(lpBalance) : 0;
        })()
      : 0;

  const canAdd =
    !loading &&
    parseTokenAmount(nornAmount) > 0n &&
    parseTokenAmount(tokenAmount) > 0n;
  const canRemove = !loading && parseTokenAmount(lpAmount) > 0n;

  return (
    <PageContainer
      title="Liquidity"
      description={`Pool #${poolId.toString()}`}
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "AMM Pool", href: "/apps/amm-pool"}, {label: truncateHash(loomId, 8), href: `/apps/amm-pool/${loomId}`}, {label: `Pool #${poolId.toString()}`}]}
    >
      <div className="mx-auto max-w-md space-y-4">
        <Tabs defaultValue="add">
          <TabsList className="w-full">
            <TabsTrigger value="add" className="flex-1">
              Add
            </TabsTrigger>
            <TabsTrigger value="remove" className="flex-1">
              Remove
            </TabsTrigger>
          </TabsList>

          <TabsContent value="add" className="mt-4 space-y-3">
            <Card>
              <CardContent className="p-4 space-y-3">
                <div>
                  <label className="text-xs text-muted-foreground">
                    NORN Amount
                  </label>
                  <Input
                    type="text"
                    inputMode="decimal"
                    placeholder="0.0"
                    value={nornAmount}
                    onChange={(e) => setNornAmount(e.target.value)}
                    className="mt-1 font-mono"
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">
                    Token Amount ({tokenDisplay})
                  </label>
                  <Input
                    type="text"
                    inputMode="decimal"
                    placeholder="0.0"
                    value={tokenAmount}
                    onChange={(e) => setTokenAmount(e.target.value)}
                    className="mt-1 font-mono"
                  />
                </div>

                <FormButton
                  className="w-full"
                  onClick={handleAdd}
                  disabled={!canAdd}
                  disabledReason={
                    !activeAddress
                      ? "Connect wallet"
                      : parseTokenAmount(nornAmount) <= 0n
                        ? "Enter NORN amount"
                        : undefined
                  }
                >
                  {loading ? (
                    <>
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      Adding...
                    </>
                  ) : (
                    "Add Liquidity"
                  )}
                </FormButton>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="remove" className="mt-4 space-y-3">
            <Card>
              <CardContent className="p-4 space-y-3">
                <div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs text-muted-foreground">
                      LP Tokens to Burn
                    </label>
                    <button
                      onClick={handleMax}
                      className="text-xs text-norn hover:underline"
                    >
                      Max
                    </button>
                  </div>
                  <Input
                    type="text"
                    inputMode="decimal"
                    placeholder="0.0"
                    value={lpAmount}
                    onChange={(e) => setLpAmount(e.target.value)}
                    className="mt-1 font-mono"
                  />
                </div>

                <FormButton
                  className="w-full"
                  onClick={handleRemove}
                  disabled={!canRemove}
                  disabledReason={
                    !activeAddress
                      ? "Connect wallet"
                      : lpBalance === 0n
                        ? "No LP tokens"
                        : undefined
                  }
                >
                  {loading ? (
                    <>
                      <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                      Removing...
                    </>
                  ) : (
                    "Remove Liquidity"
                  )}
                </FormButton>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

        {/* Position card */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Your Position</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-muted-foreground">LP Token Balance</span>
              <span className="font-mono tabular-nums">
                {formatAmount(lpBalance.toString())}
              </span>
            </div>
            {pool && (
              <>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">NORN Reserve</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(pool.reserveNorn.toString())}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Token Reserve</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(pool.reserveToken.toString())}
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
