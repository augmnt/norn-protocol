"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { STAKING_LOOM_ID } from "@/lib/apps-config";
import { useStaking } from "@/hooks/use-staking";
import { ArrowLeft, Landmark, Loader2 } from "lucide-react";
import { toast } from "sonner";

export default function StakePage() {
  const router = useRouter();
  const { stake, loading } = useStaking(STAKING_LOOM_ID);

  const [amount, setAmount] = useState("");

  const canSubmit = parseFloat(amount) > 0;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await stake(amountRaw);
      toast.success("Tokens staked successfully");
      router.push("/apps/staking");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to stake");
    }
  };

  return (
    <PageContainer
      title="Stake Tokens"
      action={
        <Link href="/apps/staking">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
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

            <Button
              onClick={handleSubmit}
              disabled={!canSubmit || loading}
              className="w-full"
            >
              {loading ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <Landmark className="mr-2 h-3.5 w-3.5" />
              )}
              Stake Tokens
            </Button>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
