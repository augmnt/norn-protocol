"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

const sidebarSections = [
  {
    title: "Getting Started",
    items: [
      { href: "/docs", label: "Overview" },
      { href: "/docs/quickstart", label: "Quick Start" },
      { href: "/docs/architecture", label: "Architecture" },
    ],
  },
  {
    title: "Core Features",
    items: [
      { href: "/docs/wallet", label: "Wallet CLI" },
      { href: "/docs/names", label: "NornNames" },
      { href: "/docs/tokens", label: "NT-1 Tokens" },
      { href: "/docs/looms", label: "Loom Contracts" },
    ],
  },
  {
    title: "SDKs",
    items: [
      { href: "/docs/sdk/contracts", label: "Contract SDK" },
      { href: "/docs/sdk/typescript", label: "TypeScript SDK" },
    ],
  },
  {
    title: "Tools",
    items: [
      { href: "/docs/explorer", label: "Block Explorer" },
      { href: "/docs/wallet-web", label: "Web Wallet" },
      { href: "/docs/wallet-extension", label: "Wallet Extension" },
    ],
  },
  {
    title: "Community",
    items: [{ href: "/docs/contributing", label: "Contributing" }],
  },
];

export function DocsSidebar() {
  const pathname = usePathname();

  return (
    <aside className="w-64 shrink-0">
      <nav className="sticky top-20 space-y-6 pr-4">
        {sidebarSections.map((section) => (
          <div key={section.title}>
            <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {section.title}
            </h4>
            <ul className="space-y-0.5">
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className={cn(
                        "block rounded-md px-3 py-1.5 text-sm transition-colors",
                        isActive
                          ? "bg-accent text-accent-foreground font-medium"
                          : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                      )}
                    >
                      {item.label}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </nav>
    </aside>
  );
}

export function MobileDocsSidebar() {
  const pathname = usePathname();

  return (
    <details className="group mb-6 rounded-lg border md:hidden">
      <summary className="flex cursor-pointer items-center justify-between px-4 py-3 text-sm font-medium">
        Documentation
        <svg
          className="h-4 w-4 transition-transform group-open:rotate-180"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </summary>
      <nav className="border-t px-4 py-3 space-y-4">
        {sidebarSections.map((section) => (
          <div key={section.title}>
            <h4 className="mb-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {section.title}
            </h4>
            <ul className="space-y-0.5">
              {section.items.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      className={cn(
                        "block rounded-md px-3 py-1.5 text-sm transition-colors",
                        isActive
                          ? "bg-accent text-accent-foreground font-medium"
                          : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                      )}
                    >
                      {item.label}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </nav>
    </details>
  );
}
