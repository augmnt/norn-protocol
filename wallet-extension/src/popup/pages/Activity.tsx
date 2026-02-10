import { useEffect } from "react";
import { Clock } from "lucide-react";
import { useWalletStore } from "@/stores/wallet-store";
import { useActivityStore } from "@/stores/activity-store";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { ActivityRow } from "../components/wallet/ActivityRow";
import { Spinner } from "../components/ui/spinner";

export function Activity() {
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const address = getActiveAddress() ?? "";
  const transactions = useActivityStore((s) => s.transactions);
  const isLoading = useActivityStore((s) => s.isLoading);
  const error = useActivityStore((s) => s.error);
  const fetch = useActivityStore((s) => s.fetch);

  useEffect(() => {
    if (address) fetch(address, 50);
  }, [address, fetch]);

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col overflow-y-auto scrollbar-thin">
        <div className="p-4 pb-2">
          <h2 className="text-lg font-semibold">Activity</h2>
        </div>

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
        ) : (
          <div className="divide-y divide-border px-4 pb-2">
            {transactions.map((tx, i) => (
              <div
                key={i}
                className="animate-slide-in"
                style={{ animationDelay: `${i * 50}ms`, animationFillMode: "backwards" }}
              >
                <ActivityRow tx={tx} currentAddress={address} />
              </div>
            ))}
          </div>
        )}
      </div>

      <BottomNav />
    </div>
  );
}
