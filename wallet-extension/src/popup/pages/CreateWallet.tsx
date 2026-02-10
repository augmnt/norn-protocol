import { useState } from "react";
import { Eye, EyeOff, AlertTriangle, CheckSquare, Square } from "lucide-react";
import { toast } from "sonner";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { toHex } from "@norn-protocol/sdk";
import { Header } from "../components/layout/Header";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent } from "../components/ui/card";
import { CopyButton } from "../components/ui/copy-button";
import { Spinner } from "../components/ui/spinner";

type Step = "form" | "backup";

export function CreateWallet() {
  const [step, setStep] = useState<Step>("form");
  const [name, setName] = useState("Account 1");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [privateKeyHex, setPrivateKeyHex] = useState("");
  const [backedUp, setBackedUp] = useState(false);

  const createNewAccount = useWalletStore((s) => s.createNewAccount);
  const reset = useNavigationStore((s) => s.reset);

  const isValid =
    name.trim().length > 0 &&
    password.length >= 8 &&
    password === confirmPassword;

  const handleCreate = async () => {
    if (!isValid) return;
    setLoading(true);
    try {
      await createNewAccount(name.trim(), password);
      const wallet = useWalletStore.getState().activeWallet;
      if (wallet) {
        setPrivateKeyHex(toHex(wallet.privateKey));
      }
      setStep("backup");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to create wallet",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleContinue = () => {
    reset("dashboard");
  };

  if (step === "backup") {
    return (
      <div className="flex h-full flex-col">
        <Header />
        <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin animate-fade-in">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold">Back Up Your Key</h2>
            <p className="text-sm text-muted-foreground">
              Save this private key somewhere safe. You will need it to recover
              your wallet.
            </p>
          </div>

          <Card className="border-orange-500/30 bg-orange-500/5">
            <CardContent className="flex items-start gap-2 p-3">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-orange-400" />
              <p className="text-xs text-orange-300">
                Never share your private key. Anyone with it has full control of
                your funds.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-3">
              <div className="flex items-start justify-between gap-2">
                <code className="break-all font-mono text-xs leading-relaxed">
                  {privateKeyHex}
                </code>
                <CopyButton value={privateKeyHex} />
              </div>
            </CardContent>
          </Card>

          <button
            onClick={() => setBackedUp(!backedUp)}
            className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            {backedUp ? (
              <CheckSquare className="h-4 w-4 text-emerald-400" />
            ) : (
              <Square className="h-4 w-4" />
            )}
            I have saved my private key securely
          </button>

          <Button
            className="w-full"
            disabled={!backedUp}
            onClick={handleContinue}
          >
            Continue to Wallet
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <Header />
      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Create New Wallet</h2>
          <p className="text-sm text-muted-foreground">
            Set up a name and password for your wallet.
          </p>
        </div>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Account Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Wallet"
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Password</label>
            <div className="relative">
              <Input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="At least 8 characters"
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showPassword ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
            {password.length > 0 && password.length < 8 && (
              <p className="animate-fade-in text-xs text-destructive">
                Password must be at least 8 characters
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Confirm Password</label>
            <Input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Repeat password"
            />
            {confirmPassword.length > 0 && password !== confirmPassword && (
              <p className="animate-fade-in text-xs text-destructive">
                Passwords do not match
              </p>
            )}
          </div>
        </div>

        <Button
          className="w-full"
          disabled={!isValid || loading}
          onClick={handleCreate}
        >
          {loading ? <Spinner size="sm" /> : "Create Wallet"}
        </Button>
      </div>
    </div>
  );
}
