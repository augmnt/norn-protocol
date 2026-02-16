"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FormButton } from "@/components/ui/form-button";
import { useAmm } from "@/hooks/use-amm";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { formatAmount, truncateAddress, truncateHash } from "@/lib/format";
import type { AmmPool, AmmConfig } from "@/lib/borsh-amm";
import { ArrowDownUp, Loader2 } from "lucide-react";
import { toast } from "sonner";

const DECIMALS = 12;
const SLIPPAGE_BPS = 50; // 0.5%

function parseTokenAmount(value: string): bigint {
  if (!value || value === "0") return 0n;
  const parts = value.split(".");
  const whole = parts[0] || "0";
  const frac = (parts[1] || "").padEnd(DECIMALS, "0").slice(0, DECIMALS);
  return BigInt(whole) * BigInt(10 ** DECIMALS) + BigInt(frac);
}

export default function SwapPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const poolId = BigInt(params.poolId as string);
  const { activeAddress } = useWallet();
  const {
    getPool,
    getQuote,
    getConfig,
    swapNornForToken,
    swapTokenForNorn,
    loading,
  } = useAmm(loomId);

  const [pool, setPool] = useState<AmmPool | null>(null);
  const [config, setConfig] = useState<AmmConfig | null>(null);
  const [isNornInput, setIsNornInput] = useState(true);
  const [inputAmount, setInputAmount] = useState("");
  const [outputAmount, setOutputAmount] = useState<bigint>(0n);
  const [quoting, setQuoting] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const tokenDisplay = pool
    ? pool.token === "0".repeat(64)
      ? "NORN"
      : truncateAddress("0x" + pool.token.slice(0, 40))
    : "...";

  const fetchPoolData = useCallback(async () => {
    const [p, c] = await Promise.all([getPool(poolId), getConfig()]);
    if (p) setPool(p);
    if (c) setConfig(c);
  }, [getPool, getConfig, poolId]);

  useEffect(() => {
    fetchPoolData();
  }, [fetchPoolData]);

  useLoomRefresh(loomId, fetchPoolData);

  // Debounced quote
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);

    const parsed = parseTokenAmount(inputAmount);
    if (parsed <= 0n) {
      setOutputAmount(0n);
      return;
    }

    debounceRef.current = setTimeout(async () => {
      setQuoting(true);
      try {
        const out = await getQuote(poolId, isNornInput, parsed);
        setOutputAmount(out);
      } catch {
        setOutputAmount(0n);
      } finally {
        setQuoting(false);
      }
    }, 300);

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [inputAmount, isNornInput, getQuote, poolId]);

  const handleFlip = () => {
    setIsNornInput(!isNornInput);
    setInputAmount("");
    setOutputAmount(0n);
  };

  const handleSwap = async () => {
    const parsed = parseTokenAmount(inputAmount);
    if (parsed <= 0n) return;

    // Apply slippage to min output
    const minOut = (outputAmount * BigInt(10000 - SLIPPAGE_BPS)) / 10000n;

    try {
      if (isNornInput) {
        await swapNornForToken(poolId, parsed, minOut);
      } else {
        await swapTokenForNorn(poolId, parsed, minOut);
      }
      toast.success("Swap executed successfully");
      setInputAmount("");
      setOutputAmount(0n);
      fetchPoolData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Swap failed");
    }
  };

  const fromLabel = isNornInput ? "NORN" : tokenDisplay;
  const toLabel = isNornInput ? tokenDisplay : "NORN";

  const price =
    pool && pool.reserveToken > 0n
      ? isNornInput
        ? Number(pool.reserveToken) / Number(pool.reserveNorn)
        : Number(pool.reserveNorn) / Number(pool.reserveToken)
      : 0;

  const canSwap = !loading && parseTokenAmount(inputAmount) > 0n && outputAmount > 0n;
  const disabledReason = !activeAddress
    ? "Connect wallet"
    : parseTokenAmount(inputAmount) <= 0n
      ? "Enter an amount"
      : outputAmount <= 0n
        ? "Insufficient liquidity"
        : undefined;

  return (
    <PageContainer
      title="Swap"
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "AMM Pool", href: "/apps/amm-pool"}, {label: truncateHash(loomId, 8), href: `/apps/amm-pool/${loomId}`}, {label: "Swap"}]}
    >
      <div className="mx-auto max-w-md space-y-4">
        {/* Swap card */}
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
              <div className="mt-1.5 flex items-center gap-2">
                <span className="text-lg font-mono tabular-nums text-foreground">
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
                  <span>Rate</span>
                  <span className="font-mono tabular-nums">
                    1 {fromLabel} = {price.toFixed(4)} {toLabel}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Min received (0.5% slippage)</span>
                  <span className="font-mono tabular-nums">
                    {formatAmount(
                      ((outputAmount * BigInt(10000 - SLIPPAGE_BPS)) / 10000n).toString()
                    )}{" "}
                    {toLabel}
                  </span>
                </div>
                {config && (
                  <div className="flex justify-between">
                    <span>Fee</span>
                    <span>{(config.feeBps / 100).toFixed(1)}%</span>
                  </div>
                )}
              </div>
            )}

            <FormButton
              className="w-full"
              onClick={handleSwap}
              disabled={!canSwap}
              disabledReason={disabledReason}
            >
              {loading ? (
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

        {/* Pool stats */}
        {pool && (
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Pool Stats</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-xs">
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
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token</span>
                <span className="font-mono">{tokenDisplay}</span>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </PageContainer>
  );
}
