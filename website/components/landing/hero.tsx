import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ArrowRight, ExternalLink } from "lucide-react";

export function Hero() {
  return (
    <section className="relative">
      <div className="mx-auto max-w-7xl px-4 py-24 sm:px-6 sm:py-32 lg:px-8 lg:py-40">
        <div className="max-w-3xl">
          <p className="font-mono text-sm text-norn mb-6">norn protocol</p>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl leading-[1.1]">
            The chain is a courtroom,
            <br />
            <span className="text-muted-foreground">not a bank.</span>
          </h1>
          <p className="mt-6 text-lg text-muted-foreground max-w-2xl leading-relaxed">
            Norn is a blockchain where users transact directly with cryptographic
            signatures. The network only intervenes when there&rsquo;s a dispute.
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
      </div>
    </section>
  );
}
