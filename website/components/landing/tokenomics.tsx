import { Flame } from "lucide-react";

const allocations = [
  { category: "Validator Rewards", pct: "30%", amount: "300,000,000", note: "Block rewards over 10+ years" },
  { category: "Ecosystem Development", pct: "20%", amount: "200,000,000", note: "Controlled release over 5 years" },
  { category: "Founder & Core Team", pct: "15%", amount: "150,000,000", note: "4-year linear, 1-year cliff" },
  { category: "Community & Grants", pct: "15%", amount: "150,000,000", note: "Governance-controlled" },
  { category: "Treasury Reserve", pct: "10%", amount: "100,000,000", note: "DAO-governed after decentralization" },
  { category: "Initial Liquidity", pct: "5%", amount: "50,000,000", note: "Available at launch" },
  { category: "Testnet Participants", pct: "5%", amount: "50,000,000", note: "Airdrop at mainnet launch" },
];

const burns = [
  { action: "NornName Registration", cost: "1 NORN" },
  { action: "NT-1 Token Creation", cost: "10 NORN" },
  { action: "Loom Deployment", cost: "50 NORN" },
];

export function Tokenomics() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mb-12">
          <h2 className="text-heading">Token Economics</h2>
          <p className="mt-2 text-muted-foreground">
            Fixed maximum supply of{" "}
            <span className="font-mono text-foreground">1,000,000,000 NORN</span>,
            enforced at the protocol level.
          </p>
        </div>

        <div className="grid gap-8 lg:grid-cols-[2fr_1fr]">
          {/* Allocation table */}
          <div className="rounded-lg border overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/30">
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Category</th>
                  <th className="px-4 py-3 text-right font-medium text-muted-foreground">%</th>
                  <th className="px-4 py-3 text-right font-medium text-muted-foreground hidden sm:table-cell">Amount</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground hidden md:table-cell">Vesting</th>
                </tr>
              </thead>
              <tbody>
                {allocations.map((row) => (
                  <tr key={row.category} className="border-b last:border-0">
                    <td className="px-4 py-3 font-medium">{row.category}</td>
                    <td className="px-4 py-3 text-right font-mono text-norn">{row.pct}</td>
                    <td className="px-4 py-3 text-right font-mono text-muted-foreground hidden sm:table-cell">{row.amount}</td>
                    <td className="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{row.note}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Deflationary burns */}
          <div className="rounded-lg border p-6">
            <div className="flex items-center gap-2 mb-4">
              <Flame className="h-4 w-4 text-orange-500" />
              <h3 className="font-semibold text-sm">Deflationary Burns</h3>
            </div>
            <p className="text-xs text-muted-foreground mb-4">
              Protocol actions permanently burn NORN, reducing circulating supply.
            </p>
            <div className="space-y-3">
              {burns.map((burn) => (
                <div
                  key={burn.action}
                  className="flex items-center justify-between rounded-md bg-muted/30 px-3 py-2"
                >
                  <span className="text-sm">{burn.action}</span>
                  <span className="font-mono text-sm text-orange-500">
                    {burn.cost}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
