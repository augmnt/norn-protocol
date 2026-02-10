"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Blocks,
  ArrowRightLeft,
  Coins,
  FileCode2,
  Shield,
  Menu,
  X,
} from "lucide-react";
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { SearchBar } from "@/components/search/search-bar";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/blocks", label: "Blocks", icon: Blocks, shortcut: "G B" },
  { href: "/transactions", label: "Transactions", icon: ArrowRightLeft, shortcut: "G T" },
  { href: "/tokens", label: "Tokens", icon: Coins, shortcut: "G K" },
  { href: "/contracts", label: "Contracts", icon: FileCode2, shortcut: "G C" },
  { href: "/validators", label: "Validators", icon: Shield, shortcut: "G V" },
];

export function Header() {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  // Close mobile nav on route change
  useEffect(() => {
    setMobileOpen(false);
  }, [pathname]);

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 w-full max-w-7xl items-center px-4 sm:px-6 lg:px-8">
        <Link
          href="/"
          className="mr-6 flex items-center space-x-2"
        >
          <span className="font-mono text-xl font-bold tracking-[-0.02em] text-foreground">
            norn
          </span>
          <span className="text-xs text-muted-foreground hidden sm:inline">
            explorer
          </span>
        </Link>

        <nav className="hidden md:flex items-center gap-1">
          {navItems.map((item) => {
            const isActive =
              pathname === item.href || pathname.startsWith(item.href + "/");
            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm transition-colors",
                  isActive
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

        <div className="ml-auto flex items-center gap-2">
          <SearchBar />
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden h-10 w-10"
            onClick={() => setMobileOpen(!mobileOpen)}
          >
            {mobileOpen ? (
              <X className="h-5 w-5" />
            ) : (
              <Menu className="h-5 w-5" />
            )}
          </Button>
        </div>
      </div>

      {/* Mobile nav */}
      <div
        className={cn(
          "overflow-hidden transition-all duration-200 ease-in-out md:hidden",
          mobileOpen ? "max-h-96 border-t" : "max-h-0"
        )}
      >
        <nav className="mx-auto max-w-7xl px-4 py-2 space-y-1">
          {navItems.map((item) => {
            const isActive =
              pathname === item.href || pathname.startsWith(item.href + "/");
            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "flex items-center justify-between rounded-md px-3 py-3 text-sm transition-colors min-h-[44px]",
                  isActive
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                )}
              >
                <span className="flex items-center gap-2">
                  <item.icon className="h-4 w-4" />
                  {item.label}
                </span>
                <kbd className="hidden sm:inline text-[10px] font-mono text-muted-foreground">
                  {item.shortcut}
                </kbd>
              </Link>
            );
          })}
        </nav>
      </div>
    </header>
  );
}
