import { useState } from "react";
import { Eye, EyeOff } from "lucide-react";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Spinner } from "../components/ui/spinner";

export function Unlock() {
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [shaking, setShaking] = useState(false);

  const unlock = useWalletStore((s) => s.unlock);
  const accounts = useWalletStore((s) => s.accounts);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);
  const reset = useNavigationStore((s) => s.reset);

  const activeAccount = accounts.find((a) => a.id === activeAccountId);

  const handleUnlock = async () => {
    if (!password) return;
    setLoading(true);
    setError("");
    try {
      await unlock(password);
      reset("dashboard");
    } catch {
      setError("Incorrect password");
      setPassword("");
      setShaking(true);
      setTimeout(() => setShaking(false), 400);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleUnlock();
  };

  return (
    <div className="flex h-full flex-col items-center justify-center px-8">
      <div className="mb-8 flex flex-col items-center gap-2 animate-fade-in">
        <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-norn/20">
          <span className="text-2xl font-bold text-norn">N</span>
        </div>
        <h1 className="text-lg font-bold">Welcome Back</h1>
        {activeAccount && (
          <p className="text-sm text-muted-foreground">{activeAccount.name}</p>
        )}
      </div>

      <div className="flex w-full flex-col gap-3 animate-slide-in">
        <div className="space-y-1.5">
          <div className={`relative ${shaking ? "animate-shake" : ""}`}>
            <Input
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => {
                setPassword(e.target.value);
                setError("");
              }}
              onKeyDown={handleKeyDown}
              placeholder="Enter password"
              autoFocus
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors duration-150 hover:text-foreground"
            >
              {showPassword ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
            </button>
          </div>
          {error && (
            <p className="animate-fade-in text-xs text-destructive">{error}</p>
          )}
        </div>

        <Button
          className="w-full"
          disabled={!password || loading}
          onClick={handleUnlock}
        >
          {loading ? <Spinner size="sm" /> : "Unlock"}
        </Button>
      </div>
    </div>
  );
}
