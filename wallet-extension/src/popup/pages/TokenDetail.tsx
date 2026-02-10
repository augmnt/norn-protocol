import { useEffect, useState } from "react";
import { ArrowUpRight, Flame, Coins } from "lucide-react";
import { useNavigationStore } from "@/stores/navigation-store";
import { useWalletStore } from "@/stores/wallet-store";
import { rpc } from "@/lib/rpc";
import { formatAmount, truncateAddress, formatTimestamp } from "@/lib/format";
import type { TokenInfo } from "@/types";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

export function TokenDetail() {
  const [token, setToken] = useState<TokenInfo | null>(null);
  const [balance, setBalance] = useState<string>("0");
  const [loading, setLoading] = useState(true);

  const params = useNavigationStore((s) => s.params);
  const navigate = useNavigationStore((s) => s.navigate);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);

  const tokenId = params.tokenId as string;
  const address = getActiveAddress() ?? "";

  useEffect(() => {
    async function load() {
      if (!tokenId || !address) return;
      try {
        const [info, bal] = await Promise.all([
          rpc.getTokenInfo(tokenId),
          rpc.getBalance(address, tokenId),
        ]);
        setToken(info);
        setBalance(bal);
      } catch {
        // leave null
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [tokenId, address]);

  const isCreator =
    token != null &&
    address.toLowerCase().replace(/^0x/, "") ===
      token.creator.toLowerCase().replace(/^0x/, "");

  const hasBalance = BigInt(balance) > 0n;

  const navParams = {
    tokenId,
    tokenSymbol: token?.symbol ?? "",
    tokenDecimals: token?.decimals ?? 8,
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        {loading ? (
          <div className="flex flex-1 items-center justify-center">
            <Spinner size="lg" />
          </div>
        ) : !token ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground animate-fade-in">
            <Coins className="h-6 w-6" />
            <p className="text-sm">Token not found</p>
          </div>
        ) : (
          <>
            <div className="flex flex-col items-center gap-1 animate-fade-in">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-norn/20 text-sm font-bold text-norn">
                {token.symbol.slice(0, 2)}
              </div>
              <h2 className="text-lg font-semibold">{token.symbol}</h2>
              <p className="text-sm text-muted-foreground">{token.name}</p>
            </div>

            <div className="flex flex-col items-center animate-fade-in">
              <span className="font-mono text-2xl font-semibold tabular-nums">
                {formatAmount(balance, token.decimals)}
              </span>
              <span className="text-sm text-muted-foreground">
                {token.symbol} Balance
              </span>
            </div>

            <div className="flex items-center justify-center gap-4 animate-fade-in">
              <button
                onClick={() => navigate("send", navParams)}
                className="flex flex-col items-center gap-1"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95">
                  <ArrowUpRight className="h-4 w-4" />
                </div>
                <span className="text-xs text-muted-foreground">Send</span>
              </button>

              {isCreator && (
                <button
                  onClick={() => navigate("mint-token", navParams)}
                  className="flex flex-col items-center gap-1"
                >
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95">
                    <Coins className="h-4 w-4" />
                  </div>
                  <span className="text-xs text-muted-foreground">Mint</span>
                </button>
              )}

              {hasBalance && (
                <button
                  onClick={() => navigate("burn-token", navParams)}
                  className="flex flex-col items-center gap-1"
                >
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95">
                    <Flame className="h-4 w-4" />
                  </div>
                  <span className="text-xs text-muted-foreground">Burn</span>
                </button>
              )}
            </div>

            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">
                    Decimals
                  </span>
                  <span className="text-sm">{token.decimals}</span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">
                    Current Supply
                  </span>
                  <span className="font-mono text-sm tabular-nums">
                    {formatAmount(token.current_supply, token.decimals)}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">
                    Max Supply
                  </span>
                  <span className="font-mono text-sm tabular-nums">
                    {BigInt(token.max_supply) === 0n
                      ? "Unlimited"
                      : formatAmount(token.max_supply, token.decimals)}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">
                    Creator
                  </span>
                  <span className="font-mono text-sm">
                    {truncateAddress(
                      token.creator.startsWith("0x")
                        ? token.creator
                        : `0x${token.creator}`,
                    )}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-xs uppercase tracking-wider text-muted-foreground">
                    Created
                  </span>
                  <span className="text-sm">
                    {formatTimestamp(token.created_at)}
                  </span>
                </div>
              </CardContent>
            </Card>
          </>
        )}
      </div>

      <BottomNav />
    </div>
  );
}
