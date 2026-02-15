"use client";

import { useState, useEffect, useCallback, useRef } from "react";
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
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useSplitter } from "@/hooks/use-splitter";
import { useLoomRefresh } from "@/hooks/use-loom-refresh";
import { useWallet } from "@/hooks/use-wallet";
import { truncateAddress, isValidAddress } from "@/lib/format";
import {
  GitFork,
  ArrowLeft,
  Loader2,
  Plus,
  X,
  Coins,
} from "lucide-react";
import { toast } from "sonner";
import type { SplitterConfig } from "@/lib/borsh-splitter";

function InitializeForm({
  onSuccess,
  loomId,
}: {
  onSuccess: () => void;
  loomId: string;
}) {
  const { initialize, loading } = useSplitter(loomId);
  const [name, setName] = useState("");
  const [recipients, setRecipients] = useState<
    { address: string; percentage: string }[]
  >([
    { address: "", percentage: "" },
    { address: "", percentage: "" },
  ]);

  const validRecipients = recipients.filter(
    (r) => isValidAddress(r.address) && parseFloat(r.percentage) > 0
  );
  const totalPct = recipients.reduce(
    (sum, r) => sum + (parseFloat(r.percentage) || 0),
    0
  );
  const canSubmit =
    name.trim().length > 0 &&
    validRecipients.length >= 2 &&
    Math.abs(totalPct - 100) < 0.01;

  const disabledReason = !name.trim()
    ? "Enter a splitter name"
    : validRecipients.length < 2
      ? "Add at least 2 valid recipients"
      : Math.abs(totalPct - 100) >= 0.01
        ? "Percentages must total 100%"
        : undefined;

  const addRecipient = () =>
    setRecipients([...recipients, { address: "", percentage: "" }]);
  const removeRecipient = (i: number) => {
    if (recipients.length <= 2) return;
    setRecipients(recipients.filter((_, idx) => idx !== i));
  };
  const updateRecipient = (
    i: number,
    field: "address" | "percentage",
    val: string
  ) => {
    const next = [...recipients];
    next[i] = { ...next[i], [field]: val };
    setRecipients(next);
  };

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const mapped = validRecipients.map((r) => ({
        address: r.address,
        shareBps: BigInt(Math.round(parseFloat(r.percentage) * 100)),
      }));
      await initialize(name.trim(), mapped);
      toast.success("Splitter initialized successfully");
      onSuccess();
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Initialization failed"
      );
    }
  };

  return (
    <Card className="max-w-lg">
      <CardHeader className="pb-4">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
            <GitFork className="h-4 w-4 text-norn" />
          </div>
          <div>
            <CardTitle className="text-base">Initialize Splitter</CardTitle>
            <CardDescription>
              Configure payment recipients and their percentage shares.
              Percentages must total exactly 100%. This can only be done once.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            Splitter Name
          </Label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Revenue Split"
            maxLength={64}
            className="text-sm"
          />
        </div>

        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">
            Recipients ({validRecipients.length} valid, total: {totalPct.toFixed(1)}%)
          </Label>
          <div className="space-y-2">
            {recipients.map((r, i) => (
              <div key={i} className="flex items-center gap-2">
                <Input
                  value={r.address}
                  onChange={(e) =>
                    updateRecipient(i, "address", e.target.value)
                  }
                  placeholder="0x..."
                  className="font-mono text-xs"
                />
                <Input
                  type="number"
                  value={r.percentage}
                  onChange={(e) =>
                    updateRecipient(i, "percentage", e.target.value)
                  }
                  placeholder="%"
                  min="0"
                  max="100"
                  step="any"
                  className="w-24 font-mono text-sm tabular-nums"
                />
                {recipients.length > 2 && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 shrink-0"
                    onClick={() => removeRecipient(i)}
                  >
                    <X className="h-3.5 w-3.5" />
                  </Button>
                )}
              </div>
            ))}
          </div>
          <Button variant="outline" size="sm" onClick={addRecipient}>
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            Add Recipient
          </Button>
          {totalPct > 0 && Math.abs(totalPct - 100) >= 0.01 && (
            <p className="text-[10px] text-destructive">
              Percentages must total 100% (currently {totalPct.toFixed(2)}%).
            </p>
          )}
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
          Initialize Splitter
        </FormButton>
      </CardContent>
    </Card>
  );
}

export default function SplitterDashboardPage() {
  const params = useParams();
  const loomId = params.loomId as string;
  const { activeAddress } = useWallet();
  const { getConfig, loading } = useSplitter(loomId);
  const [config, setConfig] = useState<SplitterConfig | null>(null);
  const [fetching, setFetching] = useState(false);
  const hasLoadedRef = useRef(false);

  const fetchData = useCallback(async () => {
    if (!loomId) return;
    if (!hasLoadedRef.current) setFetching(true);
    try {
      const cfg = await getConfig();
      setConfig(cfg);
    } catch {
      // ignore
    } finally {
      hasLoadedRef.current = true;
      setFetching(false);
    }
  }, [getConfig, loomId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useLoomRefresh(loomId, fetchData);

  const addr = activeAddress?.toLowerCase() ?? "";
  const isCreator = config?.creator.toLowerCase() === addr;

  // Not yet initialized
  if (!fetching && !config && loomId) {
    return (
      <PageContainer
        title="Payment Splitter"
        action={
          <Link href="/apps/splitter">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
        }
      >
        <InitializeForm loomId={loomId} onSuccess={fetchData} />
      </PageContainer>
    );
  }

  return (
    <PageContainer
      title="Payment Splitter"
      description="Route payments to multiple recipients by percentage"
      action={
        <div className="flex items-center gap-2">
          <Link href="/apps/splitter">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              Apps
            </Button>
          </Link>
          {config && (
            <Link href={`/apps/splitter/${loomId}/split`}>
              <Button size="sm">
                <Coins className="mr-1.5 h-3.5 w-3.5" />
                Split Payment
              </Button>
            </Link>
          )}
        </div>
      }
    >
      {fetching || loading ? (
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      ) : config ? (
        <div className="max-w-2xl space-y-4">
          {/* Config overview */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <GitFork className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-sm">{config.name}</CardTitle>
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Creator</span>
                  <span className="font-mono text-xs">
                    {truncateAddress(config.creator)}
                    {isCreator && (
                      <Badge
                        variant="outline"
                        className="ml-2 text-[9px] py-0"
                      >
                        You
                      </Badge>
                    )}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Recipients */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">
                Recipients ({config.recipients.length})
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-3">
                {config.recipients.map((r, i) => {
                  const pct = Number(r.shareBps) / 100;
                  const isMe = r.address.toLowerCase() === addr;
                  return (
                    <div key={i} className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs">
                          {truncateAddress(r.address)}
                        </span>
                        {isMe && (
                          <Badge
                            variant="outline"
                            className="text-[9px] py-0"
                          >
                            You
                          </Badge>
                        )}
                      </div>
                      <div className="flex items-center gap-3">
                        <div className="w-24 h-1.5 rounded-full bg-muted overflow-hidden">
                          <div
                            className="h-full rounded-full bg-norn transition-all"
                            style={{ width: `${Math.min(pct, 100)}%` }}
                          />
                        </div>
                        <span className="font-mono text-xs tabular-nums w-14 text-right">
                          {pct.toFixed(2)}%
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </div>
      ) : null}
    </PageContainer>
  );
}
