import { useEffect, useState, useCallback, useRef } from "react";
import { ArrowUpRight, QrCode, Copy, AtSign, Droplets, WifiOff } from "lucide-react";
import { toast } from "sonner";
import { subscribeTransfers, type Subscription } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { useNetworkStore } from "@/stores/network-store";
import { useActivityStore } from "@/stores/activity-store";
import { rpc } from "@/lib/rpc";
import { BALANCE_POLL_INTERVAL } from "@/lib/config";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { BalanceCard } from "../components/wallet/BalanceCard";
import { AccountPill } from "../components/wallet/AccountPill";
import { ActivityRow } from "../components/wallet/ActivityRow";
import { Button } from "../components/ui/button";
import { Spinner } from "../components/ui/spinner";

export function Dashboard() {
  const [balance, setBalance] = useState<string>("0");
  const [balanceLoading, setBalanceLoading] = useState(true);
  const [connected, setConnected] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);

  const subRef = useRef<Subscription | null>(null);

  const accounts = useWalletStore((s) => s.accounts);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);
  const navigate = useNavigationStore((s) => s.navigate);
  const wsUrl = useNetworkStore((s) => s.wsUrl);
  const transactions = useActivityStore((s) => s.transactions);
  const fetchActivity = useActivityStore((s) => s.fetch);

  const activeAccount = accounts.find((a) => a.id === activeAccountId);
  const address = activeAccount?.address ?? "";

  const fetchBalance = useCallback(async () => {
    if (!address) return;
    try {
      const bal = await rpc.getBalance(address);
      setBalance(bal);
      setBalanceLoading(false);
      setConnected(true);
      setFetchError(null);
    } catch (err) {
      setBalanceLoading(false);
      setConnected(false);
      setFetchError(
        err instanceof Error ? err.message : "Failed to connect to node",
      );
    }
  }, [address]);

  // Polling + initial fetch
  useEffect(() => {
    fetchBalance();
    fetchActivity(address, 5);

    const interval = setInterval(fetchBalance, BALANCE_POLL_INTERVAL);
    return () => clearInterval(interval);
  }, [address, fetchBalance, fetchActivity]);

  // WebSocket subscription for instant balance updates
  useEffect(() => {
    if (!address || !wsUrl) return;

    let cancelled = false;

    const sub = subscribeTransfers(
      {
        url: wsUrl,
        onError: () => {
          // WS errors are non-fatal — polling continues as fallback
        },
      },
      () => {
        if (!cancelled) {
          // Re-fetch balance + activity immediately on transfer event
          fetchBalance();
          fetchActivity(address, 5);
        }
      },
      address,
    );

    subRef.current = sub;

    return () => {
      cancelled = true;
      sub.unsubscribe();
      subRef.current = null;
    };
  }, [address, wsUrl, fetchBalance, fetchActivity]);

  const [faucetLoading, setFaucetLoading] = useState(false);

  const handleCopyAddress = () => {
    navigator.clipboard.writeText(address);
    toast.success("Address copied");
  };

  const handleFaucet = async () => {
    if (!address || faucetLoading) return;
    setFaucetLoading(true);
    try {
      const result = await rpc.faucet(address);
      if (!result.success) {
        toast.error(result.reason ?? "Faucet request failed");
        return;
      }
      toast.success("Faucet tokens received!");
      fetchBalance();
      fetchActivity(address, 5);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Faucet request failed",
      );
    } finally {
      setFaucetLoading(false);
    }
  };

  const actionButtons = [
    { label: "Send", icon: ArrowUpRight, onClick: () => navigate("send") },
    { label: "Receive", icon: QrCode, onClick: () => navigate("receive") },
    { label: "Copy", icon: Copy, onClick: handleCopyAddress },
    { label: "Name", icon: AtSign, onClick: () => navigate("register-name") },
  ];

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col overflow-y-auto scrollbar-thin">
        <div className="flex flex-col items-center gap-3 px-4 pt-3">
          {activeAccount && (
            <AccountPill
              name={activeAccount.name}
              address={activeAccount.address}
            />
          )}

          {balanceLoading ? (
            <div className="flex items-center justify-center py-8">
              <Spinner size="lg" />
            </div>
          ) : fetchError ? (
            <div className="flex flex-col items-center gap-2 py-6 animate-fade-in">
              <WifiOff className="h-6 w-6 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                Cannot reach node
              </p>
              <p className="max-w-[260px] text-center text-xs text-destructive">
                {fetchError}
              </p>
              <Button variant="outline" size="sm" onClick={fetchBalance}>
                Retry
              </Button>
            </div>
          ) : (
            <BalanceCard balance={balance} isLive={connected} />
          )}

          <div className="flex items-center gap-4">
            {actionButtons.map(({ label, icon: Icon, onClick }) => (
              <button
                key={label}
                onClick={onClick}
                className="flex flex-col items-center gap-1"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95">
                  <Icon className="h-4 w-4" />
                </div>
                <span className="text-xs text-muted-foreground">{label}</span>
              </button>
            ))}

            <button
              onClick={handleFaucet}
              disabled={faucetLoading}
              className="flex flex-col items-center gap-1"
            >
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-norn/20 text-norn transition-all duration-150 hover:bg-norn/30 active:scale-95">
                {faucetLoading ? (
                  <Spinner size="sm" />
                ) : (
                  <Droplets className="h-4 w-4" />
                )}
              </div>
              <span className="text-xs text-muted-foreground">Faucet</span>
            </button>
          </div>
        </div>

        <div className="mt-4 flex-1 px-4 pb-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">Recent Activity</h3>
            {transactions.length > 0 && (
              <Button
                variant="link"
                size="sm"
                className="h-auto p-0 text-xs text-norn"
                onClick={() => navigate("activity")}
              >
                View All
              </Button>
            )}
          </div>

          {transactions.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-8 text-muted-foreground animate-fade-in">
              <p className="text-sm">No recent activity</p>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {transactions.slice(0, 5).map((tx, i) => (
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
      </div>

      <BottomNav />
    </div>
  );
}
