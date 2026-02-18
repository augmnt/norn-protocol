"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { useNetwork } from "@/hooks/use-network";
import { useSettingsStore } from "@/stores/settings-store";
import {
  LayoutDashboard,
  ArrowUpRight,
  QrCode,
  History,
  Coins,
  AtSign,
  Blocks,
  ArrowLeftRight,
  Droplets,
  MessageSquare,
  Settings,
  MoreHorizontal,
  X,
  PanelLeftClose,
  PanelLeft,
} from "lucide-react";

interface NavItem {
  href: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  testnetOnly?: boolean;
  /** Additional path prefixes that should highlight this nav item */
  matchPrefixes?: string[];
}

const navItems: NavItem[] = [
  { href: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { href: "/send", label: "Send", icon: ArrowUpRight },
  { href: "/receive", label: "Receive", icon: QrCode },
  { href: "/history", label: "History", icon: History },
  { href: "/tokens", label: "Tokens", icon: Coins },
  { href: "/swap", label: "Swap", icon: ArrowLeftRight },
  { href: "/names", label: "Names", icon: AtSign },
  { href: "/discover", label: "Apps", icon: Blocks, matchPrefixes: ["/apps", "/contracts"] },
  // { href: "/chat", label: "Chat", icon: MessageSquare },
  { href: "/faucet", label: "Faucet", icon: Droplets, testnetOnly: true },
  { href: "/settings", label: "Settings", icon: Settings },
];

// Show 5 tabs on mobile: Dashboard, Send, History, Tokens, More
const MOBILE_TAB_COUNT = 4;

function isNavActive(item: NavItem, pathname: string): boolean {
  if (pathname === item.href) return true;
  if (item.matchPrefixes) {
    return item.matchPrefixes.some((prefix) => pathname.startsWith(prefix));
  }
  return false;
}

export function WalletNav() {
  const pathname = usePathname();
  const { isTestnet } = useNetwork();
  const [moreOpen, setMoreOpen] = useState(false);
  const collapsed = useSettingsStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useSettingsStore((s) => s.toggleSidebar);

  const filteredItems = navItems.filter(
    (item) => !item.testnetOnly || isTestnet
  );

  const mobileTabs = filteredItems.slice(0, MOBILE_TAB_COUNT);
  const moreItems = filteredItems.slice(MOBILE_TAB_COUNT);
  const moreActive = moreItems.some((item) => isNavActive(item, pathname));

  // Close menu on navigation
  useEffect(() => {
    setMoreOpen(false);
  }, [pathname]);

  return (
    <>
      {/* Desktop sidebar */}
      <nav
        className={cn(
          "hidden md:flex flex-col border-r p-3 gap-0.5 transition-all duration-200",
          collapsed ? "w-14" : "w-52"
        )}
      >
        {filteredItems.map((item) => {
          const active = isNavActive(item, pathname);
          return (
            <Link
              key={item.href}
              href={item.href}
              title={collapsed ? item.label : undefined}
              className={cn(
                "flex items-center rounded-md transition-colors",
                collapsed ? "justify-center px-0 py-2" : "gap-2.5 px-3 py-2",
                active
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:text-foreground hover:bg-accent/50",
                "text-sm"
              )}
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {!collapsed && item.label}
            </Link>
          );
        })}

        {/* Collapse toggle at bottom */}
        <div className="mt-auto pt-2 border-t border-border">
          <button
            onClick={toggleSidebar}
            title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
            className={cn(
              "flex items-center rounded-md py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors w-full",
              collapsed ? "justify-center px-0" : "gap-2.5 px-3"
            )}
          >
            {collapsed ? (
              <PanelLeft className="h-4 w-4 shrink-0" />
            ) : (
              <>
                <PanelLeftClose className="h-4 w-4 shrink-0" />
                Collapse
              </>
            )}
          </button>
        </div>
      </nav>

      {/* Mobile: More menu overlay */}
      {moreOpen && (
        <>
          <div
            className="fixed inset-0 z-50 bg-black/40 md:hidden"
            onClick={() => setMoreOpen(false)}
          />
          <div className="fixed bottom-0 left-0 right-0 z-50 md:hidden animate-in slide-in-from-bottom duration-200 pb-[env(safe-area-inset-bottom)]">
            <div className="mx-2 mb-2 rounded-xl border bg-background shadow-lg">
              <div className="flex items-center justify-between px-4 py-3 border-b">
                <span className="text-xs font-medium text-muted-foreground">More</span>
                <button
                  onClick={() => setMoreOpen(false)}
                  className="text-muted-foreground hover:text-foreground p-1 touch-manipulation"
                >
                  <X className="h-5 w-5" />
                </button>
              </div>
              <div className="p-3 grid grid-cols-4 gap-2">
                {moreItems.map((item) => {
                  const active = isNavActive(item, pathname);
                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={cn(
                        "flex flex-col items-center gap-1.5 rounded-lg py-4 text-[11px] font-medium transition-colors active:scale-95 touch-manipulation",
                        active
                          ? "text-foreground bg-accent"
                          : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                      )}
                    >
                      <item.icon className="h-6 w-6" />
                      {item.label}
                    </Link>
                  );
                })}
              </div>
            </div>
          </div>
        </>
      )}

      {/* Mobile bottom tabs */}
      <nav className="fixed bottom-0 left-0 right-0 z-40 flex md:hidden border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 pb-[env(safe-area-inset-bottom)]">
        {mobileTabs.map((item) => {
          const active = isNavActive(item, pathname);
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex flex-1 flex-col items-center gap-1 py-2.5 text-[11px] font-medium transition-colors active:scale-95 touch-manipulation",
                active ? "text-foreground" : "text-muted-foreground"
              )}
            >
              <item.icon className="h-5 w-5" />
              {item.label}
            </Link>
          );
        })}
        <button
          onClick={() => setMoreOpen(!moreOpen)}
          className={cn(
            "flex flex-1 flex-col items-center gap-1 py-2.5 text-[11px] font-medium transition-colors active:scale-95 touch-manipulation",
            moreActive || moreOpen ? "text-foreground" : "text-muted-foreground"
          )}
        >
          <MoreHorizontal className="h-5 w-5" />
          More
        </button>
      </nav>
    </>
  );
}
