import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ArrowRight, ExternalLink } from "lucide-react";

const heroArt = `     │           │           │
     │           │           │
     │           │           │
─────┼─────      │      ─────┼─────
     │      ─────┼─────      │
═════╪═════      │      ═════╪═════
     │      ═════╪═════      │
─────┼─────      │      ─────┼─────
     │      ─────┼─────      │
═════╪═════      │      ═════╪═════
      ╲     ═════╪═════     ╱
       ╲    ─────┼─────    ╱
════════╪════════╪════════╪════════
         ╲  ═════╪═════  ╱
──────────╲──────┼──────╱──────────
           ╲     │     ╱
════════════╪════╪════╪════════════
             ╲   │   ╱
──────────────╲──┼──╱──────────────
               ╲ │ ╱
                ╲│╱
                 ●
                ╱│╲
               ╱ │ ╲
──────────────╱──┼──╲──────────────
             ╱   │   ╲
════════════╪════╪════╪════════════
           ╱     │     ╲
──────────╱──────┼──────╲──────────
         ╱  ═════╪═════  ╲
════════╪════════╪════════╪════════
       ╱    ─────┼─────    ╲
      ╱     ═════╪═════     ╲
═════╪═════      │      ═════╪═════
     │      ─────┼─────      │
─────┼─────      │      ─────┼─────
     │      ═════╪═════      │
═════╪═════      │      ═════╪═════
     │      ─────┼─────      │
─────┼─────      │      ─────┼─────
     │           │           │
     │           │           │
     │           │           │`;

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      <div className="mx-auto max-w-7xl px-4 py-24 sm:px-6 sm:py-32 lg:px-8 lg:py-40">
        <div className="flex items-start gap-12 lg:gap-20">
          <div className="max-w-2xl flex-1">
            <p className="font-mono text-sm text-norn mb-6">norn protocol</p>
            <h1 className="text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl leading-[1.1]">
              You hold
              <br />
              <span className="text-muted-foreground">the thread.</span>
            </h1>
            <p className="mt-6 text-lg text-muted-foreground max-w-2xl leading-relaxed">
              Norn is a blockchain where users own their state through personal
              cryptographic chains. The network validates transitions and
              guarantees correctness.
            </p>
            <div className="mt-10 flex flex-wrap gap-4">
              <Button asChild size="lg" variant="norn">
                <Link href="/docs/quickstart">
                  Read the docs
                  <ArrowRight className="ml-1 h-4 w-4" />
                </Link>
              </Button>
              <Button asChild size="lg" variant="outline">
                <a
                  href="https://explorer.norn.network"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Explore the network
                  <ExternalLink className="ml-1 h-4 w-4" />
                </a>
              </Button>
            </div>
          </div>
          <div className="hidden lg:block flex-shrink-0" aria-hidden="true">
            <pre className="font-mono text-[9px] leading-[1.2] text-muted-foreground/20 select-none">
              {heroArt}
            </pre>
          </div>
        </div>
      </div>
    </section>
  );
}
