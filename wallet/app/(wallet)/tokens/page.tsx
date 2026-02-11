"use client";

import { useState, useMemo } from "react";
import { useQueries } from "@tanstack/react-query";
import { useWallet } from "@/hooks/use-wallet";
import { useTokenBalances } from "@/hooks/use-token-balances";
import { useCreatedTokens } from "@/hooks/use-created-tokens";
import { useTokenOps } from "@/hooks/use-token-ops";
import { rpcCall } from "@/lib/rpc";
import { PageContainer } from "@/components/ui/page-container";
import { DataTable } from "@/components/ui/data-table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { HashDisplay } from "@/components/ui/hash-display";
import { EmptyState } from "@/components/ui/empty-state";
import { TableSkeleton } from "@/components/ui/loading-skeleton";
import { Separator } from "@/components/ui/separator";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { NATIVE_TOKEN_ID, QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { explorerTokenUrl } from "@/lib/explorer";
import { formatAmount } from "@/lib/format";
import { Coins, Plus, Fingerprint, ArrowUpCircle, Flame, ArrowUpRight } from "lucide-react";
import { useRouter } from "next/navigation";
import { toast } from "sonner";
import type { BalanceEntry, TokenInfo } from "@/types";

interface EnrichedBalance {
  token_id: string;
  amount: string;
  symbol: string;
  name: string;
  decimals: number;
  isNative: boolean;
  isCreator: boolean;
}

export default function TokensPage() {
  const { activeAddress } = useWallet();
  const { data: threadState, isLoading: balancesLoading } = useTokenBalances(activeAddress ?? undefined);
  const { data: createdTokens, isLoading: createdLoading } = useCreatedTokens(activeAddress ?? undefined);
  const { createToken, mintToken, burnToken, loading } = useTokenOps();
  const router = useRouter();

  const isLoading = balancesLoading || createdLoading;

  // Filter to non-zero balances
  const balances = useMemo(
    () => threadState?.balances?.filter((b: BalanceEntry) => BigInt(b.amount || "0") > 0n) ?? [],
    [threadState]
  );

  // Fetch token info for each non-native balance token
  const nonNativeIds = useMemo(
    () => balances.filter((b: BalanceEntry) => b.token_id !== NATIVE_TOKEN_ID).map((b: BalanceEntry) => b.token_id),
    [balances]
  );

  const tokenInfoQueries = useQueries({
    queries: nonNativeIds.map((id: string) => ({
      queryKey: QUERY_KEYS.tokenInfo(id),
      queryFn: () => rpcCall<TokenInfo>("norn_getTokenInfo", [id]),
      staleTime: STALE_TIMES.semiStatic,
    })),
  });

  // Build enriched list: balances + created tokens (with 0 balance if not held)
  const enrichedBalances: EnrichedBalance[] = useMemo(() => {
    const infoMap = new Map<string, TokenInfo>();
    nonNativeIds.forEach((id: string, i: number) => {
      const data = tokenInfoQueries[i]?.data;
      if (data) infoMap.set(id, data);
    });

    // Start with balance entries
    const balanceTokenIds = new Set(balances.map((b: BalanceEntry) => b.token_id));
    const result: EnrichedBalance[] = balances.map((b: BalanceEntry) => {
      const isNative = b.token_id === NATIVE_TOKEN_ID;
      const info = infoMap.get(b.token_id);
      return {
        token_id: b.token_id,
        amount: b.amount,
        symbol: isNative ? "NORN" : info?.symbol ?? "—",
        name: isNative ? "Norn Protocol" : info?.name ?? "Unknown Token",
        decimals: isNative ? 12 : info?.decimals ?? 12,
        isNative,
        isCreator: !!info && info.creator?.toLowerCase() === activeAddress?.toLowerCase(),
      };
    });

    // Append created tokens that don't appear in balances (0 balance)
    if (createdTokens) {
      for (const token of createdTokens) {
        if (!balanceTokenIds.has(token.token_id)) {
          result.push({
            token_id: token.token_id,
            amount: "0",
            symbol: token.symbol,
            name: token.name,
            decimals: token.decimals,
            isNative: false,
            isCreator: true,
          });
        }
      }
    }

    return result;
  }, [balances, nonNativeIds, tokenInfoQueries, activeAddress, createdTokens]);

  // Create dialog state
  const [createOpen, setCreateOpen] = useState(false);
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [decimals, setDecimals] = useState("12");
  const [maxSupply, setMaxSupply] = useState("1000000");
  const [initialSupply, setInitialSupply] = useState("0");

  // Mint dialog state
  const [mintOpen, setMintOpen] = useState(false);
  const [mintTokenId, setMintTokenId] = useState("");
  const [mintTo, setMintTo] = useState("");
  const [mintAmount, setMintAmount] = useState("");
  const [mintDecimals, setMintDecimals] = useState(12);
  const [mintSymbol, setMintSymbol] = useState("");

  // Burn dialog state
  const [burnOpen, setBurnOpen] = useState(false);
  const [burnTokenId, setBurnTokenId] = useState("");
  const [burnAmount, setBurnAmount] = useState("");
  const [burnDecimals, setBurnDecimals] = useState(12);
  const [burnConfirmText, setBurnConfirmText] = useState("");
  const [burnSymbol, setBurnSymbol] = useState("");

  const handleCreate = async () => {
    try {
      await createToken({
        name,
        symbol: symbol.toUpperCase(),
        decimals: parseInt(decimals),
        maxSupply,
        initialSupply: initialSupply || "0",
      });
      setCreateOpen(false);
      toast.success(`Token ${symbol.toUpperCase()} created`);
      setName("");
      setSymbol("");
      setInitialSupply("0");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to create token");
    }
  };

  const handleMint = async () => {
    try {
      await mintToken({
        tokenId: mintTokenId,
        to: mintTo || activeAddress!,
        amount: mintAmount,
        decimals: mintDecimals,
      });
      setMintOpen(false);
      toast.success("Tokens minted");
      setMintAmount("");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to mint tokens");
    }
  };

  const handleBurn = async () => {
    try {
      await burnToken({
        tokenId: burnTokenId,
        amount: burnAmount,
        decimals: burnDecimals,
      });
      setBurnOpen(false);
      toast.success("Tokens burned");
      setBurnAmount("");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to burn tokens");
    }
  };

  const openMintDialog = (b: EnrichedBalance) => {
    setMintTokenId(b.token_id);
    setMintTo(activeAddress ?? "");
    setMintAmount("");
    setMintDecimals(b.decimals);
    setMintSymbol(b.symbol);
    setMintOpen(true);
  };

  const openBurnDialog = (b: EnrichedBalance) => {
    setBurnTokenId(b.token_id);
    setBurnAmount("");
    setBurnConfirmText("");
    setBurnDecimals(b.decimals);
    setBurnSymbol(b.symbol);
    setBurnOpen(true);
  };

  const columns = [
    {
      header: "Symbol",
      key: "symbol",
      render: (b: EnrichedBalance) => (
        <div className="flex items-center gap-2">
          {b.isNative ? (
            <Badge variant="outline" className="font-mono">NORN</Badge>
          ) : (
            <a href={explorerTokenUrl(b.token_id)} target="_blank" rel="noopener noreferrer" className="text-norn hover:underline">
              <Badge variant="outline" className="font-mono">{b.symbol}</Badge>
            </a>
          )}
          {b.isNative && (
            <Badge variant="secondary" className="text-[10px] px-1.5 py-0">Native</Badge>
          )}
          {b.isCreator && (
            <Badge variant="secondary" className="text-[10px] px-1.5 py-0">Creator</Badge>
          )}
        </div>
      ),
    },
    {
      header: "Name",
      key: "name",
      hideOnMobile: true,
      render: (b: EnrichedBalance) => (
        <span className="text-sm font-medium">{b.name}</span>
      ),
    },
    {
      header: "Balance",
      key: "balance",
      className: "text-right",
      render: (b: EnrichedBalance) => (
        <span className="font-mono text-sm tabular-nums">
          {formatAmount(b.amount, b.decimals)}
        </span>
      ),
    },
    {
      header: "Token ID",
      key: "token_id",
      hideOnMobile: true,
      render: (b: EnrichedBalance) =>
        b.isNative ? (
          <span className="text-xs text-muted-foreground">—</span>
        ) : (
          <HashDisplay
            hash={b.token_id}
            href={explorerTokenUrl(b.token_id)}
            chars={6}
            copy={false}
          />
        ),
    },
    {
      header: "",
      key: "actions",
      className: "text-right w-[80px]",
      render: (b: EnrichedBalance) =>
        b.isNative ? (
          <div className="flex justify-end">
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-muted-foreground hover:text-foreground"
              title="Send NORN"
              aria-label="Send NORN"
              onClick={(e) => { e.stopPropagation(); router.push("/send"); }}
            >
              <ArrowUpRight className="h-3.5 w-3.5" />
            </Button>
          </div>
        ) : (
          <div className="flex justify-end gap-0.5">
            {BigInt(b.amount || "0") > 0n && (
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-muted-foreground hover:text-foreground"
                title="Send"
                aria-label={`Send ${b.symbol}`}
                onClick={(e) => { e.stopPropagation(); router.push(`/send?token=${b.token_id}`); }}
              >
                <ArrowUpRight className="h-3.5 w-3.5" />
              </Button>
            )}
            {b.isCreator && (
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-muted-foreground hover:text-foreground"
                title="Mint"
                aria-label={`Mint ${b.symbol}`}
                onClick={(e) => { e.stopPropagation(); openMintDialog(b); }}
              >
                <ArrowUpCircle className="h-3.5 w-3.5" />
              </Button>
            )}
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-muted-foreground hover:text-foreground"
              title="Burn"
              aria-label={`Burn ${b.symbol}`}
              onClick={(e) => { e.stopPropagation(); openBurnDialog(b); }}
            >
              <Flame className="h-3.5 w-3.5" />
            </Button>
          </div>
        ),
    },
  ];

  return (
    <PageContainer
      title="Tokens"
      action={
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <Plus className="mr-1.5 h-3.5 w-3.5" />
          Create Token
        </Button>
      }
    >
      {isLoading ? (
        <TableSkeleton rows={5} cols={5} />
      ) : enrichedBalances.length === 0 ? (
        <EmptyState
          icon={Coins}
          title="No token balances"
          description="Create a new token or receive tokens to get started"
        />
      ) : (
        <DataTable
          columns={columns}
          data={enrichedBalances}
          keyExtractor={(b) => b.token_id}
          emptyMessage="No tokens found"
        />
      )}

      {/* Create Token Dialog */}
      <Dialog open={createOpen} onOpenChange={(open) => {
        setCreateOpen(open);
        if (!open) { setName(""); setSymbol(""); setDecimals("12"); setMaxSupply("1000000"); setInitialSupply("0"); }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Plus className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Create Token
            </DialogTitle>
            <DialogDescription>
              Define a new fungible token on the Norn network.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-1">
            <div className="space-y-3">
              <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Token Identity
              </p>
              <div className="space-y-2">
                <Label>Token Name</Label>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="My Token"
                />
              </div>
              <div className="space-y-2">
                <Label>Symbol</Label>
                <Input
                  value={symbol}
                  onChange={(e) => setSymbol(e.target.value)}
                  placeholder="TKN"
                  className="uppercase"
                />
              </div>
            </div>

            <Separator />

            <div className="space-y-3">
              <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Supply Configuration
              </p>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-2">
                  <Label>Decimals</Label>
                  <Input
                    type="number"
                    min={0}
                    max={18}
                    value={decimals}
                    onChange={(e) => {
                      const v = Math.max(0, Math.min(18, parseInt(e.target.value) || 0));
                      setDecimals(String(v));
                    }}
                  />
                  <p className="text-[11px] text-muted-foreground">0–18</p>
                </div>
                <div className="space-y-2">
                  <Label>Max Supply</Label>
                  <Input
                    value={maxSupply}
                    onChange={(e) => setMaxSupply(e.target.value)}
                    placeholder="0 = unlimited"
                  />
                  <p className="text-[11px] text-muted-foreground">0 = unlimited</p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>Initial Supply</Label>
                <Input
                  value={initialSupply}
                  onChange={(e) => setInitialSupply(e.target.value)}
                  placeholder="0"
                />
                <p className="text-[11px] text-muted-foreground">
                  Minted to your address on creation. Leave 0 to mint later.
                </p>
              </div>
            </div>

            <div className="rounded-lg bg-secondary/50 px-3 py-2">
              <p className="text-xs text-muted-foreground">
                Network fee: <span className="text-foreground font-medium">10 NORN</span>
              </p>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleCreate}
              disabled={loading || !name || !symbol}
            >
              {loading ? (
                <span className="flex items-center gap-2">
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Creating...
                </span>
              ) : (
                <>
                  <Fingerprint className="mr-2 h-4 w-4" />
                  Create Token
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Mint Token Dialog */}
      <Dialog open={mintOpen} onOpenChange={(open) => {
        setMintOpen(open);
        if (!open) { setMintAmount(""); setMintTo(""); }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <ArrowUpCircle className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Mint {mintSymbol || "Tokens"}
            </DialogTitle>
            <DialogDescription>
              Mint new tokens to a recipient address (creator only).
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-1">
            <div className="space-y-2">
              <Label>Token ID</Label>
              <Input value={mintTokenId} readOnly className="font-mono text-xs bg-secondary/50" />
            </div>
            <div className="space-y-2">
              <Label>Recipient</Label>
              <Input
                value={mintTo}
                onChange={(e) => setMintTo(e.target.value)}
                placeholder={activeAddress ?? "0x..."}
                className="font-mono text-sm"
              />
              <p className="text-[11px] text-muted-foreground">
                Defaults to your address if left unchanged
              </p>
            </div>
            <div className="space-y-2">
              <Label>Amount</Label>
              <Input
                value={mintAmount}
                onChange={(e) => setMintAmount(e.target.value)}
                placeholder="1000"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setMintOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleMint}
              disabled={loading || !mintAmount}
            >
              {loading ? (
                <span className="flex items-center gap-2">
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Minting...
                </span>
              ) : (
                <>
                  <ArrowUpCircle className="mr-2 h-4 w-4" />
                  Mint
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Burn Token Dialog */}
      <Dialog open={burnOpen} onOpenChange={(open) => {
        setBurnOpen(open);
        if (!open) { setBurnAmount(""); setBurnConfirmText(""); }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Flame className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Burn {burnSymbol || "Tokens"}
            </DialogTitle>
            <DialogDescription>
              Permanently destroy tokens from your balance. This cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-1">
            <div className="space-y-2">
              <Label>Token ID</Label>
              <Input value={burnTokenId} readOnly className="font-mono text-xs bg-secondary/50" />
            </div>
            <div className="space-y-2">
              <Label>Amount to Burn</Label>
              <Input
                value={burnAmount}
                onChange={(e) => setBurnAmount(e.target.value)}
                placeholder="100"
              />
            </div>
            {burnAmount && (
              <div className="space-y-2">
                <Label className="text-xs text-destructive">
                  Type &quot;{burnAmount}&quot; to confirm
                </Label>
                <Input
                  value={burnConfirmText}
                  onChange={(e) => setBurnConfirmText(e.target.value)}
                  placeholder="Type amount to confirm"
                  className="text-sm"
                />
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setBurnOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleBurn}
              disabled={loading || !burnAmount || burnConfirmText !== burnAmount}
            >
              {loading ? (
                <span className="flex items-center gap-2">
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Burning...
                </span>
              ) : (
                <>
                  <Flame className="mr-2 h-4 w-4" />
                  Burn Tokens
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  );
}
