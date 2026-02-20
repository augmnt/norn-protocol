const stats = [
  { value: "$0", label: "Transfer fees" },
  { value: "~3s", label: "Block finality" },
  { value: "~2 GB", label: "RAM to start a full node" },
  { value: "3s", label: "Block time" },
  { value: "SMT", label: "State verification" },
  { value: "Wasm", label: "Smart contracts" },
];

export function Stats() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
        <div className="grid grid-cols-2 gap-8 sm:grid-cols-3">
          {stats.map((stat) => (
            <div key={stat.label}>
              <p className="font-mono text-2xl sm:text-3xl font-semibold tracking-tight">
                {stat.value}
              </p>
              <p className="mt-1 text-sm text-muted-foreground">{stat.label}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
