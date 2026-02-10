import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ArrowRight, ExternalLink } from "lucide-react";

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      {/* Subtle grid background */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,hsl(var(--border)/0.3)_1px,transparent_1px),linear-gradient(to_bottom,hsl(var(--border)/0.3)_1px,transparent_1px)] bg-[size:4rem_4rem]" />
      <div className="absolute inset-0 bg-gradient-to-b from-background via-background/95 to-background" />

      <div className="relative mx-auto max-w-7xl px-4 py-24 sm:px-6 sm:py-32 lg:px-8 lg:py-40">
        <div className="max-w-3xl">
          <p className="font-mono text-sm text-norn mb-4">norn protocol</p>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl">
            Your thread. Your fate.{" "}
            <span className="text-muted-foreground">The chain just watches.</span>
          </h1>
          <p className="mt-6 text-lg text-muted-foreground max-w-2xl leading-relaxed">
            A radically minimal blockchain where users transact directly with
            cryptographic signatures. The chain is a courtroom, not a bank
            &mdash; it intervenes only when there is a dispute.
          </p>
          <div className="mt-10 flex flex-wrap gap-4">
            <Button asChild size="lg" variant="norn">
              <Link href="/docs/quickstart">
                Get Started
                <ArrowRight className="ml-1 h-4 w-4" />
              </Link>
            </Button>
            <Button asChild size="lg" variant="outline">
              <a
                href="https://explorer.norn.network"
                target="_blank"
                rel="noopener noreferrer"
              >
                Explore the Network
                <ExternalLink className="ml-1 h-4 w-4" />
              </a>
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
}
