import { useState } from "react";
import { ArrowRightLeft } from "lucide-react";
import { toast } from "sonner";
import { buildNameTransfer } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { truncateAddress, isValidAddress, strip0x } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

export function TransferName() {
  const [recipient, setRecipient] = useState("");
  const [loading, setLoading] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const activeWallet = useWalletStore((s) => s.activeWallet);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const params = useNavigationStore((s) => s.params);
  const navigate = useNavigationStore((s) => s.navigate);

  const address = getActiveAddress() ?? "";
  const name = (params.name as string) ?? "";

  const recipientValid = recipient.length === 0 || isValidAddress(recipient);
  const isValid =
    name.length > 0 &&
    recipient.length > 0 &&
    isValidAddress(recipient) &&
    recipient.toLowerCase() !== address.toLowerCase() &&
    !loading;

  const handleTransfer = async () => {
    if (!activeWallet || !isValid) return;

    setLoading(true);
    try {
      const transferHex = buildNameTransfer(activeWallet, {
        name,
        to: strip0x(recipient),
      });
      const result = await rpc.transferName(name, activeWallet.addressHex, transferHex);
      if (!result.success) {
        toast.error(result.reason ?? "Transfer failed");
        return;
      }
      toast.success(`Name "${name}" transferred`);
      navigate("register-name");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Transfer failed",
      );
    } finally {
      setLoading(false);
    }
  };

  // Confirmation step
  if (showConfirm) {
    return (
      <div className="flex h-full flex-col">
        <Header />

        <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold">Confirm Transfer</h2>
            <p className="text-sm text-muted-foreground">
              Review the details before transferring.
            </p>
          </div>

          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wider text-muted-foreground">Name</span>
                <span className="text-sm font-medium">{name}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wider text-muted-foreground">From</span>
                <span className="font-mono text-sm">{truncateAddress(address)}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wider text-muted-foreground">To</span>
                <span className="font-mono text-sm">{truncateAddress(recipient)}</span>
              </div>
              <div className="border-t pt-3">
                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">Fee</span>
                  <span className="font-mono text-lg font-medium tabular-nums">Free</span>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="flex gap-3">
            <Button
              variant="ghost"
              className="flex-1"
              onClick={() => setShowConfirm(false)}
              disabled={loading}
            >
              Back
            </Button>
            <Button
              variant="norn"
              className="flex-1"
              onClick={handleTransfer}
              disabled={loading}
            >
              {loading ? <Spinner size="sm" /> : "Confirm"}
            </Button>
          </div>
        </div>

        <BottomNav />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Transfer Name</h2>
          <p className="text-sm text-muted-foreground">
            Transfer ownership of a name to another address.
          </p>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Name</span>
            <span className="text-sm font-medium">{name}</span>
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Recipient Address</label>
            <Input
              value={recipient}
              onChange={(e) => setRecipient(e.target.value.trim())}
              placeholder="0x..."
              maxLength={42}
              className="font-mono"
            />
            {!recipientValid && (
              <p className="animate-fade-in text-xs text-destructive">
                Enter a valid address (0x followed by 40 hex characters).
              </p>
            )}
            {recipient.length > 0 &&
              isValidAddress(recipient) &&
              recipient.toLowerCase() === address.toLowerCase() && (
                <p className="animate-fade-in text-xs text-destructive">
                  Cannot transfer to yourself.
                </p>
              )}
          </div>

          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Fee</span>
            <span className="font-mono text-sm font-medium tabular-nums">Free</span>
          </div>
        </div>

        <Button
          className="w-full"
          disabled={!isValid}
          onClick={() => setShowConfirm(true)}
        >
          <ArrowRightLeft className="h-4 w-4" />
          Review Transfer
        </Button>

        {!name && (
          <div className="flex flex-col items-center gap-2 py-6 text-muted-foreground animate-fade-in">
            <ArrowRightLeft className="h-5 w-5" />
            <p className="text-sm">No name selected</p>
          </div>
        )}
      </div>

      <BottomNav />
    </div>
  );
}
