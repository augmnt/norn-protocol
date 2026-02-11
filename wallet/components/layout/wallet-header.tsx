"use client";

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import Link from "next/link";
import { useWallet } from "@/hooks/use-wallet";
import { useNetwork } from "@/hooks/use-network";
import { usePasskeyAuth } from "@/hooks/use-passkey-auth";
import { useWalletStore } from "@/stores/wallet-store";
import { useRealtimeStore } from "@/stores/realtime-store";
import { useContactsStore } from "@/stores/contacts-store";
import { addAccount } from "@/lib/wallet-manager";
import { Button } from "@/components/ui/button";
import { Identicon } from "@/components/ui/identicon";
import { formatNorn, truncateAddress } from "@/lib/format";
import { explorerAddressUrl } from "@/lib/explorer";
import {
  Lock,
  Bell,
  ChevronDown,
  Check,
  Plus,
  ExternalLink,
  User,
} from "lucide-react";
import { toast } from "sonner";

// ---------------------------------------------------------------------------
// Hook: click-outside to close dropdowns
// ---------------------------------------------------------------------------
function useClickOutside(
  ref: React.RefObject<HTMLElement | null>,
  onClose: () => void
) {
  useEffect(() => {
    function handler(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    }
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [ref, onClose]);
}

// ---------------------------------------------------------------------------
// Notification item type (session-only)
// ---------------------------------------------------------------------------
interface Notification {
  id: string;
  from: string;
  to: string;
  amount: string;
  timestamp: number;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------
export function WalletHeader() {
  const { activeAccount, activeAccountIndex, accounts, meta, prfSupported } =
    useWallet();
  const { network } = useNetwork();
  const { lock } = usePasskeyAuth();
  const recentTransfers = useRealtimeStore((s) => s.recentTransfers);
  const getContactLabel = useContactsStore((s) => s.getContactLabel);

  // ---- dropdown open states ----
  const [accountOpen, setAccountOpen] = useState(false);
  const [notifOpen, setNotifOpen] = useState(false);

  // ---- refs for click-outside ----
  const accountRef = useRef<HTMLDivElement>(null);
  const notifRef = useRef<HTMLDivElement>(null);

  useClickOutside(
    accountRef,
    useCallback(() => setAccountOpen(false), [])
  );
  useClickOutside(
    notifRef,
    useCallback(() => setNotifOpen(false), [])
  );

  // ---- notifications (session-only) ----
  const notificationsRef = useRef<Notification[]>([]);
  const [readCount, setReadCount] = useState(0);
  const [, forceRender] = useState(0);

  // Track the active address set for filtering incoming transfers
  const addressSet = useMemo(() => {
    const set = new Set<string>();
    for (const acct of accounts) {
      set.add(acct.address.toLowerCase());
    }
    return set;
  }, [accounts]);

  // Sync incoming transfers into notifications ref
  const prevTransferCountRef = useRef(0);
  useEffect(() => {
    if (recentTransfers.length <= prevTransferCountRef.current) {
      prevTransferCountRef.current = recentTransfers.length;
      return;
    }

    const newCount = recentTransfers.length - prevTransferCountRef.current;
    const incoming = recentTransfers.slice(0, newCount).filter((t) => {
      return addressSet.has(t.to.toLowerCase());
    });

    if (incoming.length > 0) {
      const newNotifs: Notification[] = incoming.map((t, i) => ({
        id: `${Date.now()}-${i}`,
        from: t.from,
        to: t.to,
        amount: t.amount,
        timestamp: Date.now(),
      }));
      notificationsRef.current = [
        ...newNotifs,
        ...notificationsRef.current,
      ].slice(0, 50);
      forceRender((n) => n + 1);
    }

    prevTransferCountRef.current = recentTransfers.length;
  }, [recentTransfers, addressSet]);

  const notifications = notificationsRef.current;
  const unreadCount = Math.max(0, notifications.length - readCount);
  const displayNotifs = notifications.slice(0, 10);

  function markAllRead() {
    setReadCount(notifications.length);
  }

  // ---- account switching ----
  function switchAccount(index: number) {
    useWalletStore.getState().setActiveAccountIndex(index);
    setAccountOpen(false);
  }

  // ---- add account (PRF only) ----
  const [addingAccount, setAddingAccount] = useState(false);
  async function handleAddAccount() {
    if (!meta || addingAccount) return;
    setAddingAccount(true);
    try {
      const name = `Account ${accounts.length + 1}`;
      await addAccount(name, meta);
      // Reload meta so UI picks up the new account
      const { loadWalletMeta } = await import("@/lib/passkey-storage");
      const refreshed = await loadWalletMeta();
      useWalletStore.getState().setMeta(refreshed);
    } catch (e) {
      // DOMException name "NotAllowedError" = user cancelled passkey prompt
      if (e instanceof DOMException && e.name === "NotAllowedError") return;
      toast.error("Failed to add account");
    } finally {
      setAddingAccount(false);
    }
  }

  // ---- time-ago helper ----
  function timeAgo(ts: number): string {
    const diff = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function labelFor(address: string): string {
    // Check own accounts first
    const own = accounts.find(
      (a) => a.address.toLowerCase() === address.toLowerCase()
    );
    if (own) return own.label;
    // Check contacts
    return getContactLabel(address) || truncateAddress(address);
  }

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex h-14 items-center justify-between px-4 sm:px-6 lg:px-8">
        {/* ---- Brand ---- */}
        <Link href="/dashboard" className="flex items-center space-x-2">
          <span className="font-mono text-xl font-bold tracking-[-0.02em] text-foreground">
            norn
          </span>
          <span className="text-xs text-muted-foreground hidden sm:inline">
            wallet
          </span>
        </Link>

        <div className="flex items-center gap-2">
          {/* ---- Network badge ---- */}
          <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
            {network.name}
          </span>

          {/* ---- Account switcher ---- */}
          {activeAccount && (
            <div ref={accountRef} className="relative">
              <button
                onClick={() => {
                  setAccountOpen((o) => !o);
                  setNotifOpen(false);
                }}
                className="inline-flex items-center gap-2 rounded-md px-2.5 py-1.5 text-sm transition-colors hover:bg-accent/50"
              >
                <Identicon
                  address={activeAccount.address}
                  size={22}
                  className="shrink-0"
                />
                <span className="hidden sm:inline text-xs font-medium text-foreground max-w-[100px] truncate">
                  {activeAccount.label}
                </span>
                <span className="hidden md:inline text-xs font-mono text-muted-foreground">
                  {truncateAddress(activeAccount.address)}
                </span>
                <ChevronDown className="h-3 w-3 text-muted-foreground" />
              </button>

              {accountOpen && (
                <div className="absolute right-0 top-full mt-1 w-72 rounded-lg border bg-popover shadow-lg">
                  <div className="px-3 py-2 border-b">
                    <p className="text-xs font-medium text-muted-foreground">
                      Accounts
                    </p>
                  </div>

                  <div className="max-h-64 overflow-y-auto py-1">
                    {accounts.map((acct, i) => {
                      const isActive = i === activeAccountIndex;
                      return (
                        <button
                          key={acct.address}
                          onClick={() => switchAccount(i)}
                          className={`flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-accent/50 ${
                            isActive ? "bg-accent/30" : ""
                          }`}
                        >
                          <Identicon
                            address={acct.address}
                            size={28}
                            className="shrink-0"
                          />
                          <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium text-foreground truncate">
                              {acct.label}
                            </p>
                            <p className="text-xs font-mono text-muted-foreground">
                              {truncateAddress(acct.address)}
                            </p>
                          </div>
                          {isActive && (
                            <Check className="h-4 w-4 text-primary shrink-0" />
                          )}
                        </button>
                      );
                    })}
                  </div>

                  {/* ---- Explorer link for active account ---- */}
                  <div className="border-t px-3 py-2">
                    <a
                      href={explorerAddressUrl(activeAccount.address)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2 text-xs text-muted-foreground transition-colors hover:text-foreground"
                    >
                      <ExternalLink className="h-3 w-3" />
                      View in explorer
                    </a>
                  </div>

                  {/* ---- Add Account (PRF wallets only) ---- */}
                  {prfSupported && meta?.usesPrf && (
                    <div className="border-t px-3 py-2">
                      <button
                        onClick={handleAddAccount}
                        disabled={addingAccount}
                        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:text-foreground hover:bg-accent/50 disabled:opacity-50"
                      >
                        <Plus className="h-3.5 w-3.5" />
                        {addingAccount ? "Adding..." : "Add Account"}
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* ---- Notification bell ---- */}
          <div ref={notifRef} className="relative">
            <Button
              variant="ghost"
              size="icon"
              className="h-9 w-9 relative"
              onClick={() => {
                setNotifOpen((o) => !o);
                setAccountOpen(false);
              }}
              title="Notifications"
            >
              <Bell className="h-3.5 w-3.5" />
              {unreadCount > 0 && (
                <span className="absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-destructive px-1 text-[10px] font-bold text-destructive-foreground">
                  {unreadCount > 99 ? "99+" : unreadCount}
                </span>
              )}
            </Button>

            {notifOpen && (
              <div className="absolute right-0 top-full mt-1 w-80 rounded-lg border bg-popover shadow-lg">
                <div className="flex items-center justify-between px-3 py-2 border-b">
                  <p className="text-xs font-medium text-muted-foreground">
                    Notifications
                  </p>
                  {notifications.length > 0 && (
                    <button
                      onClick={markAllRead}
                      className="text-[11px] text-primary hover:underline"
                    >
                      Mark all read
                    </button>
                  )}
                </div>

                <div className="max-h-72 overflow-y-auto">
                  {displayNotifs.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                      <Bell className="h-6 w-6 mb-2 opacity-40" />
                      <p className="text-xs">No notifications yet</p>
                    </div>
                  ) : (
                    displayNotifs.map((notif, idx) => {
                      const isUnread = idx < unreadCount;
                      return (
                        <div
                          key={notif.id}
                          className={`flex items-start gap-3 px-3 py-2.5 border-b last:border-0 transition-colors ${
                            isUnread ? "bg-accent/20" : ""
                          }`}
                        >
                          <Identicon
                            address={notif.from}
                            size={24}
                            className="shrink-0 mt-0.5"
                          />
                          <div className="flex-1 min-w-0">
                            <p className="text-xs text-foreground">
                              <span className="font-medium">
                                {labelFor(notif.from)}
                              </span>{" "}
                              sent you{" "}
                              <span className="font-mono font-medium">
                                {formatNorn(notif.amount)} NORN
                              </span>
                            </p>
                            <p className="text-[11px] text-muted-foreground mt-0.5">
                              {timeAgo(notif.timestamp)}
                            </p>
                          </div>
                          {isUnread && (
                            <span className="mt-1.5 h-1.5 w-1.5 rounded-full bg-primary shrink-0" />
                          )}
                        </div>
                      );
                    })
                  )}
                </div>
              </div>
            )}
          </div>

          {/* ---- Lock button ---- */}
          <Button
            variant="ghost"
            size="icon"
            onClick={lock}
            title="Lock wallet"
            className="h-9 w-9"
          >
            <Lock className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
    </header>
  );
}
