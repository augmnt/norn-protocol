const steps = [
  {
    number: "01",
    title: "Threads",
    description:
      "Each user owns a personal state chain. Your balance, your names, your history — all yours.",
  },
  {
    number: "02",
    title: "Knots",
    description:
      "Transact by co-signing state transitions with your counterparty. Instant, free, private.",
  },
  {
    number: "03",
    title: "Weave",
    description:
      "Commit hashes to the anchor chain periodically. Fraud proofs punish cheating.",
  },
];

export function HowItWorks() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <h2 className="text-heading mb-12">How it works</h2>
        <div className="grid gap-12 sm:grid-cols-3">
          {steps.map((step) => (
            <div key={step.number}>
              <p className="font-mono text-sm text-norn mb-3">{step.number}</p>
              <h3 className="text-xl font-semibold mb-2">{step.title}</h3>
              <p className="text-muted-foreground leading-relaxed">
                {step.description}
              </p>
            </div>
          ))}
        </div>
        <p className="mt-12 text-muted-foreground max-w-2xl">
          Need smart contracts?{" "}
          <span className="text-foreground">
            Looms run WebAssembly off-chain with on-chain guarantees.
          </span>
        </p>
      </div>
    </section>
  );
}
