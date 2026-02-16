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
import { Button } from "@/components/ui/button";
import { FormButton } from "@/components/ui/form-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useTimelock } from "@/hooks/use-timelock";
import { truncateHash } from "@/lib/format";
import { Clock, Loader2 } from "lucide-react";
import { toast } from "sonner";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function CreateLockPage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { lock, loading } = useTimelock(loomId);

  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [amount, setAmount] = useState("");
  const [unlockDays, setUnlockDays] = useState("30");

  const canSubmit =
    tokenId.length === 64 &&
    parseFloat(amount) > 0 &&
    parseFloat(unlockDays) > 0;

  const disabledReason = tokenId.length !== 64
    ? "Token ID must be 64 characters"
    : parseFloat(amount) <= 0
      ? "Enter an amount"
      : parseFloat(unlockDays) <= 0
        ? "Lock duration must be greater than 0"
        : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      const unlockTime =
        BigInt(Math.floor(Date.now() / 1000)) +
        BigInt(Math.floor(parseFloat(unlockDays) * 86400));
      await lock(tokenId, amountRaw, unlockTime);
      toast.success("Tokens locked successfully");
      router.push(`/apps/timelock/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to lock tokens");
    }
  };

  return (
    <PageContainer
      title="Create Time Lock"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Time-locked Vault", href: "/apps/timelock" },
        { label: truncateHash(loomId, 8), href: `/apps/timelock/${loomId}` },
        { label: "Create Lock" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Clock className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">New Time Lock</CardTitle>
                <CardDescription>
                  Deposit tokens with a future unlock date. Tokens are locked in
                  the contract and cannot be withdrawn until the unlock time.
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
              <Label className="text-xs text-muted-foreground">
                Lock Duration (days)
              </Label>
              <Input
                type="number"
                value={unlockDays}
                onChange={(e) => setUnlockDays(e.target.value)}
                placeholder="30"
                min="1"
                className="font-mono text-sm tabular-nums"
              />
              <p className="text-[10px] text-muted-foreground">
                Tokens will unlock{" "}
                {parseFloat(unlockDays) > 0
                  ? `in ${unlockDays} day${parseFloat(unlockDays) !== 1 ? "s" : ""}`
                  : "immediately"}
                .
              </p>
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
                <Clock className="mr-2 h-3.5 w-3.5" />
              )}
              Lock Tokens
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
