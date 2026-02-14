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
import { AIRDROP_LOOM_ID } from "@/lib/apps-config";
import { useAirdrop } from "@/hooks/use-airdrop";
import { isValidAddress } from "@/lib/format";
import { ArrowLeft, Gift, Loader2, Plus, X } from "lucide-react";
import { toast } from "sonner";

interface RecipientRow {
  address: string;
  amount: string;
}

export default function AddRecipientsPage() {
  const router = useRouter();
  const { addRecipients, loading } = useAirdrop(AIRDROP_LOOM_ID);

  const [recipients, setRecipients] = useState<RecipientRow[]>([
    { address: "", amount: "" },
  ]);

  const validRecipients = recipients.filter(
    (r) => isValidAddress(r.address) && parseFloat(r.amount) > 0
  );
  const canSubmit = validRecipients.length > 0;

  const addRow = () =>
    setRecipients([...recipients, { address: "", amount: "" }]);

  const removeRow = (i: number) => {
    if (recipients.length <= 1) return;
    setRecipients(recipients.filter((_, idx) => idx !== i));
  };

  const updateRow = (
    i: number,
    field: keyof RecipientRow,
    value: string
  ) => {
    const next = [...recipients];
    next[i] = { ...next[i], [field]: value };
    setRecipients(next);
  };

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      const parsed = validRecipients.map((r) => ({
        address: r.address,
        amount: BigInt(Math.floor(parseFloat(r.amount) * 1e12)),
      }));
      await addRecipients(parsed);
      toast.success(
        `${parsed.length} recipient${parsed.length > 1 ? "s" : ""} added`
      );
      router.push("/apps/airdrop");
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to add recipients"
      );
    }
  };

  return (
    <PageContainer
      title="Add Recipients"
      action={
        <Link href="/apps/airdrop">
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
                <Gift className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">Add Recipients</CardTitle>
                <CardDescription>
                  Add addresses and their allocation amounts to the airdrop. You
                  can add recipients in multiple batches before finalizing.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Recipients ({validRecipients.length} valid)
              </Label>
              <div className="space-y-2">
                {recipients.map((row, i) => (
                  <div key={i} className="flex items-center gap-2">
                    <Input
                      value={row.address}
                      onChange={(e) => updateRow(i, "address", e.target.value)}
                      placeholder="0x..."
                      className="font-mono text-xs"
                    />
                    <Input
                      type="number"
                      value={row.amount}
                      onChange={(e) => updateRow(i, "amount", e.target.value)}
                      placeholder="Amount"
                      min="0"
                      step="any"
                      className="w-32 shrink-0 font-mono text-sm tabular-nums"
                    />
                    {recipients.length > 1 && (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 shrink-0"
                        onClick={() => removeRow(i)}
                      >
                        <X className="h-3.5 w-3.5" />
                      </Button>
                    )}
                  </div>
                ))}
              </div>
              <Button variant="outline" size="sm" onClick={addRow}>
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                Add Row
              </Button>
            </div>

            <Button
              onClick={handleSubmit}
              disabled={!canSubmit || loading}
              className="w-full"
            >
              {loading ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <Gift className="mr-2 h-3.5 w-3.5" />
              )}
              Add {validRecipients.length} Recipient
              {validRecipients.length !== 1 ? "s" : ""}
            </Button>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
