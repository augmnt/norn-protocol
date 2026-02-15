"use client";

import { useState, useEffect, useCallback } from "react";
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
import { FormButton } from "@/components/ui/form-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SPLITTER_LOOM_ID } from "@/lib/apps-config";
import { useSplitter } from "@/hooks/use-splitter";
import { truncateAddress, formatAmount } from "@/lib/format";
import { ArrowLeft, GitFork, Loader2 } from "lucide-react";
import { toast } from "sonner";
import type { SplitterConfig } from "@/lib/borsh-splitter";

const NATIVE_TOKEN_ID = "0".repeat(64);

export default function SplitPaymentPage() {
  const router = useRouter();
  const { split, getConfig, loading } = useSplitter(SPLITTER_LOOM_ID);

  const [config, setConfig] = useState<SplitterConfig | null>(null);
  const [tokenId, setTokenId] = useState(NATIVE_TOKEN_ID);
  const [amount, setAmount] = useState("");
  const [fetching, setFetching] = useState(true);

  const fetchConfig = useCallback(async () => {
    try {
      const cfg = await getConfig();
      setConfig(cfg);
    } catch {
      // ignore
    } finally {
      setFetching(false);
    }
  }, [getConfig]);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const canSubmit = parseFloat(amount) > 0;

  const disabledReason = parseFloat(amount) <= 0
    ? "Enter an amount"
    : undefined;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const amountRaw = BigInt(Math.floor(parseFloat(amount) * 1e12));
      await split(tokenId, amountRaw);
      toast.success("Payment split successfully");
      router.push("/apps/splitter");
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to split payment"
      );
    }
  };

  return (
    <PageContainer
      title="Split Payment"
      action={
        <Link href="/apps/splitter">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-lg space-y-4">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
                <GitFork className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Split Payment</CardTitle>
                <CardDescription>
                  Send tokens to be automatically distributed among the
                  configured recipients according to their shares.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
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
                <GitFork className="mr-2 h-3.5 w-3.5" />
              )}
              Split Payment
            </FormButton>
          </CardContent>
        </Card>

        {/* Configured recipients preview */}
        {!fetching && config && parseFloat(amount) > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Distribution Preview</CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-2">
                {config.recipients.map((r, i) => {
                  const pct = Number(r.shareBps) / 100;
                  const share =
                    (BigInt(Math.floor(parseFloat(amount) * 1e12)) *
                      r.shareBps) /
                    10000n;
                  return (
                    <div
                      key={i}
                      className="flex items-center justify-between text-sm"
                    >
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs">
                          {truncateAddress(r.address)}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          ({pct.toFixed(2)}%)
                        </span>
                      </div>
                      <span className="font-mono tabular-nums text-xs">
                        {formatAmount(share.toString())}
                      </span>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </PageContainer>
  );
}
