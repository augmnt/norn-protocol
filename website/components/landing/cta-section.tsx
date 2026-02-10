import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Terminal, Code2, GitPullRequest } from "lucide-react";

const ctas = [
  {
    icon: Terminal,
    title: "Run a Node",
    description: "Join the devnet with a single command and start syncing blocks.",
    href: "/docs/quickstart",
    label: "Quick Start",
  },
  {
    icon: Code2,
    title: "Build a Contract",
    description: "Write WebAssembly smart contracts using the Rust SDK.",
    href: "/docs/looms",
    label: "Loom Contracts",
  },
  {
    icon: GitPullRequest,
    title: "Contribute",
    description: "Help build the protocol. PRs, issues, and feedback welcome.",
    href: "/docs/contributing",
    label: "Contributing Guide",
  },
];

export function CtaSection() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mb-12 text-center">
          <h2 className="text-heading">Get Involved</h2>
          <p className="mt-2 text-muted-foreground">
            Norn is open source. Everyone can participate.
          </p>
        </div>
        <div className="grid gap-6 sm:grid-cols-3">
          {ctas.map((cta) => (
            <div
              key={cta.title}
              className="flex flex-col items-center rounded-lg border bg-card p-8 text-center"
            >
              <cta.icon className="h-6 w-6 text-norn mb-4" />
              <h3 className="font-semibold mb-2">{cta.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed mb-6">
                {cta.description}
              </p>
              <Button asChild variant="outline" size="sm" className="mt-auto">
                <Link href={cta.href}>{cta.label}</Link>
              </Button>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
