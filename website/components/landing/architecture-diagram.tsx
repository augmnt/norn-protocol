export function ArchitectureDiagram() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mb-12">
          <h2 className="text-heading">Architecture</h2>
          <p className="mt-2 text-muted-foreground max-w-2xl">
            Users transact directly using cryptographic signatures. The chain
            intervenes only when there is a dispute, processing fraud proofs
            rather than transactions.
          </p>
        </div>

        {/* Architecture visual */}
        <div className="rounded-lg border bg-card p-6 sm:p-8 lg:p-10">
          <div className="grid gap-8 lg:grid-cols-[1fr_auto_1fr]">
            {/* Threads (left) */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                Off-Chain
              </h3>
              <div className="space-y-3">
                <ArchNode
                  label="Threads"
                  description="Personal state chains for each user"
                  color="norn"
                />
                <ArchNode
                  label="Knots"
                  description="Bilateral agreements tying threads together"
                  color="norn"
                />
                <ArchNode
                  label="Looms"
                  description="WebAssembly smart contracts executing off-chain"
                  color="norn"
                />
              </div>
            </div>

            {/* Connector */}
            <div className="hidden lg:flex flex-col items-center justify-center gap-2">
              <div className="h-full w-px border-l border-dashed border-muted-foreground/30" />
              <span className="text-[10px] font-mono text-muted-foreground px-2 whitespace-nowrap">
                commitments &amp; fraud proofs
              </span>
              <div className="h-full w-px border-l border-dashed border-muted-foreground/30" />
            </div>

            {/* Visible on mobile */}
            <div className="flex items-center justify-center lg:hidden">
              <div className="w-full border-t border-dashed border-muted-foreground/30 relative">
                <span className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-card px-2 text-[10px] font-mono text-muted-foreground">
                  commitments &amp; fraud proofs
                </span>
              </div>
            </div>

            {/* Weave (right) */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                On-Chain (The Weave)
              </h3>
              <div className="space-y-3">
                <ArchNode
                  label="Weave"
                  description="Minimal HotStuff BFT anchor chain"
                  color="muted"
                />
                <ArchNode
                  label="Spindles"
                  description="Watchtower services monitoring for fraud"
                  color="muted"
                />
                <ArchNode
                  label="Relays"
                  description="P2P message delivery between threads"
                  color="muted"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Component breakdown */}
        <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[
            {
              name: "Threads",
              desc: "Personal state chains -- each user maintains their own signed history of state transitions.",
            },
            {
              name: "Knots",
              desc: "Atomic state transitions -- bilateral or multilateral agreements signed by all participants.",
            },
            {
              name: "Weave",
              desc: "The anchor chain -- a minimal HotStuff BFT blockchain for commitments and fraud proofs.",
            },
            {
              name: "Looms",
              desc: "Off-chain smart contracts -- WebAssembly programs with on-chain fraud proof guarantees.",
            },
            {
              name: "Spindles",
              desc: "Watchtower services -- monitor the Weave and submit fraud proofs when misbehavior is detected.",
            },
            {
              name: "Relays",
              desc: "P2P message buffers -- asynchronous message delivery between Threads via libp2p.",
            },
          ].map((item) => (
            <div key={item.name} className="rounded-md border p-4">
              <h4 className="font-mono text-sm font-semibold text-norn">
                {item.name}
              </h4>
              <p className="mt-1 text-xs text-muted-foreground leading-relaxed">
                {item.desc}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function ArchNode({
  label,
  description,
  color,
}: {
  label: string;
  description: string;
  color: "norn" | "muted";
}) {
  return (
    <div
      className={`rounded-md border p-4 ${
        color === "norn"
          ? "border-norn/20 bg-norn/5"
          : "border-muted bg-muted/30"
      }`}
    >
      <div className="font-mono text-sm font-semibold">{label}</div>
      <div className="text-xs text-muted-foreground mt-1">{description}</div>
    </div>
  );
}
