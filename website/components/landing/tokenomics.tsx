const allocations = [
  { category: "Validator Rewards", pct: "30%" },
  { category: "Ecosystem Development", pct: "20%" },
  { category: "Founder & Core Team", pct: "15%" },
  { category: "Community & Grants", pct: "15%" },
  { category: "Treasury Reserve", pct: "10%" },
  { category: "Initial Liquidity", pct: "5%" },
  { category: "Testnet Participants", pct: "5%" },
];

export function Tokenomics() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <h2 className="text-heading mb-2">Token Economics</h2>
        <p className="text-muted-foreground mb-10">
          Fixed supply of{" "}
          <span className="font-mono text-foreground">1,000,000,000 NORN</span>.
        </p>
        <div className="max-w-xl space-y-2">
          {allocations.map((row) => (
            <div
              key={row.category}
              className="flex items-center justify-between py-1"
            >
              <span className="text-sm">{row.category}</span>
              <span className="font-mono text-sm text-norn">{row.pct}</span>
            </div>
          ))}
        </div>
        <p className="mt-8 text-sm text-muted-foreground max-w-xl">
          Protocol actions burn NORN permanently: names (1), tokens (10),
          contracts (50).
        </p>
      </div>
    </section>
  );
}
