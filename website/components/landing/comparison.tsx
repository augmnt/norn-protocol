import { cn } from "@/lib/utils";

const chains = ["Norn", "Bitcoin", "Ethereum", "Solana"] as const;

const rows: { label: string; values: Record<(typeof chains)[number], string> }[] = [
  {
    label: "Bilateral TPS",
    values: { Norn: "Unlimited", Bitcoin: "7", Ethereum: "~30", Solana: "~4,000" },
  },
  {
    label: "Finality",
    values: { Norn: "Instant", Bitcoin: "~60 min", Ethereum: "~13 min", Solana: "~0.4s" },
  },
  {
    label: "Tx Cost",
    values: { Norn: "Free", Bitcoin: "~$1–50", Ethereum: "~$0.50–30", Solana: "~$0.001" },
  },
  {
    label: "Phone Node",
    values: { Norn: "Yes", Bitcoin: "No", Ethereum: "No", Solana: "No" },
  },
  {
    label: "Private by Default",
    values: { Norn: "Yes", Bitcoin: "No", Ethereum: "No", Solana: "No" },
  },
];

export function Comparison() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <h2 className="text-heading mb-10">How Norn compares</h2>
        <div className="rounded-lg border overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/20">
                <th className="px-4 py-3 text-left font-medium text-muted-foreground" />
                {chains.map((chain) => (
                  <th
                    key={chain}
                    className={cn(
                      "px-4 py-3 text-left font-medium",
                      chain === "Norn"
                        ? "text-norn"
                        : "text-muted-foreground"
                    )}
                  >
                    {chain}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.label} className="border-b last:border-0">
                  <td className="px-4 py-3 font-medium whitespace-nowrap">
                    {row.label}
                  </td>
                  {chains.map((chain) => (
                    <td
                      key={chain}
                      className={cn(
                        "px-4 py-3 font-mono text-sm whitespace-nowrap",
                        chain === "Norn"
                          ? "text-foreground"
                          : "text-muted-foreground"
                      )}
                    >
                      {row.values[chain]}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
