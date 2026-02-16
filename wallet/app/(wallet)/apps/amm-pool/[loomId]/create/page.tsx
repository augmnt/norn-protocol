"use client";

import { useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { FormButton } from "@/components/ui/form-button";
import { useAmm } from "@/hooks/use-amm";
import { useWallet } from "@/hooks/use-wallet";
import { useTokenBalances } from "@/hooks/use-token-balances";
import { formatAmount, strip0x, truncateAddress, truncateHash } from "@/lib/format";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";

const DECIMALS = 12;
const NORN_TOKEN_ID = "0".repeat(64);

function parseTokenAmount(value: string): bigint {
  if (!value || value === "0") return 0n;
  const parts = value.split(".");
  const whole = parts[0] || "0";
  const frac = (parts[1] || "").padEnd(DECIMALS, "0").slice(0, DECIMALS);
  return BigInt(whole) * BigInt(10 ** DECIMALS) + BigInt(frac);
}

export default function CreatePoolPage() {
  const params = useParams();
  const router = useRouter();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { createPool, loading } = useAmm(loomId);
  const { data: threadState } = useTokenBalances(activeAddress ?? undefined);

  const [selectedToken, setSelectedToken] = useState("");
  const [nornAmount, setNornAmount] = useState("");
  const [tokenAmount, setTokenAmount] = useState("");

  // Get user's token balances, excluding NORN
  const tokenBalances = (threadState?.balances ?? []).filter(
    (b) => strip0x(b.token_id) !== NORN_TOKEN_ID
  );

  const nornParsed = parseTokenAmount(nornAmount);
  const tokenParsed = parseTokenAmount(tokenAmount);

  const price =
    nornParsed > 0n && tokenParsed > 0n
      ? Number(nornParsed) / Number(tokenParsed)
      : 0;

  const canCreate =
    !loading && !!selectedToken && nornParsed > 0n && tokenParsed > 0n;
  const disabledReason = !activeAddress
    ? "Connect wallet"
    : !selectedToken
      ? "Select a token"
      : nornParsed <= 0n
        ? "Enter NORN amount"
        : tokenParsed <= 0n
          ? "Enter token amount"
          : undefined;

  const handleCreate = async () => {
    if (!canCreate) return;

    try {
      await createPool(selectedToken, nornParsed, tokenParsed);
      toast.success("Pool created successfully");
      router.push(`/apps/amm-pool/${loomId}`);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to create pool"
      );
    }
  };

  return (
    <PageContainer
      title="Create Pool"
      description="Create a new NORN liquidity pair"
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "AMM Pool", href: "/apps/amm-pool"}, {label: truncateHash(loomId, 8), href: `/apps/amm-pool/${loomId}`}, {label: "Create Pool"}]}
    >
      <div className="mx-auto max-w-md">
        <Card>
          <CardContent className="p-4 space-y-4">
            {/* Token selector */}
            <div>
              <label className="text-xs text-muted-foreground">
                Token to Pair with NORN
              </label>
              <select
                value={selectedToken}
                onChange={(e) => setSelectedToken(e.target.value)}
                className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm font-mono"
              >
                <option value="">Select a token...</option>
                {tokenBalances.map((b) => (
                  <option key={b.token_id} value={strip0x(b.token_id)}>
                    {truncateAddress(b.token_id)} (
                    {formatAmount(b.amount)})
                  </option>
                ))}
              </select>
            </div>

            {/* NORN amount */}
            <div>
              <label className="text-xs text-muted-foreground">
                Initial NORN Amount
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

            {/* Token amount */}
            <div>
              <label className="text-xs text-muted-foreground">
                Initial Token Amount
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

            {/* Price display */}
            {price > 0 && (
              <div className="rounded-lg bg-muted p-3 text-xs">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Initial Price</span>
                  <span className="font-mono tabular-nums">
                    {price.toFixed(4)} NORN per Token
                  </span>
                </div>
              </div>
            )}

            <FormButton
              className="w-full"
              onClick={handleCreate}
              disabled={!canCreate}
              disabledReason={disabledReason}
            >
              {loading ? (
                <>
                  <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                  Creating Pool...
                </>
              ) : (
                "Create Pool"
              )}
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
