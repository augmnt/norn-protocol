"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Github, Menu, X, ExternalLink } from "lucide-react";
import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/docs", label: "Docs" },
  { href: "https://explorer.norn.network", label: "Explorer", external: true },
  { href: "https://wallet.norn.network", label: "Wallet", external: true },
  { href: "https://github.com/augmnt/norn-protocol", label: "GitHub", external: true },
];

export function Header() {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    setMobileOpen(false);
  }, [pathname]);

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 w-full max-w-7xl items-center px-4 sm:px-6 lg:px-8">
        <Link href="/" className="mr-6 flex items-center space-x-2">
          <span className="font-mono text-xl font-bold tracking-[-0.02em] text-foreground">
            norn
          </span>
        </Link>

        <nav className="hidden md:flex items-center gap-1">
          {navItems.map((item) => {
            const isActive =
              !item.external &&
              (pathname === item.href || pathname.startsWith(item.href + "/"));

            if (item.external) {
              return (
                <a
                  key={item.href}
                  href={item.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground hover:bg-accent/50"
                >
                  {item.label}
                  <ExternalLink className="h-3 w-3" />
                </a>
              );
            }

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
                {item.label}
              </Link>
            );
          })}
        </nav>

        <div className="ml-auto flex items-center gap-2">
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
              !item.external &&
              (pathname === item.href || pathname.startsWith(item.href + "/"));

            if (item.external) {
              return (
                <a
                  key={item.href}
                  href={item.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-between rounded-md px-3 py-3 text-sm text-muted-foreground transition-colors hover:text-foreground hover:bg-accent/50 min-h-[44px]"
                >
                  <span className="flex items-center gap-2">
                    {item.label}
                  </span>
                  <ExternalLink className="h-3.5 w-3.5" />
                </a>
              );
            }

            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "flex items-center rounded-md px-3 py-3 text-sm transition-colors min-h-[44px]",
                  isActive
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                )}
              >
                {item.label}
              </Link>
            );
          })}
        </nav>
      </div>
    </header>
  );
}
