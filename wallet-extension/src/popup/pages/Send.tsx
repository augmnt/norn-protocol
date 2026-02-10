import { useState, useEffect, useCallback } from "react";
import { AtSign } from "lucide-react";
import { useNavigationStore } from "@/stores/navigation-store";
import { useWalletStore } from "@/stores/wallet-store";
import { rpc } from "@/lib/rpc";
import { isValidAddress, formatNorn, formatAmount, truncateAddress } from "@/lib/format";
import type { TokenInfo } from "@/types";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";

interface SelectedToken {
  id: string;
  symbol: string;
  decimals: number;
}

export function Send() {
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [balance, setBalance] = useState<string>("0");

  // Token selection
  const [selectedToken, setSelectedToken] = useState<SelectedToken | null>(null);
  const [userTokens, setUserTokens] = useState<(TokenInfo & { balance: string })[]>([]);
  const [tokensLoading, setTokensLoading] = useState(true);

  // Name resolution state
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolving, setResolving] = useState(false);
  const [resolveError, setResolveError] = useState<string | null>(null);

  const params = useNavigationStore((s) => s.params);
  const navigate = useNavigationStore((s) => s.navigate);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const address = getActiveAddress();

  // Pre-select token if navigated from TokenDetail
  useEffect(() => {
    if (params.tokenId) {
      setSelectedToken({
        id: params.tokenId as string,
        symbol: params.tokenSymbol as string,
        decimals: params.tokenDecimals as number,
      });
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Fetch user's tokens with non-zero balances
  useEffect(() => {
    async function loadTokens() {
      if (!address) return;
      try {
        const tokenList = await rpc.listTokens();
        const withBalances = await Promise.all(
          tokenList.map(async (token) => {
            try {
              const b = await rpc.getBalance(address, token.token_id);
              return { ...token, balance: b };
            } catch {
              return { ...token, balance: "0" };
            }
          }),
        );
        setUserTokens(withBalances.filter((t) => BigInt(t.balance) > 0n));
      } catch {
        // leave empty
      } finally {
        setTokensLoading(false);
      }
    }
    loadTokens();
  }, [address]);

  const selectedDecimals = selectedToken?.decimals ?? 12;
  const selectedSymbol = selectedToken?.symbol ?? "NORN";

  const fetchBalance = useCallback(async () => {
    if (!address) return;
    try {
      const bal = await rpc.getBalance(address, selectedToken?.id);
      setBalance(bal);
    } catch {}
  }, [address, selectedToken?.id]);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  // Determine if input looks like a name (not an address)
  const isNameInput = to.length > 0 && !to.startsWith("0x") && /^[a-z0-9-]+$/.test(to);
  const isAddressInput = to.length > 0 && to.startsWith("0x");

  // Resolve name when input changes
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

  // The effective destination address (either typed directly or resolved from name)
  const effectiveAddress = isNameInput ? resolvedAddress : to;

  const isValid =
    effectiveAddress != null &&
    isValidAddress(effectiveAddress) &&
    amount.length > 0 &&
    parseFloat(amount) > 0;

  const handleMax = () => {
    const raw = BigInt(balance);
    const divisor = BigInt(10 ** selectedDecimals);
    const whole = raw / divisor;
    const frac = (raw % divisor).toString().padStart(selectedDecimals, "0").replace(/0+$/, "");
    setAmount(frac ? `${whole}.${frac}` : whole.toString());
  };

  const handlePreview = () => {
    if (!isValid || !effectiveAddress) return;
    navigate("confirm", {
      to: effectiveAddress,
      amount,
      memo: memo || undefined,
      tokenId: selectedToken?.id,
      tokenSymbol: selectedToken?.symbol,
      tokenDecimals: selectedToken?.decimals,
    });
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Send {selectedSymbol}</h2>
          <p className="text-sm text-muted-foreground">
            Available:{" "}
            <span className="font-mono tabular-nums">
              {selectedToken ? formatAmount(balance, selectedDecimals) : formatNorn(balance)}
            </span>{" "}
            {selectedSymbol}
          </p>
        </div>

        {/* Token selector */}
        {(!tokensLoading && userTokens.length > 0) && (
          <div className="flex flex-wrap gap-1.5">
            <button
              type="button"
              onClick={() => { setSelectedToken(null); setAmount(""); }}
              className={`rounded-full px-3 py-1 text-xs font-medium transition-all duration-150 ${
                selectedToken === null
                  ? "bg-norn text-white"
                  : "bg-secondary text-secondary-foreground hover:bg-muted"
              }`}
            >
              NORN
            </button>
            {userTokens.map((t) => (
              <button
                key={t.token_id}
                type="button"
                onClick={() => {
                  setSelectedToken({ id: t.token_id, symbol: t.symbol, decimals: t.decimals });
                  setAmount("");
                }}
                className={`rounded-full px-3 py-1 text-xs font-medium transition-all duration-150 ${
                  selectedToken?.id === t.token_id
                    ? "bg-norn text-white"
                    : "bg-secondary text-secondary-foreground hover:bg-muted"
                }`}
              >
                {t.symbol}
              </button>
            ))}
          </div>
        )}

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

          <div className="space-y-1.5">
            <label className="text-sm font-medium">
              Memo <span className="text-muted-foreground">(optional)</span>
            </label>
            <Input
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              placeholder="Add a note"
              maxLength={256}
            />
          </div>
        </div>

        <Button
          className="w-full"
          disabled={!isValid}
          onClick={handlePreview}
        >
          Preview Transaction
        </Button>
      </div>

      <BottomNav />
    </div>
  );
}
