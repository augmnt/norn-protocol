import { cn } from "@/lib/utils";

const chains = ["Norn", "Bitcoin", "Ethereum", "Solana"] as const;

const rows: { label: string; values: Record<(typeof chains)[number], string> }[] = [
  {
    label: "Finality",
    values: { Norn: "~3s", Bitcoin: "~60 min", Ethereum: "~13 min", Solana: "~0.4s" },
  },
  {
    label: "Transfer Cost",
    values: { Norn: "Free", Bitcoin: "~$1–50", Ethereum: "~$0.50–30", Solana: "~$0.001" },
  },
  {
    label: "Lightweight Node",
    values: { Norn: "Yes", Bitcoin: "No", Ethereum: "No", Solana: "No" },
  },
  {
    label: "State Verification",
    values: { Norn: "Merkle proofs", Bitcoin: "Full node", Ethereum: "Full node", Solana: "Full node" },
  },
  {
    label: "Smart Contracts",
    values: { Norn: "Wasm", Bitcoin: "Script", Ethereum: "EVM", Solana: "SVM" },
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
