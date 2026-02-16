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
import { useStaking } from "@/hooks/use-staking";
import { truncateHash } from "@/lib/format";
import { Coins, Loader2 } from "lucide-react";
import { toast } from "sonner";

export default function FundRewardsPage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { fundRewards, loading } = useStaking(loomId);

  const [amount, setAmount] = useState("");

  const canSubmit = parseFloat(amount) > 0;

  const disabledReason = parseFloat(amount) <= 0
    ? "Enter an amount"
    : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await fundRewards(amountRaw);
      toast.success("Reward pool funded successfully");
      router.push(`/apps/staking/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to fund rewards");
    }
  };

  return (
    <PageContainer
      title="Fund Reward Pool"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Staking Vault", href: "/apps/staking" },
        { label: truncateHash(loomId, 8), href: `/apps/staking/${loomId}` },
        { label: "Fund Rewards" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Coins className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Fund Reward Pool</CardTitle>
                <CardDescription>
                  Add tokens to the staking reward pool. These tokens will be
                  distributed to stakers over time. Anyone can fund the pool.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Amount (NORN)
              </Label>
              <Input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
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
                <Coins className="mr-2 h-3.5 w-3.5" />
              )}
              Fund Rewards
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
