import { useState, useEffect } from "react";
import { toast } from "sonner";
import { buildTokenBurn, parseAmount } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { formatAmount } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";

export function BurnToken() {
  const [amount, setAmount] = useState("");
  const [balance, setBalance] = useState<string>("0");
  const [balanceLoaded, setBalanceLoaded] = useState(false);
  const [loading, setLoading] = useState(false);

  const params = useNavigationStore((s) => s.params);
  const goBack = useNavigationStore((s) => s.goBack);
  const activeWallet = useWalletStore((s) => s.activeWallet);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);

  const tokenId = params.tokenId as string;
  const tokenSymbol = params.tokenSymbol as string;
  const tokenDecimals = params.tokenDecimals as number;
  const address = getActiveAddress() ?? "";

  useEffect(() => {
    if (!address || !tokenId) return;
    rpc
      .getBalance(address, tokenId)
      .then((bal) => {
        setBalance(bal);
        setBalanceLoaded(true);
      })
      .catch(() => {
        setBalanceLoaded(true);
      });
  }, [address, tokenId]);

  const handleMax = () => {
    const raw = BigInt(balance);
    const divisor = BigInt(10 ** tokenDecimals);
    const whole = raw / divisor;
    const frac = (raw % divisor).toString().padStart(tokenDecimals, "0").replace(/0+$/, "");
    setAmount(frac ? `${whole}.${frac}` : whole.toString());
  };

  const isValid =
    amount.length > 0 &&
    parseFloat(amount) > 0 &&
    balanceLoaded &&
    !loading;

  const handleBurn = async () => {
    if (!activeWallet || !isValid) return;

    setLoading(true);
    try {
      const rawAmount = parseAmount(amount, tokenDecimals);

      if (rawAmount > BigInt(balance)) {
        toast.error("Amount exceeds balance");
        return;
      }

      const knotHex = buildTokenBurn(activeWallet, {
        tokenId,
        amount: rawAmount,
      });

      const result = await rpc.burnToken(knotHex);
      if (!result.success) {
        toast.error(result.reason ?? "Burn failed");
        return;
      }
      toast.success(`Burned ${amount} ${tokenSymbol}`);
      goBack();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Burn failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Burn {tokenSymbol}</h2>
          <p className="text-sm text-muted-foreground">
            Permanently destroy tokens from your balance.
          </p>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">
              Your Balance
            </span>
            <span className="font-mono text-sm tabular-nums">
              {balanceLoaded
                ? `${formatAmount(balance, tokenDecimals)} ${tokenSymbol}`
                : "Loading..."}
            </span>
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Amount</label>
            <div className="relative">
              <Input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.0"
                min="0"
                step="any"
                className="font-mono tabular-nums"
              />
              <button
                type="button"
                onClick={handleMax}
                className="absolute right-2 top-1/2 -translate-y-1/2 rounded bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground transition-all duration-150 hover:bg-norn/20 active:scale-95"
              >
                MAX
              </button>
            </div>
          </div>
        </div>

        <Button
          className="w-full"
          variant="norn"
          disabled={!isValid}
          onClick={handleBurn}
        >
          {loading ? <Spinner size="sm" /> : `Burn ${tokenSymbol}`}
        </Button>
      </div>

      <BottomNav />
    </div>
  );
}
