import { useState, useMemo } from "react";
import { Terminal, Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";
import { Wallet } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { isValidPrivateKeyHex } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

export function ImportFromCli() {
  const [name, setName] = useState("CLI Wallet");
  const [privateKey, setPrivateKey] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);

  const importExistingAccount = useWalletStore((s) => s.importExistingAccount);
  const reset = useNavigationStore((s) => s.reset);

  const previewAddress = useMemo(() => {
    if (!isValidPrivateKeyHex(privateKey)) return null;
    try {
      const wallet = Wallet.fromPrivateKeyHex(privateKey);
      return wallet.addressHex;
    } catch {
      return null;
    }
  }, [privateKey]);

  const isValid =
    name.trim().length > 0 &&
    isValidPrivateKeyHex(privateKey) &&
    password.length >= 8 &&
    password === confirmPassword;

  const handleImport = async () => {
    if (!isValid) return;
    setLoading(true);
    try {
      await importExistingAccount(name.trim(), privateKey, password);
      reset("dashboard");
      toast.success("CLI wallet imported successfully");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to import wallet",
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
          <h2 className="text-lg font-semibold">Import from CLI</h2>
          <p className="text-sm text-muted-foreground">
            Export your private key from the Norn CLI, then paste it below.
          </p>
        </div>

        <Card className="border-norn/20 bg-norn/5">
          <CardContent className="flex items-start gap-3 p-3">
            <Terminal className="mt-0.5 h-4 w-4 shrink-0 text-norn" />
            <div className="space-y-1.5">
              <p className="text-xs text-muted-foreground">
                Run this command in your terminal:
              </p>
              <code className="block rounded bg-background/80 px-2 py-1.5 font-mono text-xs leading-relaxed text-foreground">
                norn wallet export &lt;name&gt; --show-private-key
              </code>
              <p className="text-xs text-muted-foreground">
                Copy the 64-character hex key from the output.
              </p>
            </div>
          </CardContent>
        </Card>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Private Key (hex)</label>
            <textarea
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value.trim())}
              placeholder="Paste 64-char hex key from CLI export"
              className="flex min-h-[72px] w-full rounded-md border border-input bg-transparent px-3 py-2 font-mono text-xs shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-norn/50 focus-visible:border-norn/50 resize-none transition-colors duration-150"
            />
            {previewAddress && (
              <p className="animate-fade-in font-mono text-xs text-emerald-400">
                Address: {previewAddress}
              </p>
            )}
            {privateKey.length > 0 && !isValidPrivateKeyHex(privateKey) && (
              <p className="animate-fade-in text-xs text-destructive">
                Invalid key format (expected 64 hex characters, with optional 0x prefix)
              </p>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Account Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="CLI Wallet"
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
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-150"
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
          onClick={handleImport}
        >
          {loading ? <Spinner size="sm" /> : "Import Wallet"}
        </Button>
      </div>
    </div>
  );
}
