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
import { useTreasury } from "@/hooks/use-treasury";
import { truncateHash } from "@/lib/format";
import { Download, Loader2 } from "lucide-react";
import { toast } from "sonner";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function DepositPage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { deposit, loading } = useTreasury(loomId);

  const [amount, setAmount] = useState("");
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);

  const canSubmit = parseFloat(amount) > 0;

  const disabledReason = parseFloat(amount) <= 0
    ? "Enter an amount"
    : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await deposit(tokenId, amountRaw);
      toast.success("Deposit successful");
      router.push(`/apps/treasury/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to deposit");
    }
  };

  return (
    <PageContainer
      title="Deposit"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Multisig Treasury", href: "/apps/treasury" },
        { label: truncateHash(loomId, 8), href: `/apps/treasury/${loomId}` },
        { label: "Deposit" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Download className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Deposit to Treasury</CardTitle>
                <CardDescription>
                  Send tokens to the shared treasury. Anyone can deposit.
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

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Token ID</Label>
              <Input
                value={tokenId}
                onChange={(e) => setTokenId(e.target.value)}
                placeholder="64 hex chars (native = all zeros)"
                className="font-mono text-xs"
              />
              <p className="text-[10px] text-muted-foreground">
                Leave default for native NORN token.
              </p>
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
                <Download className="mr-2 h-3.5 w-3.5" />
              )}
              Deposit
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
