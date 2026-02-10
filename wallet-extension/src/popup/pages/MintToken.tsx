import { useState, useEffect } from "react";
import { AtSign } from "lucide-react";
import { toast } from "sonner";
import { buildTokenMint, parseAmount } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { isValidAddress, truncateAddress } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";

export function MintToken() {
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);

  // Name resolution state
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(false);
  const [resolveError, setResolveError] = useState<string | null>(null);

  const params = useNavigationStore((s) => s.params);
  const goBack = useNavigationStore((s) => s.goBack);
  const activeWallet = useWalletStore((s) => s.activeWallet);

  const tokenId = params.tokenId as string;
  const tokenSymbol = params.tokenSymbol as string;
  const tokenDecimals = params.tokenDecimals as number;

  const isNameInput = to.length > 0 && !to.startsWith("0x") && /^[a-z0-9-]+$/.test(to);
  const isAddressInput = to.length > 0 && to.startsWith("0x");

  useEffect(() => {
    if (!isNameInput || to.length < 3) {
      setResolvedAddress(null);
      setResolveError(null);
      return;
    }

    let cancelled = false;
    const timer = setTimeout(async () => {
      setResolving(true);
      setResolveError(null);
      try {
        const result = await rpc.resolveName(to);
        if (cancelled) return;
        if (result?.owner) {
          setResolvedAddress(result.owner.startsWith("0x") ? result.owner : `0x${result.owner}`);
          setResolveError(null);
        } else {
          setResolvedAddress(null);
          setResolveError("Name not found");
        }
      } catch {
        if (!cancelled) {
          setResolvedAddress(null);
          setResolveError("Failed to resolve name");
        }
      } finally {
        if (!cancelled) setResolving(false);
      }
    }, 300);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [to, isNameInput]);

  const effectiveAddress = isNameInput ? resolvedAddress : to;

  const isValid =
    effectiveAddress != null &&
    isValidAddress(effectiveAddress) &&
    amount.length > 0 &&
    parseFloat(amount) > 0 &&
    !loading;

  const handleMint = async () => {
    if (!activeWallet || !isValid || !effectiveAddress) return;

    setLoading(true);
    try {
      const rawAmount = parseAmount(amount, tokenDecimals);
      const knotHex = buildTokenMint(activeWallet, {
        tokenId,
        to: effectiveAddress,
        amount: rawAmount,
      });

      const result = await rpc.mintToken(knotHex);
      if (!result.success) {
        toast.error(result.reason ?? "Mint failed");
        return;
      }
      toast.success(`Minted ${amount} ${tokenSymbol}`);
      goBack();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Mint failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Mint {tokenSymbol}</h2>
          <p className="text-sm text-muted-foreground">
            Mint new tokens to a recipient address.
          </p>
        </div>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Recipient</label>
            <Input
              value={to}
              onChange={(e) => setTo(e.target.value.trim())}
              placeholder="0x... or registered name"
              className={isNameInput ? "text-sm" : "font-mono text-xs"}
            />
            {isNameInput && resolving && (
              <div className="flex items-center gap-1.5 animate-fade-in">
                <Spinner size="sm" />
                <span className="text-xs text-muted-foreground">Resolving name...</span>
              </div>
            )}
            {isNameInput && resolvedAddress && !resolving && (
              <div className="flex items-center gap-1.5 animate-fade-in">
                <AtSign className="h-3 w-3 text-emerald-400" />
                <span className="font-mono text-xs text-emerald-400">
                  {truncateAddress(resolvedAddress)}
                </span>
              </div>
            )}
            {isNameInput && resolveError && !resolving && (
              <p className="animate-fade-in text-xs text-destructive">{resolveError}</p>
            )}
            {isAddressInput && !isValidAddress(to) && (
              <p className="animate-fade-in text-xs text-destructive">Invalid address format</p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Amount</label>
            <Input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.0"
              min="0"
              step="any"
              className="font-mono tabular-nums"
            />
          </div>
        </div>

        <Button className="w-full" disabled={!isValid} onClick={handleMint}>
          {loading ? <Spinner size="sm" /> : `Mint ${tokenSymbol}`}
        </Button>
      </div>

      <BottomNav />
    </div>
  );
}
