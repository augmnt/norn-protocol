import { Globe, Wallet, Code2 } from "lucide-react";

const ecosystemItems = [
  {
    icon: Globe,
    title: "Block Explorer",
    description:
      "Browse blocks, transactions, accounts, tokens, and smart contracts in real time.",
    href: "https://explorer.norn.network",
    linkLabel: "explorer.norn.network",
  },
  {
    icon: Wallet,
    title: "Browser Wallet",
    description:
      "Chrome extension for sending NORN, managing accounts, and registering NornNames.",
    href: "/docs/wallet-extension",
    linkLabel: "Installation guide",
    internal: true,
  },
  {
    icon: Code2,
    title: "TypeScript SDK",
    description:
      "Wallet primitives, transaction builders, RPC client, and WebSocket subscriptions.",
    href: "/docs/sdk/typescript",
    linkLabel: "SDK documentation",
    internal: true,
  },
];

export function Ecosystem() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mb-12">
          <h2 className="text-heading">Ecosystem</h2>
          <p className="mt-2 text-muted-foreground">
            Tools for interacting with the Norn network.
          </p>
        </div>
        <div className="grid gap-6 sm:grid-cols-3">
          {ecosystemItems.map((item) => (
            <a
              key={item.title}
              href={item.href}
              target={item.internal ? undefined : "_blank"}
              rel={item.internal ? undefined : "noopener noreferrer"}
              className="group rounded-lg border bg-card p-6 transition-colors hover:border-norn/30"
            >
              <item.icon className="h-5 w-5 text-norn mb-4" />
              <h3 className="font-semibold mb-2">{item.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed mb-4">
                {item.description}
              </p>
              <span className="text-sm font-mono text-norn group-hover:underline">
                {item.linkLabel}
              </span>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
