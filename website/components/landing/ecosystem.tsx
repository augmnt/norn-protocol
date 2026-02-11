import { Globe, Terminal, Code2, Fingerprint } from "lucide-react";

const items = [
  {
    icon: Globe,
    title: "Explorer",
    description: "Browse blocks, transactions, and accounts in real time.",
    href: "https://explorer.norn.network",
    linkLabel: "explorer.norn.network",
  },
  {
    icon: Fingerprint,
    title: "Web Wallet",
    description: "Passkey-secured browser wallet — no extension needed.",
    href: "https://wallet.norn.network",
    linkLabel: "wallet.norn.network",
  },
  {
    icon: Terminal,
    title: "Wallet CLI",
    description: "Create wallets, send NORN, register names from the terminal.",
    href: "/docs/wallet",
    linkLabel: "Documentation",
    internal: true,
  },
  {
    icon: Code2,
    title: "TypeScript SDK",
    description: "Wallet primitives, RPC client, and WebSocket subscriptions.",
    href: "/docs/sdk/typescript",
    linkLabel: "Documentation",
    internal: true,
  },
];

export function Ecosystem() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <h2 className="text-heading mb-10">Ecosystem</h2>
        <div className="grid gap-8 sm:grid-cols-2">
          {items.map((item) => (
            <a
              key={item.title}
              href={item.href}
              target={item.internal ? undefined : "_blank"}
              rel={item.internal ? undefined : "noopener noreferrer"}
              className="group flex items-start gap-4"
            >
              <item.icon className="h-5 w-5 text-norn mt-0.5 shrink-0" />
              <div>
                <h3 className="font-semibold mb-1">{item.title}</h3>
                <p className="text-sm text-muted-foreground leading-relaxed mb-2">
                  {item.description}
                </p>
                <span className="text-sm font-mono text-norn group-hover:underline">
                  {item.linkLabel}
                </span>
              </div>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
