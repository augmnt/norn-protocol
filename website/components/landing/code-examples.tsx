"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import { Check, Copy } from "lucide-react";

function TerminalBlock({
  title,
  commands,
}: {
  title: string;
  commands: string[];
}) {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(commands.join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group relative rounded-lg border border-border bg-[hsl(240,10%,6%)]">
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <span className="text-xs font-mono text-muted-foreground">{title}</span>
        <button
          onClick={handleCopy}
          className="rounded-md p-1.5 text-muted-foreground transition-colors hover:text-foreground opacity-0 group-hover:opacity-100"
          aria-label="Copy commands"
        >
          {copied ? (
            <Check className="h-3.5 w-3.5" />
          ) : (
            <Copy className="h-3.5 w-3.5" />
          )}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 text-sm leading-relaxed">
        <code>
          {commands.map((cmd, i) => (
            <React.Fragment key={i}>
              <span className="text-norn select-none">$ </span>
              <span className="text-muted-foreground">{cmd}</span>
              {i < commands.length - 1 && "\n"}
            </React.Fragment>
          ))}
        </code>
      </pre>
    </div>
  );
}

export function CodeExamples() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <h2 className="text-heading mb-2">Get started in 30 seconds</h2>
        <p className="text-muted-foreground mb-10">
          Everything runs from a single binary.
        </p>
        <div className="max-w-2xl space-y-4">
          <TerminalBlock
            title="Run a Node"
            commands={["cargo install norn-node && norn run --dev"]}
          />
          <TerminalBlock
            title="Setup Wallet"
            commands={[
              "norn wallet create",
              "norn wallet faucet",
              "norn wallet balance",
            ]}
          />
          <TerminalBlock
            title="Send NORN"
            commands={["norn wallet transfer --to alice --amount 10"]}
          />
        </div>
      </div>
    </section>
  );
}
