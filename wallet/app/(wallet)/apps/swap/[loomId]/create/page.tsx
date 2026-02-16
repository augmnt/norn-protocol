"use client";

import { useState } from "react";
import { useRouter, useParams } from "next/navigation";
import { PageContainer } from "@/components/ui/page-container";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { FormButton } from "@/components/ui/form-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { truncateHash } from "@/lib/format";
import { useSwap } from "@/hooks/use-swap";
import { ArrowLeftRight, Loader2 } from "lucide-react";
import { toast } from "sonner";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function CreateOrderPage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { createOrder, loading } = useSwap(loomId);

  const [sellToken, setSellToken] = useState(NATIVE_TOKEN_ID);
  const [sellAmount, setSellAmount] = useState("");
  const [buyToken, setBuyToken] = useState("");
  const [buyAmount, setBuyAmount] = useState("");

  const canSubmit =
    sellToken.length === 64 &&
    buyToken.length === 64 &&
    parseFloat(sellAmount) > 0 &&
    parseFloat(buyAmount) > 0;

  const disabledReason = sellToken.length !== 64
    ? "Sell token ID must be 64 characters"
    : parseFloat(sellAmount) <= 0
      ? "Enter a sell amount"
      : buyToken.length !== 64
        ? "Buy token ID must be 64 characters"
        : parseFloat(buyAmount) <= 0
          ? "Enter a buy amount"
          : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const sellRaw = BigInt(Math.floor(parseFloat(sellAmount) * 1e12));
      const buyRaw = BigInt(Math.floor(parseFloat(buyAmount) * 1e12));
      await createOrder(sellToken, sellRaw, buyToken, buyRaw);
      toast.success("Swap order created successfully");
      router.push(`/apps/swap/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to create order");
    }
  };

  return (
    <PageContainer
      title="Create Swap Order"
      breadcrumb={[{label: "Apps", href: "/discover"}, {label: "OTC Swap", href: "/apps/swap"}, {label: truncateHash(loomId, 8), href: `/apps/swap/${loomId}`}, {label: "Create Order"}]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <ArrowLeftRight className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">New Swap Order</CardTitle>
                <CardDescription>
                  Post an offer to trade one token for another at a fixed rate.
                  Tokens are held in escrow until the order is filled or
                  cancelled.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Sell Token ID
              </Label>
              <Input
                value={sellToken}
                onChange={(e) => setSellToken(e.target.value)}
                placeholder="64 hex chars (native = all zeros)"
                className="font-mono text-xs"
              />
              <p className="text-[10px] text-muted-foreground">
                Leave default for native NORN token.
              </p>
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Sell Amount (NORN)
              </Label>
              <Input
                type="number"
                value={sellAmount}
                onChange={(e) => setSellAmount(e.target.value)}
                placeholder="0.00"
                min="0"
                step="any"
                className="font-mono text-sm tabular-nums"
              />
            </div>

            <div className="flex items-center justify-center py-1">
              <ArrowLeftRight className="h-4 w-4 text-muted-foreground" />
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Buy Token ID
              </Label>
              <Input
                value={buyToken}
                onChange={(e) => setBuyToken(e.target.value)}
                placeholder="64 hex chars"
                className="font-mono text-xs"
              />
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Buy Amount
              </Label>
              <Input
                type="number"
                value={buyAmount}
                onChange={(e) => setBuyAmount(e.target.value)}
                placeholder="0.00"
                min="0"
                step="any"
                className="font-mono text-sm tabular-nums"
              />
            </div>

            <FormButton
              onClick={handleSubmit}
              disabled={!canSubmit || loading}
              disabledReason={disabledReason}
              className="w-full"
            >
              {loading ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <ArrowLeftRight className="mr-2 h-3.5 w-3.5" />
              )}
              Create Order
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
