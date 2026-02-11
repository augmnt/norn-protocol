"use client";

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

export function WalletNav() {
  const pathname = usePathname();
  const { isTestnet } = useNetwork();

  const filteredItems = navItems.filter(
    (item) => !item.testnetOnly || isTestnet
  );

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

      {/* Mobile bottom tabs */}
      <nav className="fixed bottom-0 left-0 right-0 z-40 flex md:hidden border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 pb-[env(safe-area-inset-bottom)]">
        {filteredItems.slice(0, 5).map((item) => {
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
        <Link
          href="/settings"
          className={cn(
            "flex flex-1 flex-col items-center gap-0.5 py-2.5 text-[10px] transition-colors",
            pathname === "/settings" ? "text-foreground" : "text-muted-foreground"
          )}
        >
          <Settings className="h-4 w-4" />
          More
        </Link>
      </nav>
    </>
  );
}
