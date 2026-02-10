import { useState, useEffect } from "react";
import { toast } from "sonner";
import { buildTokenDefinition, parseAmount } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { formatNorn } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";

/** Token creation fee: 10 NORN (12 decimals). */
const TOKEN_FEE = "10000000000000";

export function CreateToken() {
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [decimals, setDecimals] = useState("8");
  const [maxSupply, setMaxSupply] = useState("0");
  const [balance, setBalance] = useState("0");
  const [balanceLoaded, setBalanceLoaded] = useState(false);
  const [loading, setLoading] = useState(false);

  const activeWallet = useWalletStore((s) => s.activeWallet);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const reset = useNavigationStore((s) => s.reset);
  const address = getActiveAddress() ?? "";

  useEffect(() => {
    if (!address) return;
    rpc
      .getBalance(address)
      .then((bal) => {
        setBalance(bal);
        setBalanceLoaded(true);
      })
      .catch(() => {
        setBalanceLoaded(true);
      });
  }, [address]);

  const nameValid = name.length === 0 || (name.length >= 1 && name.length <= 64 && /^[\x20-\x7E]+$/.test(name));
  const symbolValid =
    symbol.length === 0 || (symbol.length >= 1 && symbol.length <= 12 && /^[A-Z0-9]+$/.test(symbol));
  const decimalsNum = parseInt(decimals, 10);
  const decimalsValid = !isNaN(decimalsNum) && decimalsNum >= 0 && decimalsNum <= 18;
  const maxSupplyValid = maxSupply.length === 0 || (!isNaN(Number(maxSupply)) && Number(maxSupply) >= 0);
  const hasSufficientBalance = BigInt(balance) >= BigInt(TOKEN_FEE);

  const isValid =
    name.length >= 1 &&
    nameValid &&
    symbol.length >= 1 &&
    symbolValid &&
    decimalsValid &&
    maxSupplyValid &&
    hasSufficientBalance &&
    balanceLoaded &&
    !loading;

  const handleCreate = async () => {
    if (!activeWallet || !isValid) return;

    setLoading(true);
    try {
      const maxSupplyRaw =
        maxSupply === "" || maxSupply === "0"
          ? 0n
          : parseAmount(maxSupply, decimalsNum);

      const knotHex = buildTokenDefinition(activeWallet, {
        name,
        symbol,
        decimals: decimalsNum,
        maxSupply: maxSupplyRaw,
      });

      const result = await rpc.createToken(knotHex);
      if (!result.success) {
        toast.error(result.reason ?? "Token creation failed");
        return;
      }
      toast.success(`Token "${symbol}" created`);
      reset("tokens");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Token creation failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Create Token</h2>
          <p className="text-sm text-muted-foreground">
            Deploy a new custom token on the network. Costs 10 NORN.
          </p>
        </div>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Token Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Token"
              maxLength={64}
            />
            {name.length > 0 && !nameValid && (
              <p className="animate-fade-in text-xs text-destructive">
                1-64 printable ASCII characters
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Symbol</label>
            <Input
              value={symbol}
              onChange={(e) => setSymbol(e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, ""))}
              placeholder="MTK"
              maxLength={12}
            />
            {symbol.length > 0 && !symbolValid && (
              <p className="animate-fade-in text-xs text-destructive">
                1-12 uppercase alphanumeric characters
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Decimals</label>
            <Input
              type="number"
              value={decimals}
              onChange={(e) => setDecimals(e.target.value)}
              placeholder="8"
              min="0"
              max="18"
            />
            {decimals.length > 0 && !decimalsValid && (
              <p className="animate-fade-in text-xs text-destructive">
                Must be between 0 and 18
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">
              Max Supply <span className="text-muted-foreground">(0 = unlimited)</span>
            </label>
            <Input
              type="number"
              value={maxSupply}
              onChange={(e) => setMaxSupply(e.target.value)}
              placeholder="0"
              min="0"
              step="any"
              className="font-mono tabular-nums"
            />
          </div>

          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Fee</span>
            <span className="font-mono text-sm font-medium tabular-nums">10 NORN</span>
          </div>

          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Balance</span>
            <span className="font-mono text-sm tabular-nums">
              {balanceLoaded ? `${formatNorn(balance)} NORN` : "Loading..."}
            </span>
          </div>

          {balanceLoaded && !hasSufficientBalance && (
            <p className="animate-fade-in text-xs text-destructive">
              Insufficient balance. You need at least 10 NORN.
            </p>
          )}
        </div>

        <Button className="w-full" disabled={!isValid} onClick={handleCreate}>
          {loading ? <Spinner size="sm" /> : "Create Token"}
        </Button>
      </div>

      <BottomNav />
    </div>
  );
}
