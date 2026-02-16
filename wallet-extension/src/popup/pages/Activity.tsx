import { useEffect, useState } from "react";
import { Clock } from "lucide-react";
import { useWalletStore } from "@/stores/wallet-store";
import { useActivityStore } from "@/stores/activity-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { ActivityRow } from "../components/wallet/ActivityRow";
import { Spinner } from "../components/ui/spinner";
import { cn } from "@/lib/utils";

type Filter = "all" | "sent" | "received";

const FILTERS: { value: Filter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "sent", label: "Sent" },
  { value: "received", label: "Received" },
];

export function Activity() {
  const [filter, setFilter] = useState<Filter>("all");
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const address = getActiveAddress() ?? "";
  const transactions = useActivityStore((s) => s.transactions);
  const isLoading = useActivityStore((s) => s.isLoading);
  const error = useActivityStore((s) => s.error);
  const fetch = useActivityStore((s) => s.fetch);
  const navigate = useNavigationStore((s) => s.navigate);

  useEffect(() => {
    if (address) fetch(address, 50);
  }, [address, fetch]);

  const filtered = transactions.filter((tx) => {
    if (filter === "all") return true;
    const isSent = tx.from?.toLowerCase() === address.toLowerCase();
    return filter === "sent" ? isSent : !isSent;
  });

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col overflow-y-auto scrollbar-thin">
        <div className="p-4 pb-2">
          <h2 className="text-lg font-semibold">Activity</h2>
        </div>

        {/* Filter chips */}
        {transactions.length > 0 && (
          <div className="flex gap-1.5 px-4 pb-2">
            {FILTERS.map(({ value, label }) => (
              <button
                key={value}
                onClick={() => setFilter(value)}
                className={cn(
                  "rounded-full px-3 py-1 text-xs font-medium transition-all duration-150",
                  filter === value
                    ? "bg-norn text-white"
                    : "bg-secondary text-secondary-foreground hover:bg-muted",
                )}
              >
                {label}
              </button>
            ))}
          </div>
        )}

        {isLoading ? (
          <div className="flex flex-1 items-center justify-center">
            <Spinner size="lg" />
          </div>
        ) : error ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground animate-fade-in">
            <p className="text-sm">{error}</p>
          </div>
        ) : transactions.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground animate-fade-in">
            <Clock className="h-8 w-8" />
            <p className="text-sm">No transactions yet</p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground animate-fade-in">
            <p className="text-sm">No {filter} transactions</p>
          </div>
        ) : (
          <div className="divide-y divide-border px-4 pb-2">
            {filtered.map((tx, i) => (
              <div
                key={tx.knot_id || i}
                className="animate-slide-in"
                style={{ animationDelay: `${i * 50}ms`, animationFillMode: "backwards" }}
              >
                <ActivityRow
                  tx={tx}
                  currentAddress={address}
                  onClick={() => navigate("transaction-detail", { tx })}
                />
              </div>
            ))}
          </div>
        )}
      </div>

      <BottomNav />
    </div>
  );
}
