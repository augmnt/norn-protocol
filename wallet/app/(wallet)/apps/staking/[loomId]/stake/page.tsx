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
import { Landmark, Loader2 } from "lucide-react";
import { toast } from "sonner";

export default function StakePage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { stake, loading } = useStaking(loomId);

  const [amount, setAmount] = useState("");

  const canSubmit = parseFloat(amount) > 0;

  const disabledReason = parseFloat(amount) <= 0
    ? "Enter an amount"
    : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await stake(amountRaw);
      toast.success("Tokens staked successfully");
      router.push(`/apps/staking/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to stake");
    }
  };

  return (
    <PageContainer
      title="Stake Tokens"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Staking Vault", href: "/apps/staking" },
        { label: truncateHash(loomId, 8), href: `/apps/staking/${loomId}` },
        { label: "Stake" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Landmark className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Stake Tokens</CardTitle>
                <CardDescription>
                  Lock tokens in the staking vault to earn rewards over time.
                  Tokens are transferred to the contract on staking.
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
                <Landmark className="mr-2 h-3.5 w-3.5" />
              )}
              Stake Tokens
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
