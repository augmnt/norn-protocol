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
import { FieldError } from "@/components/ui/field-error";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useVesting } from "@/hooks/use-vesting";
import { isValidAddress, truncateHash } from "@/lib/format";
import { Hourglass, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function CreateSchedulePage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { createSchedule, loading } = useVesting(loomId);

  const [beneficiary, setBeneficiary] = useState("");
  const [amount, setAmount] = useState("");
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [startDelayHours, setStartDelayHours] = useState("0");
  const [cliffDays, setCliffDays] = useState("30");
  const [durationDays, setDurationDays] = useState("365");
  const [revocable, setRevocable] = useState(true);

  const canSubmit =
    isValidAddress(beneficiary) &&
    parseFloat(amount) > 0 &&
    parseFloat(durationDays) > 0 &&
    parseFloat(cliffDays) >= 0 &&
    parseFloat(cliffDays) <= parseFloat(durationDays);

  const disabledReason = !beneficiary
    ? "Enter a beneficiary address"
    : !isValidAddress(beneficiary)
      ? "Invalid beneficiary address"
      : parseFloat(amount) <= 0
        ? "Enter an amount"
        : parseFloat(durationDays) <= 0
          ? "Duration must be greater than 0"
          : parseFloat(cliffDays) > parseFloat(durationDays)
            ? "Cliff cannot exceed duration"
            : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      const startTime =
        BigInt(Math.floor(Date.now() / 1000)) +
        BigInt(Math.floor(parseFloat(startDelayHours) * 3600));
      const cliffDuration = BigInt(
        Math.floor(parseFloat(cliffDays) * 86400)
      );
      const totalDuration = BigInt(
        Math.floor(parseFloat(durationDays) * 86400)
      );

      await createSchedule(
        beneficiary,
        tokenId,
        amountRaw,
        startTime,
        cliffDuration,
        totalDuration,
        revocable
      );
      toast.success("Vesting schedule created successfully");
      router.push(`/apps/vesting/${loomId}`);
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to create schedule"
      );
    }
  };

  return (
    <PageContainer
      title="Create Vesting Schedule"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Token Vesting", href: "/apps/vesting" },
        { label: truncateHash(loomId, 8), href: `/apps/vesting/${loomId}` },
        { label: "Create Schedule" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Hourglass className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">
                  New Vesting Schedule
                </CardTitle>
                <CardDescription>
                  Lock tokens with a time-based release schedule. Tokens
                  are transferred to the contract on creation.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Beneficiary Address
              </Label>
              <Input
                value={beneficiary}
                onChange={(e) => setBeneficiary(e.target.value)}
                placeholder="0x..."
                className={cn(
                  "font-mono text-sm",
                  beneficiary && !isValidAddress(beneficiary) && "border-destructive"
                )}
              />
              <FieldError
                message="Invalid address format"
                show={!!beneficiary && !isValidAddress(beneficiary)}
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
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
                  Start Delay (hours)
                </Label>
                <Input
                  type="number"
                  value={startDelayHours}
                  onChange={(e) => setStartDelayHours(e.target.value)}
                  placeholder="0"
                  min="0"
                  className="font-mono text-sm tabular-nums"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">
                  Cliff (days)
                </Label>
                <Input
                  type="number"
                  value={cliffDays}
                  onChange={(e) => setCliffDays(e.target.value)}
                  placeholder="30"
                  min="0"
                  className="font-mono text-sm tabular-nums"
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">
                  Total Duration (days)
                </Label>
                <Input
                  type="number"
                  value={durationDays}
                  onChange={(e) => setDurationDays(e.target.value)}
                  placeholder="365"
                  min="1"
                  className="font-mono text-sm tabular-nums"
                />
              </div>
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

            <div className="flex items-center gap-3">
              <button
                type="button"
                role="switch"
                aria-checked={revocable}
                onClick={() => setRevocable(!revocable)}
                className={cn(
                  "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors",
                  revocable ? "bg-norn" : "bg-muted"
                )}
              >
                <span
                  className={cn(
                    "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg transition-transform",
                    revocable ? "translate-x-4" : "translate-x-0"
                  )}
                />
              </button>
              <div>
                <Label className="text-sm">Revocable</Label>
                <p className="text-[10px] text-muted-foreground">
                  {revocable
                    ? "Creator can revoke and reclaim unvested tokens."
                    : "Tokens are permanently locked until vested."}
                </p>
              </div>
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
                <Hourglass className="mr-2 h-3.5 w-3.5" />
              )}
              Create Schedule
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
