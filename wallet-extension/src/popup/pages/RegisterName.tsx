import { useState, useEffect } from "react";
import { AtSign, Check } from "lucide-react";
import { toast } from "sonner";
import { buildNameRegistration } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { formatNorn } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";
import type { NameInfo } from "@/types";

/** Name registration fee: 1 NORN */
const NAME_FEE = "1000000000000";

export function RegisterName() {
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [ownedNames, setOwnedNames] = useState<NameInfo[]>([]);
  const [namesLoading, setNamesLoading] = useState(true);
  const [balance, setBalance] = useState("0");
  const [balanceLoaded, setBalanceLoaded] = useState(false);

  const activeWallet = useWalletStore((s) => s.activeWallet);
  const getActiveAddress = useWalletStore((s) => s.getActiveAddress);
  const reset = useNavigationStore((s) => s.reset);
  const address = getActiveAddress() ?? "";

  useEffect(() => {
    if (!address) return;

    // Fetch balance and names independently so one failure doesn't block the other
    rpc
      .getBalance(address)
      .then((bal) => {
        setBalance(bal);
        setBalanceLoaded(true);
      })
      .catch(() => {
        setBalanceLoaded(true);
      });

    rpc
      .listNames(address)
      .then((names) => {
        setOwnedNames(names ?? []);
        setNamesLoading(false);
      })
      .catch(() => {
        setNamesLoading(false);
      });
  }, [address]);

  const nameValid = name.length === 0 || isValidName(name);
  const hasSufficientBalance = BigInt(balance) >= BigInt(NAME_FEE);

  const isValid =
    name.length > 0 &&
    isValidName(name) &&
    hasSufficientBalance &&
    balanceLoaded &&
    !loading;

  const handleRegister = async () => {
    if (!activeWallet || !isValid) return;

    setLoading(true);
    try {
      const knotHex = buildNameRegistration(activeWallet, name);
      const result = await rpc.registerName(name, activeWallet.addressHex, knotHex);
      if (!result.success) {
        toast.error(result.reason ?? "Registration failed");
        return;
      }
      toast.success(`Name "${name}" registered`);
      reset("dashboard");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Registration failed",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Register Name</h2>
          <p className="text-sm text-muted-foreground">
            Register a human-readable name for your address. Costs 1 NORN.
          </p>
        </div>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value.trim().toLowerCase())}
              placeholder="myname"
              maxLength={32}
            />
            {!nameValid && (
              <p className="animate-fade-in text-xs text-destructive">
                3-32 characters, lowercase letters, numbers, and hyphens
                only. Cannot start or end with a hyphen.
              </p>
            )}
          </div>

          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Fee</span>
            <span className="font-mono text-sm font-medium tabular-nums">1 NORN</span>
          </div>

          <div className="flex items-center justify-between rounded-lg border px-3 py-2.5">
            <span className="text-xs uppercase tracking-wider text-muted-foreground">Balance</span>
            <span className="font-mono text-sm tabular-nums">
              {balanceLoaded ? `${formatNorn(balance)} NORN` : "Loading..."}
            </span>
          </div>

          {balanceLoaded && !hasSufficientBalance && (
            <p className="animate-fade-in text-xs text-destructive">
              Insufficient balance. You need at least 1 NORN.
            </p>
          )}
        </div>

        <Button
          className="w-full"
          disabled={!isValid}
          onClick={handleRegister}
        >
          {loading ? <Spinner size="sm" /> : "Register Name"}
        </Button>

        {/* Owned names */}
        <div className="mt-2">
          <h3 className="mb-2 text-sm font-medium">Your Names</h3>
          {namesLoading ? (
            <div className="flex justify-center py-4">
              <Spinner size="sm" />
            </div>
          ) : ownedNames.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-6 text-muted-foreground animate-fade-in">
              <AtSign className="h-5 w-5" />
              <p className="text-sm">No names registered</p>
            </div>
          ) : (
            <div className="divide-y divide-border rounded-lg border">
              {ownedNames.map((n, i) => (
                <div
                  key={n.name}
                  className="flex items-center gap-2 px-3 py-2.5 animate-slide-in"
                  style={{ animationDelay: `${i * 50}ms`, animationFillMode: "backwards" }}
                >
                  <Check className="h-3.5 w-3.5 text-emerald-500" />
                  <span className="text-sm font-medium">{n.name}</span>
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

function isValidName(name: string): boolean {
  if (name.length < 3 || name.length > 32) return false;
  if (name.startsWith("-") || name.endsWith("-")) return false;
  return /^[a-z0-9-]+$/.test(name);
}
