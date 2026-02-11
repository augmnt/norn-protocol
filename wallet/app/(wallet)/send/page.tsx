"use client";

import { useState, useMemo, useEffect, useRef, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import { useQueries } from "@tanstack/react-query";
import { useWallet } from "@/hooks/use-wallet";
import { useBalance } from "@/hooks/use-balance";
import { useTokenBalances } from "@/hooks/use-token-balances";
import { useSend } from "@/hooks/use-send";
import { useContactsStore } from "@/stores/contacts-store";
import { rpcCall } from "@/lib/rpc";
import { NATIVE_TOKEN_ID, QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { formatAmount, isValidAddress, strip0x, truncateAddress } from "@/lib/format";
import {
  Fingerprint,
  ArrowUpRight,
  Check,
  ChevronDown,
  MessageSquare,
  Coins,
  UserPlus,
} from "lucide-react";
import { toast } from "sonner";
import type { NameResolution, TokenInfo, BalanceEntry } from "@/types";

// ---------------------------------------------------------------------------
// Token option used in the dropdown
// ---------------------------------------------------------------------------
interface TokenOption {
  tokenId: string;
  symbol: string;
  name: string;
  decimals: number;
  balance: string;
  isNative: boolean;
}

// ---------------------------------------------------------------------------
// Inner component that reads search params (must be wrapped in Suspense)
// ---------------------------------------------------------------------------
function SendPageInner() {
  const { activeAddress } = useWallet();
  const { send, sending, error } = useSend();
  const { recentRecipients, getContactLabel, addRecentRecipient, addContact, isContact } =
    useContactsStore();

  // ---- Search params: pre-select token via ?token=<id> ----
  const searchParams = useSearchParams();
  const preselectedToken = searchParams.get("token") ?? undefined;

  // ---- Token balances ----
  const { data: threadState } = useTokenBalances(activeAddress ?? undefined);

  const nonZeroBalances = useMemo(
    () =>
      threadState?.balances?.filter(
        (b: BalanceEntry) => BigInt(b.amount || "0") > 0n
      ) ?? [],
    [threadState]
  );

  // IDs for which we need to fetch TokenInfo (non-native, non-zero balance)
  const nonNativeIds = useMemo(
    () =>
      nonZeroBalances
        .filter((b: BalanceEntry) => b.token_id !== NATIVE_TOKEN_ID)
        .map((b: BalanceEntry) => b.token_id),
    [nonZeroBalances]
  );

  // Parallel token info fetches
  const tokenInfoQueries = useQueries({
    queries: nonNativeIds.map((id: string) => ({
      queryKey: QUERY_KEYS.tokenInfo(id),
      queryFn: () => rpcCall<TokenInfo>("norn_getTokenInfo", [id]),
      staleTime: STALE_TIMES.semiStatic,
    })),
  });

  // Build enriched token options
  const tokenOptions: TokenOption[] = useMemo(() => {
    const infoMap = new Map<string, TokenInfo>();
    nonNativeIds.forEach((id: string, i: number) => {
      const data = tokenInfoQueries[i]?.data;
      if (data) infoMap.set(id, data);
    });

    const options: TokenOption[] = nonZeroBalances.map((b: BalanceEntry) => {
      const isNative = b.token_id === NATIVE_TOKEN_ID;
      const info = infoMap.get(b.token_id);
      return {
        tokenId: b.token_id,
        symbol: isNative ? "NORN" : info?.symbol ?? "???",
        name: isNative ? "Norn Protocol" : info?.name ?? "Unknown Token",
        decimals: isNative ? 12 : info?.decimals ?? 12,
        balance: b.amount,
        isNative,
      };
    });

    // Ensure NORN appears first even if zero-balance (shouldn't happen, but safety)
    if (!options.some((o) => o.isNative)) {
      options.unshift({
        tokenId: NATIVE_TOKEN_ID,
        symbol: "NORN",
        name: "Norn Protocol",
        decimals: 12,
        balance: "0",
        isNative: true,
      });
    }

    return options;
  }, [nonZeroBalances, nonNativeIds, tokenInfoQueries]);

  // ---- Selected token state ----
  const [selectedTokenId, setSelectedTokenId] = useState<string>(
    preselectedToken ?? NATIVE_TOKEN_ID
  );
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Sync preselectedToken on mount / change
  useEffect(() => {
    if (preselectedToken) {
      setSelectedTokenId(preselectedToken);
    }
  }, [preselectedToken]);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node)
      ) {
        setDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const selectedToken = useMemo(
    () =>
      tokenOptions.find((t) => t.tokenId === selectedTokenId) ??
      tokenOptions[0] ?? {
        tokenId: NATIVE_TOKEN_ID,
        symbol: "NORN",
        name: "Norn Protocol",
        decimals: 12,
        balance: "0",
        isNative: true,
      },
    [tokenOptions, selectedTokenId]
  );

  // Also fetch specific balance for the selected token (for real-time updates)
  const { data: selectedBalance } = useBalance(
    activeAddress ?? undefined,
    selectedTokenId
  );

  // Effective balance: prefer the real-time hook, fall back to threadState entry
  const effectiveBalance = selectedBalance?.balance ?? selectedToken.balance;

  // ---- Form state ----
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [showMemo, setShowMemo] = useState(false);
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [resolvedNornName, setResolvedNornName] = useState<string | null>(null);
  const [resolving, setResolving] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);

  // ---- Contact label for entered address ----
  const effectiveAddress = resolvedAddress || recipient;
  const contactLabel = useMemo(() => {
    if (!effectiveAddress || !isValidAddress(effectiveAddress)) return undefined;
    return getContactLabel(effectiveAddress);
  }, [effectiveAddress, getContactLabel]);

  // ---- NornName resolution ----
  const resolveRecipient = async (value: string) => {
    setRecipient(value);
    setResolvedAddress(null);
    setResolvedNornName(null);

    if (isValidAddress(value)) {
      setResolvedAddress(value);
      return;
    }

    // Try resolving as a NornName
    if (value.length >= 3 && !value.startsWith("0x")) {
      setResolving(true);
      try {
        const result = await rpcCall<NameResolution | null>(
          "norn_resolveName",
          [value]
        );
        if (result?.owner) {
          setResolvedAddress(result.owner);
          setResolvedNornName(value);
        }
      } catch {
        // Ignore resolution errors
      } finally {
        setResolving(false);
      }
    }
  };

  // ---- Max amount (accounts for fee only on native token) ----
  const handleSetMax = () => {
    const raw = BigInt(effectiveBalance || "0");
    if (raw === 0n) return;

    if (selectedToken.isNative) {
      // Reserve ~0.01 NORN for fees
      const fee = BigInt("10000000000"); // 0.01 NORN
      const max = raw > fee ? raw - fee : 0n;
      const divisor = BigInt(10 ** selectedToken.decimals);
      const whole = max / divisor;
      const frac = (max % divisor)
        .toString()
        .padStart(selectedToken.decimals, "0")
        .replace(/0+$/, "");
      setAmount(frac ? `${whole}.${frac}` : whole.toString());
    } else {
      const divisor = BigInt(10 ** selectedToken.decimals);
      const whole = raw / divisor;
      const frac = (raw % divisor)
        .toString()
        .padStart(selectedToken.decimals, "0")
        .replace(/0+$/, "");
      setAmount(frac ? `${whole}.${frac}` : whole.toString());
    }
  };

  // ---- Confirm & send ----
  const handleConfirm = async () => {
    const to = resolvedAddress || recipient;
    try {
      await send({
        to: strip0x(to),
        amount,
        tokenId: selectedToken.isNative ? undefined : selectedToken.tokenId,
        memo: memo || undefined,
      });
      setConfirmOpen(false);
      toast.success("Transaction submitted");

      // Add recipient to recent recipients
      addRecentRecipient(to);

      // Reset form
      setRecipient("");
      setAmount("");
      setMemo("");
      setShowMemo(false);
      setResolvedAddress(null);
      setResolvedNornName(null);
    } catch {
      toast.error(error || "Transaction failed");
    }
  };

  // ---- Validation ----
  const parsedAmount = Number(amount);
  const isSelfSend = resolvedAddress
    ? resolvedAddress.toLowerCase() === activeAddress?.toLowerCase()
    : recipient.toLowerCase() === activeAddress?.toLowerCase();
  const memoBytes = new TextEncoder().encode(memo).length;
  const canSend =
    (resolvedAddress || isValidAddress(recipient)) &&
    parsedAmount > 0 &&
    Number.isFinite(parsedAmount) &&
    memoBytes <= 31;

  // ---- Recent recipients (only show when input is empty) ----
  const showRecentRecipients = !recipient && recentRecipients.length > 0;

  return (
    <PageContainer title="Send" description="Transfer NORN or tokens">
      <div className="max-w-lg mx-auto">
        <Card>
          <CardContent className="pt-6 space-y-5">
            {/* Token Selector */}
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                Token
              </Label>
              <div className="relative" ref={dropdownRef}>
                <button
                  type="button"
                  onClick={() => setDropdownOpen((o) => !o)}
                  className="flex w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2.5 text-sm shadow-sm transition-colors hover:bg-secondary/50 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  <div className="flex items-center gap-2.5">
                    <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                      <Coins className="h-3.5 w-3.5 text-muted-foreground" />
                    </div>
                    <div className="flex flex-col items-start">
                      <span className="font-medium text-sm leading-tight">
                        {selectedToken.symbol}
                      </span>
                      <span className="text-[11px] text-muted-foreground leading-tight">
                        {selectedToken.name}
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground font-mono tabular-nums">
                      {formatAmount(effectiveBalance, selectedToken.decimals)}
                    </span>
                    <ChevronDown
                      className={`h-4 w-4 text-muted-foreground transition-transform ${
                        dropdownOpen ? "rotate-180" : ""
                      }`}
                    />
                  </div>
                </button>

                {dropdownOpen && (
                  <div className="absolute z-50 mt-1 w-full rounded-md border border-input bg-background shadow-lg max-h-60 overflow-y-auto">
                    {tokenOptions.map((opt) => (
                      <button
                        key={opt.tokenId}
                        type="button"
                        onClick={() => {
                          setSelectedTokenId(opt.tokenId);
                          setDropdownOpen(false);
                          // Reset amount when switching tokens
                          setAmount("");
                        }}
                        className={`flex w-full items-center justify-between px-3 py-2.5 text-sm transition-colors hover:bg-secondary/60 ${
                          opt.tokenId === selectedTokenId
                            ? "bg-secondary/40"
                            : ""
                        }`}
                      >
                        <div className="flex items-center gap-2.5">
                          <div className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary">
                            <Coins className="h-3 w-3 text-muted-foreground" />
                          </div>
                          <div className="flex flex-col items-start">
                            <span className="font-medium text-sm leading-tight">
                              {opt.symbol}
                            </span>
                            {opt.isNative && (
                              <span className="text-[10px] text-muted-foreground leading-tight">
                                Native
                              </span>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-muted-foreground font-mono tabular-nums">
                            {formatAmount(opt.balance, opt.decimals)}
                          </span>
                          {opt.tokenId === selectedTokenId && (
                            <Check className="h-3.5 w-3.5 text-green-500" />
                          )}
                        </div>
                      </button>
                    ))}
                    {tokenOptions.length === 0 && (
                      <div className="px-3 py-4 text-center text-sm text-muted-foreground">
                        No tokens found
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>

            {/* Recipient */}
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                Recipient
              </Label>
              <Input
                value={recipient}
                onChange={(e) => resolveRecipient(e.target.value)}
                placeholder="0x address or NornName"
                className="font-mono text-sm h-11"
              />
              {resolving && (
                <p className="text-xs text-muted-foreground animate-pulse">
                  Resolving name...
                </p>
              )}
              {resolvedAddress && resolvedAddress !== recipient && (
                <div className="flex items-center gap-1.5">
                  <Badge
                    variant="secondary"
                    className="bg-green-500/10 text-green-500 border-0 text-xs font-mono"
                  >
                    <Check className="h-3 w-3 mr-1" />
                    {truncateAddress(resolvedAddress)}
                  </Badge>
                </div>
              )}
              {/* Contact label display */}
              {contactLabel && (
                <div className="flex items-center gap-1.5">
                  <Badge
                    variant="secondary"
                    className="border-border text-muted-foreground text-xs"
                  >
                    {contactLabel}
                  </Badge>
                </div>
              )}
              {/* Save to contacts prompt */}
              {resolvedNornName && resolvedAddress && !isContact(resolvedAddress) && (
                <button
                  type="button"
                  onClick={() => {
                    addContact(resolvedAddress, resolvedNornName, resolvedNornName);
                    toast.success(`Saved ${resolvedNornName} to contacts`);
                  }}
                  className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  <UserPlus className="h-3 w-3" />
                  Save to contacts
                </button>
              )}
              {/* Recent recipients */}
              {showRecentRecipients && (
                <div className="space-y-1.5">
                  <p className="text-[11px] text-muted-foreground">Recent</p>
                  <div className="flex flex-wrap gap-1.5">
                    {recentRecipients.map((addr) => {
                      const label = getContactLabel(addr);
                      return (
                        <button
                          key={addr}
                          type="button"
                          onClick={() => resolveRecipient(addr)}
                          className="inline-flex items-center gap-1 rounded-full border border-input bg-secondary/50 px-2.5 py-1 text-xs font-mono transition-colors hover:bg-secondary hover:border-foreground/20"
                        >
                          {label ? (
                            <span className="font-sans font-medium">
                              {label}
                            </span>
                          ) : (
                            truncateAddress(addr)
                          )}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>

            {/* Amount */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                  Amount
                </Label>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {formatAmount(effectiveBalance, selectedToken.decimals)}{" "}
                    {selectedToken.symbol}
                  </span>
                  <button
                    onClick={handleSetMax}
                    className="text-xs font-semibold uppercase tracking-wider text-foreground transition-colors px-1.5 py-0.5 rounded bg-secondary hover:bg-secondary/80"
                  >
                    Max
                  </button>
                </div>
              </div>
              <div className="relative">
                <Input
                  type="number"
                  step="0.0001"
                  min="0"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.0"
                  className="font-mono text-lg h-12 pr-16"
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground font-medium">
                  {selectedToken.symbol}
                </span>
              </div>
            </div>

            {/* Memo Toggle */}
            {!showMemo ? (
              <button
                onClick={() => setShowMemo(true)}
                className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                <MessageSquare className="h-3 w-3" />
                Add memo
                <ChevronDown className="h-3 w-3" />
              </button>
            ) : (
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                  Memo
                </Label>
                <Input
                  value={memo}
                  onChange={(e) => {
                    const bytes = new TextEncoder().encode(
                      e.target.value
                    ).length;
                    if (bytes <= 31) setMemo(e.target.value);
                  }}
                  placeholder="Optional message (max 31 bytes)"
                  className="text-sm h-10"
                  maxLength={31}
                />
                <p className="text-[11px] text-muted-foreground tabular-nums">
                  {memoBytes}/31 bytes
                </p>
              </div>
            )}

            {/* Self-send warning */}
            {isSelfSend && (
              <p className="text-xs text-yellow-500">
                You are sending to your own address.
              </p>
            )}

            {/* Review Button */}
            <Button
              className="w-full h-11 rounded-lg mt-2"
              disabled={!canSend || sending}
              onClick={() => setConfirmOpen(true)}
            >
              <ArrowUpRight className="mr-2 h-4 w-4" />
              Review Send
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* Confirm Dialog */}
      <Dialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Confirm Transaction</DialogTitle>
            <DialogDescription>
              Review the details before signing.
            </DialogDescription>
          </DialogHeader>
          <div className="rounded-lg border bg-secondary divide-y divide-border">
            <div className="flex items-center justify-between px-4 py-3">
              <span className="text-sm text-muted-foreground">To</span>
              <div className="flex flex-col items-end gap-0.5">
                <span className="font-mono text-xs text-foreground">
                  {truncateAddress(resolvedAddress || recipient)}
                </span>
                {contactLabel && (
                  <span className="text-[11px] text-blue-500">
                    {contactLabel}
                  </span>
                )}
              </div>
            </div>
            <div className="flex items-center justify-between px-4 py-3">
              <span className="text-sm text-muted-foreground">Token</span>
              <span className="text-sm font-medium">
                {selectedToken.symbol}
              </span>
            </div>
            <div className="flex items-center justify-between px-4 py-3">
              <span className="text-sm text-muted-foreground">Amount</span>
              <span className="font-mono text-sm font-medium">
                {amount} {selectedToken.symbol}
              </span>
            </div>
            {memo && (
              <div className="flex items-center justify-between px-4 py-3">
                <span className="text-sm text-muted-foreground">Memo</span>
                <span className="text-sm text-foreground max-w-[200px] truncate">
                  {memo}
                </span>
              </div>
            )}
            <div className="flex items-center justify-between px-4 py-3">
              <span className="text-sm text-muted-foreground">
                Network Fee
              </span>
              <span className="font-mono text-xs text-muted-foreground">
                ~0.01 NORN
              </span>
            </div>
          </div>
          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              onClick={() => setConfirmOpen(false)}
              className="rounded-lg"
            >
              Cancel
            </Button>
            <Button
              onClick={handleConfirm}
              disabled={sending}
              className="rounded-lg"
            >
              {sending ? (
                <span className="flex items-center gap-2">
                  <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Signing...
                </span>
              ) : (
                <>
                  <Fingerprint className="mr-2 h-4 w-4" />
                  Sign & Send
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  );
}

// ---------------------------------------------------------------------------
// Default export wraps in Suspense (required by useSearchParams in Next.js App Router)
// ---------------------------------------------------------------------------
export default function SendPage() {
  return (
    <Suspense
      fallback={
        <PageContainer title="Send" description="Transfer NORN or tokens">
          <div className="max-w-lg mx-auto">
            <Card>
              <CardContent className="pt-6">
                <div className="h-64 flex items-center justify-center">
                  <span className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent text-muted-foreground" />
                </div>
              </CardContent>
            </Card>
          </div>
        </PageContainer>
      }
    >
      <SendPageInner />
    </Suspense>
  );
}
