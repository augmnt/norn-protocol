"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { useNetwork } from "@/hooks/use-network";
import {
  LayoutDashboard,
  ArrowUpRight,
  QrCode,
  History,
  Coins,
  AtSign,
  FileCode,
  Droplets,
  Settings,
  MoreHorizontal,
  X,
} from "lucide-react";

interface NavItem {
  href: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  testnetOnly?: boolean;
}

const navItems: NavItem[] = [
  { href: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { href: "/send", label: "Send", icon: ArrowUpRight },
  { href: "/receive", label: "Receive", icon: QrCode },
  { href: "/history", label: "History", icon: History },
  { href: "/tokens", label: "Tokens", icon: Coins },
  { href: "/names", label: "Names", icon: AtSign },
  { href: "/contracts", label: "Contracts", icon: FileCode },
  { href: "/faucet", label: "Faucet", icon: Droplets, testnetOnly: true },
  { href: "/settings", label: "Settings", icon: Settings },
];

const MOBILE_TAB_COUNT = 4;

export function WalletNav() {
  const pathname = usePathname();
  const { isTestnet } = useNetwork();
  const [moreOpen, setMoreOpen] = useState(false);

  const filteredItems = navItems.filter(
    (item) => !item.testnetOnly || isTestnet
  );

  const mobileTabs = filteredItems.slice(0, MOBILE_TAB_COUNT);
  const moreItems = filteredItems.slice(MOBILE_TAB_COUNT);
  const moreActive = moreItems.some((item) => pathname === item.href);

  // Close menu on navigation
  useEffect(() => {
    setMoreOpen(false);
  }, [pathname]);

  return (
    <>
      {/* Desktop sidebar */}
      <nav className="hidden md:flex w-48 flex-col border-r p-3 gap-0.5">
        {filteredItems.map((item) => {
          const active = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors",
                active
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
              )}
            >
              <item.icon className="h-3.5 w-3.5" />
              {item.label}
            </Link>
          );
        })}
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
                  className="text-muted-foreground hover:text-foreground"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
              <div className="p-2 grid grid-cols-4 gap-1">
                {moreItems.map((item) => {
                  const active = pathname === item.href;
                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={cn(
                        "flex flex-col items-center gap-1 rounded-lg py-3 text-[10px] transition-colors",
                        active
                          ? "text-foreground bg-accent"
                          : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                      )}
                    >
                      <item.icon className="h-5 w-5" />
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
          const active = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex flex-1 flex-col items-center gap-0.5 py-2.5 text-[10px] transition-colors",
                active ? "text-foreground" : "text-muted-foreground"
              )}
            >
              <item.icon className="h-4 w-4" />
              {item.label}
            </Link>
          );
        })}
        <button
          onClick={() => setMoreOpen(!moreOpen)}
          className={cn(
            "flex flex-1 flex-col items-center gap-0.5 py-2.5 text-[10px] transition-colors",
            moreActive || moreOpen ? "text-foreground" : "text-muted-foreground"
          )}
        >
          <MoreHorizontal className="h-4 w-4" />
          More
        </button>
      </nav>
    </>
  );
}
