import {
  Zap,
  Smartphone,
  Ban,
  Eye,
  Timer,
  ShieldCheck,
} from "lucide-react";

const features = [
  {
    icon: Zap,
    title: "Unlimited bilateral throughput",
    description:
      "Two parties can exchange value as fast as they can sign messages. No block size limit, no gas auction, no mempool congestion.",
  },
  {
    icon: Smartphone,
    title: "Phone-runnable full nodes",
    description:
      "The anchor chain processes only commitments and fraud proofs, keeping on-chain state minimal. A full node runs on a modern smartphone.",
  },
  {
    icon: Ban,
    title: "Zero-fee P2P transfers",
    description:
      "Bilateral transactions incur no on-chain fee. Only periodic commitments to the anchor chain carry a small dynamic fee.",
  },
  {
    icon: Eye,
    title: "Privacy by default",
    description:
      "The chain never sees transaction details, balances, or counterparties. It sees only cryptographic commitments.",
  },
  {
    icon: Timer,
    title: "Instant bilateral finality",
    description:
      "A transaction is final the moment both parties sign. No confirmation time, no block wait.",
  },
  {
    icon: ShieldCheck,
    title: "Fraud-proof security",
    description:
      "Cheating is detectable and punishable through economic penalties. Honest behavior is the Nash equilibrium.",
  },
];

export function Features() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mb-12">
          <h2 className="text-heading">Key Properties</h2>
          <p className="mt-2 text-muted-foreground">
            What makes Norn fundamentally different.
          </p>
        </div>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {features.map((feature) => (
            <div
              key={feature.title}
              className="group rounded-lg border bg-card p-6 transition-colors hover:border-norn/30"
            >
              <feature.icon className="h-5 w-5 text-norn mb-4" />
              <h3 className="font-semibold mb-2">{feature.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
