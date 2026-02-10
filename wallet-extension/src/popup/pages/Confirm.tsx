import { useState } from "react";
import { toast } from "sonner";
import { buildTransfer, parseAmount } from "@norn-protocol/sdk";
import { useNavigationStore } from "@/stores/navigation-store";
import { useWalletStore } from "@/stores/wallet-store";
import { rpc } from "@/lib/rpc";
import { truncateAddress } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { Button } from "../components/ui/button";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

export function Confirm() {
  const [loading, setLoading] = useState(false);

  const params = useNavigationStore((s) => s.params);
  const goBack = useNavigationStore((s) => s.goBack);
  const reset = useNavigationStore((s) => s.reset);
  const activeWallet = useWalletStore((s) => s.activeWallet);

  const to = params.to as string;
  const amount = params.amount as string;
  const memo = params.memo as string | undefined;
  const tokenId = params.tokenId as string | undefined;
  const tokenSymbol = params.tokenSymbol as string | undefined;
  const tokenDecimals = params.tokenDecimals as number | undefined;

  const symbol = tokenSymbol ?? "NORN";
  const decimals = tokenDecimals ?? 12;

  const handleConfirm = async () => {
    if (!activeWallet) {
      toast.error("Wallet is locked");
      return;
    }

    setLoading(true);
    try {
      const rawAmount = parseAmount(amount, decimals);
      const knotHex = buildTransfer(activeWallet, {
        to,
        amount: rawAmount,
        tokenId: tokenId || undefined,
        memo,
      });

      await rpc.submitKnot(knotHex);
      toast.success("Transaction submitted");
      reset("dashboard");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Transaction failed",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Confirm Transaction</h2>
          <p className="text-sm text-muted-foreground">
            Review the details before confirming.
          </p>
        </div>

        <Card>
          <CardContent className="space-y-3 p-4">
            <div className="flex items-center justify-between">
              <span className="text-xs uppercase tracking-wider text-muted-foreground">From</span>
              <span className="font-mono text-sm">
                {activeWallet
                  ? truncateAddress(activeWallet.addressHex)
                  : "—"}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-xs uppercase tracking-wider text-muted-foreground">To</span>
              <span className="font-mono text-sm">{truncateAddress(to)}</span>
            </div>

            <div className="border-t pt-3">
              <div className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wider text-muted-foreground">Amount</span>
                <span className="font-mono text-lg font-medium tabular-nums">
                  {amount} {symbol}
                </span>
              </div>
            </div>

            {memo && (
              <div className="flex items-center justify-between">
                <span className="text-xs uppercase tracking-wider text-muted-foreground">Memo</span>
                <span className="max-w-[180px] truncate text-sm">{memo}</span>
              </div>
            )}
          </CardContent>
        </Card>

        <div className="flex gap-3">
          <Button
            variant="ghost"
            className="flex-1"
            onClick={goBack}
            disabled={loading}
          >
            Reject
          </Button>
          <Button
            variant="norn"
            className="flex-1"
            onClick={handleConfirm}
            disabled={loading}
          >
            {loading ? <Spinner size="sm" /> : "Confirm"}
          </Button>
        </div>
      </div>
    </div>
  );
}
