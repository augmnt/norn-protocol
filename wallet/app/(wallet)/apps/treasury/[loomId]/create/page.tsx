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
import { FieldError } from "@/components/ui/field-error";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useTreasury } from "@/hooks/use-treasury";
import { isValidAddress, truncateHash } from "@/lib/format";
import { cn } from "@/lib/utils";
import { Vault, Loader2 } from "lucide-react";
import { toast } from "sonner";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function CreateProposalPage() {
  const router = useRouter();
  const params = useParams();
  const loomId = params.loomId as string;
  const { propose, loading } = useTreasury(loomId);

  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [description, setDescription] = useState("");
  const [deadlineHours, setDeadlineHours] = useState("168");

  const canSubmit =
    isValidAddress(to) &&
    parseFloat(amount) > 0 &&
    description.trim().length > 0 &&
    parseFloat(deadlineHours) > 0;

  const disabledReason = !to
    ? "Enter a recipient address"
    : !isValidAddress(to)
      ? "Invalid recipient address"
      : parseFloat(amount) <= 0
        ? "Enter an amount"
        : !description.trim()
          ? "Enter a description"
          : parseFloat(deadlineHours) <= 0
            ? "Deadline must be greater than 0"
            : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      const deadlineSecs =
        BigInt(Math.floor(Date.now() / 1000)) +
        BigInt(Math.floor(parseFloat(deadlineHours) * 3600));

      await propose(to, tokenId, amountRaw, description.trim(), deadlineSecs);
      toast.success("Proposal created successfully");
      router.push(`/apps/treasury/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to create proposal");
    }
  };

  return (
    <PageContainer
      title="Create Proposal"
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: "Multisig Treasury", href: "/apps/treasury" },
        { label: truncateHash(loomId, 8), href: `/apps/treasury/${loomId}` },
        { label: "Create Proposal" },
      ]}
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Vault className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">New Treasury Proposal</CardTitle>
                <CardDescription>
                  Propose a transfer from the shared treasury. Requires
                  multi-signature approval.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Recipient Address
              </Label>
              <Input
                value={to}
                onChange={(e) => setTo(e.target.value)}
                placeholder="0x..."
                className={cn(
                  "font-mono text-sm",
                  to && !isValidAddress(to) && "border-destructive"
                )}
              />
              <FieldError
                message="Invalid address format"
                show={!!to && !isValidAddress(to)}
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
                  Deadline (hours)
                </Label>
                <Input
                  type="number"
                  value={deadlineHours}
                  onChange={(e) => setDeadlineHours(e.target.value)}
                  placeholder="168"
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

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Description
              </Label>
              <Textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="What is this transfer for?"
                className="text-sm min-h-[80px] resize-y"
                maxLength={256}
                rows={3}
              />
              <p className="text-[10px] text-muted-foreground text-right">
                {description.length}/256
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
                <Vault className="mr-2 h-3.5 w-3.5" />
              )}
              Create Proposal
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
