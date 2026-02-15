"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useParams } from "next/navigation";
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
import { FormButton } from "@/components/ui/form-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useCrowdfund } from "@/hooks/use-crowdfund";
import { ArrowLeft, HandCoins, Loader2 } from "lucide-react";
import { toast } from "sonner";

export default function CrowdfundContributePage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { contribute, loading } = useCrowdfund(loomId);

  const [amount, setAmount] = useState("");

  const canSubmit = parseFloat(amount) > 0;

  const disabledReason = parseFloat(amount) <= 0
    ? "Enter an amount"
    : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await contribute(amountRaw);
      toast.success("Contribution successful");
      router.push(`/apps/crowdfund/${loomId}`);
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to contribute"
      );
    }
  };

  return (
    <PageContainer
      title="Contribute to Campaign"
      action={
        <Link href={`/apps/crowdfund/${loomId}`}>
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
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <HandCoins className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Contribute</CardTitle>
                <CardDescription>
                  Support this campaign by contributing funds. If the goal
                  is not met by the deadline, you can claim a full refund.
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
                <HandCoins className="mr-2 h-3.5 w-3.5" />
              )}
              Contribute
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
