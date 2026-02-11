import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ArrowRight } from "lucide-react";

export function CtaSection() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8 text-center">
        <p className="text-lg text-muted-foreground max-w-xl mx-auto">
          Norn is open source. Read the docs, run a node, build something.
        </p>
        <div className="mt-8">
          <Button asChild size="lg" variant="outline">
            <Link href="/docs/quickstart">
              Documentation
              <ArrowRight className="ml-1 h-4 w-4" />
            </Link>
          </Button>
        </div>
      </div>
    </section>
  );
}
