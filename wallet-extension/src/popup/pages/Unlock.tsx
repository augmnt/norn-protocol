import { useState } from "react";
import { Eye, EyeOff, Lock } from "lucide-react";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";

export function Unlock() {
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [shaking, setShaking] = useState(false);

  const unlock = useWalletStore((s) => s.unlock);
  const reset = useNavigationStore((s) => s.reset);

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
      <div className="w-full max-w-sm space-y-8">
        {/* Brand */}
        <div className="flex flex-col items-center gap-5 animate-fade-in">
          <div className="flex h-14 w-14 items-center justify-center rounded-2xl border bg-card">
            <Lock className="h-5 w-5 text-norn" />
          </div>
          <div className="text-center space-y-2">
            <div className="flex items-center justify-center gap-1.5">
              <span className="font-mono text-xl font-bold tracking-[-0.02em] text-foreground">
                norn
              </span>
              <span className="text-xs text-muted-foreground">wallet</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Enter your password to unlock
            </p>
          </div>
        </div>

        {/* Auth form */}
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
                placeholder="Password"
                autoFocus
                className="pr-9"
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors duration-150 hover:text-foreground"
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
            {error && (
              <p className="animate-fade-in text-xs text-destructive">
                {error}
              </p>
            )}
          </div>

          <Button
            className="w-full"
            disabled={!password || loading}
            onClick={handleUnlock}
          >
            {loading ? (
              <span className="flex items-center gap-2">
                <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                Unlocking...
              </span>
            ) : (
              "Unlock"
            )}
          </Button>
        </div>

        {/* Recovery link */}
        <p className="text-center text-xs text-muted-foreground">
          Lost access?{" "}
          <button
            type="button"
            onClick={() => reset("welcome")}
            className="text-norn hover:underline underline-offset-2"
          >
            Import with recovery key
          </button>
        </p>
      </div>
    </div>
  );
}
