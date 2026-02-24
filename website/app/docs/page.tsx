import type { Metadata } from "next";
import Link from "next/link";
import {
  Rocket,
  Layers,
  Wallet,
  AtSign,
  Coins,
  FileCode2,
  Code2,
  Globe,
  Chrome,
  Fingerprint,
  GitPullRequest,
  Blocks,
  type LucideIcon,
} from "lucide-react";

export const metadata: Metadata = {
  title: "Documentation",
};

const sections: {
  title: string;
  description: string;
  href: string;
  icon: LucideIcon;
}[] = [
  {
    title: "Quick Start",
    description: "Install Norn, run a node, and join the devnet in minutes.",
    href: "/docs/quickstart",
    icon: Rocket,
  },
  {
    title: "Architecture",
    description: "Threads, Knots, Weave, Looms -- how the protocol works.",
    href: "/docs/architecture",
    icon: Layers,
  },
  {
    title: "Wallet CLI",
    description:
      "Create wallets, check balances, send transfers, and manage keys.",
    href: "/docs/wallet",
    icon: Wallet,
  },
  {
    title: "NornNames",
    description: "Register, transfer, and manage human-readable names with records.",
    href: "/docs/names",
    icon: AtSign,
  },
  {
    title: "NT-1 Tokens",
    description: "Create and manage custom fungible tokens on-chain.",
    href: "/docs/tokens",
    icon: Coins,
  },
  {
    title: "Loom Contracts",
    description: "Deploy and interact with WebAssembly smart contracts.",
    href: "/docs/looms",
    icon: FileCode2,
  },
  {
    title: "Contract SDK",
    description: "Write loom contracts in Rust with the norn-sdk crate.",
    href: "/docs/sdk/contracts",
    icon: Code2,
  },
  {
    title: "TypeScript SDK",
    description: "Build JavaScript/TypeScript apps with @norn-protocol/sdk.",
    href: "/docs/sdk/typescript",
    icon: Code2,
  },
  {
    title: "Smart Contracts",
    description:
      "Pre-built contracts: crowdfund, governance, escrow, treasury, and more.",
    href: "/docs/contracts/crowdfund",
    icon: Blocks,
  },
  {
    title: "Block Explorer",
    description: "Browse blocks, transactions, accounts, and tokens.",
    href: "/docs/explorer",
    icon: Globe,
  },
  {
    title: "Web Wallet",
    description: "Passkey-secured browser wallet -- no extension needed.",
    href: "/docs/wallet-web",
    icon: Fingerprint,
  },
  {
    title: "Wallet Extension",
    description: "Chrome extension for sending, receiving, and managing NORN.",
    href: "/docs/wallet-extension",
    icon: Chrome,
  },
  {
    title: "Contributing",
    description: "Set up the dev environment and contribute to the protocol.",
    href: "/docs/contributing",
    icon: GitPullRequest,
  },
];

export default function DocsPage() {
  return (
    <div>
      <h1 className="text-3xl font-bold tracking-tight mb-2">Documentation</h1>
      <p className="text-muted-foreground mb-8">
        Everything you need to understand, use, and build on the Norn Protocol.
      </p>
      <div className="grid gap-4 sm:grid-cols-2">
        {sections.map((section) => (
          <Link
            key={section.href}
            href={section.href}
            className="group rounded-lg border bg-card p-5 transition-colors hover:border-norn/30"
          >
            <div className="flex items-start gap-3">
              <section.icon className="h-5 w-5 text-norn mt-0.5 shrink-0" />
              <div>
                <h3 className="font-semibold mb-1 group-hover:text-norn transition-colors">
                  {section.title}
                </h3>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  {section.description}
                </p>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
