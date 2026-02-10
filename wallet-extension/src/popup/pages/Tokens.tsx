import { useEffect, useState } from "react";
import { Coins, Plus } from "lucide-react";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { formatNorn } from "@/lib/format";
import type { TokenInfo } from "@/types";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { TokenRow } from "../components/wallet/TokenRow";
import { Spinner } from "../components/ui/spinner";

interface TokenWithBalance extends TokenInfo {
  balance: string;
}

export function Tokens() {
  const [nornBalance, setNornBalance] = useState<string>("0");
  const [tokens, setTokens] = useState<TokenWithBalance[]>([]);
  const [loading, setLoading] = useState(true);

  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const navigate = useNavigationStore((s) => s.navigate);
  const address = getActiveAddress() ?? "";

  useEffect(() => {
    async function load() {
      if (!address) return;
      try {
        const [nornBal, tokenList] = await Promise.all([
          rpc.getBalance(address),
          rpc.listTokens(),
        ]);
        setNornBalance(nornBal);

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
        setTokens(withBalances);
      } catch {
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [address]);

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col overflow-y-auto scrollbar-thin">
        <div className="flex items-center justify-between p-4 pb-2">
          <h2 className="text-lg font-semibold">Tokens</h2>
          <button
            onClick={() => navigate("create-token")}
            className="flex h-7 w-7 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95"
          >
            <Plus className="h-4 w-4" />
          </button>
        </div>

        {loading ? (
          <div className="flex flex-1 items-center justify-center">
            <Spinner size="lg" />
          </div>
        ) : (
          <div className="divide-y divide-border px-4 pb-2">
            <div
              className="flex items-center gap-3 py-2.5 -mx-2 px-2 rounded-md transition-colors duration-150 hover:bg-muted/50 animate-slide-in"
            >
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-norn/20 text-xs font-bold text-norn">
                N
              </div>
              <div className="flex flex-1 flex-col">
                <span className="text-sm font-medium">NORN</span>
                <span className="text-xs text-muted-foreground">
                  Native Token
                </span>
              </div>
              <span className="font-mono text-sm font-medium tabular-nums">
                {formatNorn(nornBalance)}
              </span>
            </div>

            {tokens.map((token, i) => (
              <div
                key={token.token_id}
                className="animate-slide-in"
                style={{ animationDelay: `${(i + 1) * 50}ms`, animationFillMode: "backwards" }}
              >
                <TokenRow
                  symbol={token.symbol}
                  name={token.name}
                  balance={token.balance}
                  decimals={token.decimals}
                  onClick={() => navigate("token-detail", { tokenId: token.token_id })}
                />
              </div>
            ))}

            {tokens.length === 0 && (
              <div className="flex flex-col items-center gap-2 py-8 text-muted-foreground animate-fade-in">
                <Coins className="h-6 w-6" />
                <p className="text-sm">No custom tokens</p>
              </div>
            )}
          </div>
        )}
      </div>

      <BottomNav />
    </div>
  );
}
