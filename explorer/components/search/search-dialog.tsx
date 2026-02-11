"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import * as VisuallyHidden from "@radix-ui/react-visually-hidden";
import {
  Search,
  Blocks,
  Wallet,
  Tag,
  Coins,
  FileCode2,
  Loader2,
  Hash,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  isBlockHeight,
  isValidAddress,
  isValidHash,
  truncateHash,
  truncateAddress,
} from "@/lib/format";
import { rpcCall } from "@/lib/rpc";
import type { NameResolution, TokenInfo, LoomInfo, TransactionHistoryEntry } from "@/types";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface SearchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface SearchResult {
  id: string;
  category: "block" | "address" | "name" | "token" | "contract" | "hash";
  label: string;
  sublabel?: string;
  href: string;
}

// ---------------------------------------------------------------------------
// Category metadata
// ---------------------------------------------------------------------------

const CATEGORY_META: Record<
  SearchResult["category"],
  { icon: typeof Search; title: string }
> = {
  block: { icon: Blocks, title: "Blocks" },
  address: { icon: Wallet, title: "Addresses" },
  hash: { icon: Hash, title: "Hashes" },
  name: { icon: Tag, title: "Names" },
  token: { icon: Coins, title: "Tokens" },
  contract: { icon: FileCode2, title: "Contracts" },
};

const CATEGORY_ORDER: SearchResult["category"][] = [
  "block",
  "address",
  "hash",
  "name",
  "token",
  "contract",
];

function groupResults(
  results: SearchResult[]
): Map<SearchResult["category"], SearchResult[]> {
  const grouped = new Map<SearchResult["category"], SearchResult[]>();
  for (const cat of CATEGORY_ORDER) {
    const items = results.filter((r) => r.category === cat);
    if (items.length > 0) grouped.set(cat, items);
  }
  return grouped;
}

// ---------------------------------------------------------------------------
// Search logic
// ---------------------------------------------------------------------------

async function performSearch(query: string): Promise<SearchResult[]> {
  const trimmed = query.trim();
  if (!trimmed) return [];

  const results: SearchResult[] = [];

  // Deterministic results from input classification
  if (isBlockHeight(trimmed)) {
    results.push({
      id: `block-${trimmed}`,
      category: "block",
      label: `Block #${Number(trimmed).toLocaleString()}`,
      href: `/block/${trimmed}`,
    });
  }

  if (isValidAddress(trimmed)) {
    results.push({
      id: `address-${trimmed}`,
      category: "address",
      label: truncateAddress(trimmed),
      sublabel: trimmed,
      href: `/address/${trimmed}`,
    });
  }

  // Detect transaction hash — 64 hex chars with or without 0x prefix
  const isHashWithPrefix = isValidHash(trimmed);
  const isRawHash = /^[a-fA-F0-9]{64}$/.test(trimmed);
  const txLookupId = isHashWithPrefix
    ? trimmed.slice(2)
    : isRawHash
      ? trimmed
      : null;

  // RPC queries in parallel (include tx lookup if hash detected)
  const rpcCalls: [
    Promise<NameResolution | null>,
    Promise<TokenInfo[]>,
    Promise<LoomInfo[]>,
    Promise<TransactionHistoryEntry | null>,
  ] = [
    rpcCall<NameResolution | null>("norn_resolveName", [trimmed]),
    rpcCall<TokenInfo[]>("norn_listTokens", [20, 0]),
    rpcCall<LoomInfo[]>("norn_listLooms", [20, 0]),
    txLookupId
      ? rpcCall<TransactionHistoryEntry | null>("norn_getTransaction", [txLookupId])
      : Promise.resolve(null),
  ];

  const [nameResult, tokens, looms, txResult] = await Promise.allSettled(rpcCalls);

  // Transaction lookup result
  if (txLookupId) {
    const txFound = txResult.status === "fulfilled" && txResult.value != null;
    const displayHash = isHashWithPrefix ? trimmed : `0x${trimmed}`;
    results.push({
      id: `hash-tx-${displayHash}`,
      category: "hash",
      label: txFound ? "Transaction found" : "Look up transaction",
      sublabel: truncateHash(displayHash),
      href: `/tx/${displayHash}`,
    });
  }

  // Name resolution
  if (nameResult.status === "fulfilled" && nameResult.value) {
    const resolved = nameResult.value;
    results.push({
      id: `name-${resolved.name}`,
      category: "name",
      label: resolved.name,
      sublabel: truncateAddress(resolved.owner),
      href: `/address/${resolved.owner}`,
    });
  }

  // Token filtering (client-side match by symbol or name)
  if (tokens.status === "fulfilled" && tokens.value) {
    const lowerQuery = trimmed.toLowerCase();
    const matched = tokens.value.filter(
      (t) =>
        t.symbol.toLowerCase().includes(lowerQuery) ||
        t.name.toLowerCase().includes(lowerQuery)
    );
    for (const token of matched.slice(0, 5)) {
      results.push({
        id: `token-${token.token_id}`,
        category: "token",
        label: `${token.name} (${token.symbol})`,
        sublabel: `${token.decimals} decimals`,
        href: `/token/${token.token_id}`,
      });
    }
  }

  // Loom (contract) filtering (client-side match by name)
  if (looms.status === "fulfilled" && looms.value) {
    const lowerQuery = trimmed.toLowerCase();
    const matched = looms.value.filter((l) =>
      l.name.toLowerCase().includes(lowerQuery)
    );
    for (const loom of matched.slice(0, 5)) {
      results.push({
        id: `contract-${loom.loom_id}`,
        category: "contract",
        label: loom.name,
        sublabel: loom.active ? "Active" : "Inactive",
        href: `/contract/${loom.loom_id}`,
      });
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function SearchDialog({ open, onOpenChange }: SearchDialogProps) {
  const router = useRouter();
  const inputRef = useRef<HTMLInputElement>(null);
  const resultRefs = useRef<Map<number, HTMLButtonElement>>(new Map());

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);

  // Reset state when dialog closes
  useEffect(() => {
    if (!open) {
      setQuery("");
      setResults([]);
      setLoading(false);
      setActiveIndex(0);
    }
  }, [open]);

  // Debounced search with cancellation
  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setResults([]);
      setLoading(false);
      return;
    }

    setLoading(true);
    let cancelled = false;

    const timer = setTimeout(() => {
      performSearch(trimmed)
        .then((res) => {
          if (!cancelled) {
            setResults(res);
            setActiveIndex(0);
            setLoading(false);
          }
        })
        .catch(() => {
          if (!cancelled) {
            setResults([]);
            setLoading(false);
          }
        });
    }, 300);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query]);

  const navigate = useCallback(
    (href: string) => {
      onOpenChange(false);
      router.push(href);
    },
    [onOpenChange, router]
  );

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const count = results.length;
      if (!count) return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setActiveIndex((prev) => (prev < count - 1 ? prev + 1 : 0));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setActiveIndex((prev) => (prev > 0 ? prev - 1 : count - 1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const selected = results[activeIndex];
        if (selected) navigate(selected.href);
      }
    },
    [results, activeIndex, navigate]
  );

  // Scroll active item into view
  useEffect(() => {
    const el = resultRefs.current.get(activeIndex);
    if (el) el.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  const grouped = groupResults(results);
  const hasResults = results.length > 0;
  const showEmpty = !loading && query.trim().length > 0 && !hasResults;

  // Track global index across grouped categories for keyboard nav
  let globalIndex = 0;

  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
        <DialogPrimitive.Content
          onOpenAutoFocus={(e) => {
            e.preventDefault();
            inputRef.current?.focus();
          }}
          className={cn(
            "fixed left-[50%] top-[20%] z-50 w-full max-w-lg translate-x-[-50%] translate-y-0",
            "rounded-lg border border-border bg-popover shadow-2xl shadow-black/20",
            "duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
            "data-[state=closed]:slide-out-to-left-1/2 data-[state=open]:slide-in-from-left-1/2",
            "data-[state=closed]:slide-out-to-top-[2%] data-[state=open]:slide-in-from-top-[2%]",
            "outline-none"
          )}
          onKeyDown={handleKeyDown}
        >
          <VisuallyHidden.Root>
            <DialogPrimitive.Title>Search</DialogPrimitive.Title>
            <DialogPrimitive.Description>
              Search for blocks, addresses, tokens, names, and contracts.
            </DialogPrimitive.Description>
          </VisuallyHidden.Root>

          {/* Search input */}
          <div className="flex items-center gap-3 border-b border-border px-4 py-3">
            {loading ? (
              <Loader2 className="h-4 w-4 shrink-0 text-muted-foreground animate-spin" />
            ) : (
              <Search className="h-4 w-4 shrink-0 text-muted-foreground" />
            )}
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search blocks, addresses, tokens, names..."
              className={cn(
                "flex-1 bg-transparent font-mono text-sm text-foreground",
                "placeholder:text-muted-foreground/60",
                "outline-none"
              )}
              spellCheck={false}
              autoComplete="off"
            />
            <kbd className="hidden sm:inline-flex h-5 select-none items-center rounded border border-border bg-muted px-1.5 font-mono text-[10px] text-muted-foreground">
              ESC
            </kbd>
          </div>

          {/* Results */}
          <div className="max-h-[320px] overflow-y-auto scrollbar-thin">
            {/* Grouped results */}
            {hasResults && (
              <div className="py-2">
                {CATEGORY_ORDER.map((category) => {
                  const items = grouped.get(category);
                  if (!items) return null;

                  const meta = CATEGORY_META[category];
                  const Icon = meta.icon;

                  return (
                    <div key={category}>
                      <div className="px-4 py-1.5">
                        <span className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60">
                          {meta.title}
                        </span>
                      </div>
                      {items.map((result) => {
                        const idx = globalIndex++;
                        const isActive = idx === activeIndex;

                        return (
                          <button
                            key={result.id}
                            ref={(el) => {
                              if (el) resultRefs.current.set(idx, el);
                              else resultRefs.current.delete(idx);
                            }}
                            onClick={() => navigate(result.href)}
                            onMouseEnter={() => setActiveIndex(idx)}
                            className={cn(
                              "flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm",
                              "transition-colors duration-75",
                              isActive
                                ? "bg-accent text-accent-foreground"
                                : "text-foreground/80 hover:bg-accent/50"
                            )}
                          >
                            <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
                            <div className="flex flex-col gap-0.5 overflow-hidden">
                              <span className="truncate font-medium">
                                {result.label}
                              </span>
                              {result.sublabel && (
                                <span className="truncate font-mono text-xs text-muted-foreground">
                                  {result.sublabel}
                                </span>
                              )}
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  );
                })}
              </div>
            )}

            {/* Empty state */}
            {showEmpty && (
              <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
                <Search className="h-8 w-8 opacity-30" />
                <p className="text-sm">No results found</p>
                <p className="text-xs text-muted-foreground/60">
                  Try a block height, address, token name, or registered name
                </p>
              </div>
            )}

            {/* Initial state */}
            {!loading && query.trim().length === 0 && (
              <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
                <p className="text-xs text-muted-foreground/60">
                  Type to search across the Norn network
                </p>
              </div>
            )}
          </div>

          {/* Footer keyboard hints */}
          {hasResults && (
            <div className="flex items-center gap-4 border-t border-border px-4 py-2 text-[11px] text-muted-foreground/50">
              <span className="inline-flex items-center gap-1">
                <kbd className="rounded border border-border bg-muted px-1 py-0.5 font-mono text-[10px]">
                  &uarr;&darr;
                </kbd>
                navigate
              </span>
              <span className="inline-flex items-center gap-1">
                <kbd className="rounded border border-border bg-muted px-1 py-0.5 font-mono text-[10px]">
                  &crarr;
                </kbd>
                select
              </span>
              <span className="inline-flex items-center gap-1">
                <kbd className="rounded border border-border bg-muted px-1 py-0.5 font-mono text-[10px]">
                  esc
                </kbd>
                close
              </span>
            </div>
          )}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
