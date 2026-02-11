"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@/hooks/use-wallet";
import { usePasskeyAuth } from "@/hooks/use-passkey-auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { CopyButton } from "@/components/ui/copy-button";
import {
  Fingerprint,
  KeyRound,
  Import,
  ArrowLeft,
  Check,
  Shield,
  Eye,
  EyeOff,
  Lock,
  Cloud,
  AlertTriangle,
  Sparkles,
  ChevronRight,
} from "lucide-react";
import { toast } from "sonner";

type Step = "welcome" | "create" | "import" | "backup" | "success";

export default function OnboardingPage() {
  const router = useRouter();
  const { prfSupported } = useWallet();
  const auth = usePasskeyAuth();
  const [step, setStep] = useState<Step>("welcome");
  const [walletName, setWalletName] = useState("My Wallet");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [createdAddress, setCreatedAddress] = useState<string | null>(null);
  const [backupConfirmed, setBackupConfirmed] = useState(false);

  // Import state
  const [importKey, setImportKey] = useState("");
  const [importMnemonic, setImportMnemonic] = useState("");
  const [importPassword, setImportPassword] = useState("");

  const [prfFailed, setPrfFailed] = useState(false);
  const effectivePrf = prfSupported && !prfFailed;

  const handleCreate = async () => {
    try {
      let result;
      if (effectivePrf) {
        result = await auth.create(walletName);
      } else {
        if (!password || password.length < 8) {
          toast.error("Password must be at least 8 characters");
          return;
        }
        result = await auth.createWithPassword(walletName, password);
      }
      setCreatedAddress(result.address);
      setMnemonic(result.mnemonic ?? null);
      setStep(result.mnemonic ? "backup" : "success");
    } catch (e) {
      const msg = e instanceof Error ? e.message : "";
      if (msg === "PRF_UNSUPPORTED") {
        // Browser reported PRF support but it didn't work (e.g. Brave).
        // Fall back to password-based creation.
        setPrfFailed(true);
        toast.info("Passkey PRF not supported in this browser. Please set a password instead.");
        return;
      }
      toast.error(auth.error || "Failed to create wallet");
    }
  };

  const handleImportKey = async () => {
    try {
      const result = await auth.importKey(importKey.trim(), walletName, importPassword || undefined);
      setCreatedAddress(result.address);
      setStep("success");
      toast.success("Wallet imported successfully");
    } catch {
      toast.error(auth.error || "Failed to import key");
    }
  };

  const handleImportMnemonic = async () => {
    try {
      const result = await auth.importMnemonic(importMnemonic.trim(), walletName, importPassword || undefined);
      setCreatedAddress(result.address);
      setStep("success");
      toast.success("Wallet imported successfully");
    } catch {
      toast.error(auth.error || "Failed to import mnemonic");
    }
  };

  const handleGoToDashboard = () => {
    router.replace("/dashboard");
  };

  const totalSteps = effectivePrf ? 2 : 2;

  return (
    <div className="flex min-h-screen items-center justify-center p-4 bg-background">
      <div className="w-full max-w-md">
        {/* Welcome */}
        {step === "welcome" && (
          <div className="space-y-8">
            {/* Logo and heading */}
            <div className="text-center space-y-6">
              <div className="mx-auto flex items-center justify-center space-x-2">
                <span className="font-mono text-2xl font-bold tracking-[-0.02em] text-foreground">norn</span>
                <span className="text-xs text-muted-foreground">wallet</span>
              </div>
              <div className="space-y-2">
                <h1 className="text-2xl font-semibold tracking-tight">
                  Welcome to Norn
                </h1>
                <p className="text-sm text-muted-foreground max-w-xs mx-auto">
                  A self-custodial crypto wallet designed for simplicity and security.
                </p>
              </div>
            </div>

            {/* Feature pills */}
            <div className="grid gap-3">
              <div className="flex items-start gap-3.5 rounded-xl border bg-card p-4">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-secondary">
                  <Shield className="h-4 w-4 text-muted-foreground" />
                </div>
                <div className="space-y-0.5">
                  <p className="text-sm font-medium">Self-Custodial</p>
                  <p className="text-xs text-muted-foreground">
                    Your keys, your crypto. Only you control your funds.
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3.5 rounded-xl border bg-card p-4">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-secondary">
                  <Fingerprint className="h-4 w-4 text-muted-foreground" />
                </div>
                <div className="space-y-0.5">
                  <p className="text-sm font-medium">Passkey Secured</p>
                  <p className="text-xs text-muted-foreground">
                    Biometric authentication. No passwords to remember.
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3.5 rounded-xl border bg-card p-4">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-secondary">
                  <Cloud className="h-4 w-4 text-muted-foreground" />
                </div>
                <div className="space-y-0.5">
                  <p className="text-sm font-medium">Cross-Device</p>
                  <p className="text-xs text-muted-foreground">
                    Passkeys sync via iCloud or Google across your devices.
                  </p>
                </div>
              </div>
            </div>

            {/* CTAs */}
            <div className="space-y-3">
              <Button
                className="w-full h-12"
                onClick={() => setStep("create")}
              >
                Create New Wallet
                <ChevronRight className="ml-1 h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                className="w-full h-12"
                onClick={() => setStep("import")}
              >
                <Import className="mr-2 h-4 w-4" />
                Import Existing Wallet
              </Button>
            </div>
          </div>
        )}

        {/* Create */}
        {step === "create" && (
          <Card>
            <CardHeader className="space-y-3">
              <div className="flex items-center justify-between">
                <Button
                  variant="ghost"
                  size="sm"
                  className="w-fit -ml-2"
                  onClick={() => setStep("welcome")}
                >
                  <ArrowLeft className="mr-1 h-4 w-4" />
                  Back
                </Button>
                <span className="text-xs text-muted-foreground font-medium">
                  Step 1 of {totalSteps}
                </span>
              </div>
              <div className="space-y-1">
                <CardTitle className="text-lg">Create Your Wallet</CardTitle>
                <CardDescription>
                  {effectivePrf
                    ? "Your wallet will be secured with biometric authentication via passkeys."
                    : "Choose a strong password to encrypt your wallet locally."}
                </CardDescription>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="wallet-name">Wallet Name</Label>
                <Input
                  id="wallet-name"
                  value={walletName}
                  onChange={(e) => setWalletName(e.target.value)}
                  placeholder="My Wallet"
                />
              </div>
              {!effectivePrf && (
                <div className="space-y-2">
                  <Label htmlFor="password">Password</Label>
                  <div className="relative">
                    <Input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="At least 8 characters"
                    />
                    <button
                      type="button"
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                      onClick={() => setShowPassword(!showPassword)}
                      aria-label={showPassword ? "Hide password" : "Show password"}
                    >
                      {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </button>
                  </div>
                </div>
              )}

              <div className="flex items-start gap-2.5 rounded-lg bg-secondary p-3">
                <Lock className="h-3.5 w-3.5 text-muted-foreground mt-0.5 shrink-0" />
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Your wallet is encrypted and stored locally on this device. We never have access to your keys.
                </p>
              </div>

              {!effectivePrf && (
                <div className="flex items-start gap-2.5 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3">
                  <AlertTriangle className="h-3.5 w-3.5 text-amber-500 mt-0.5 shrink-0" />
                  <div className="text-xs text-amber-500/90 leading-relaxed space-y-1">
                    <p className="font-medium">Password recovery is not possible</p>
                    <p>
                      Your browser does not support passkey PRF, so your wallet will be encrypted with this password. If you forget it, the only way to recover your wallet is using the recovery phrase shown after creation.
                    </p>
                  </div>
                </div>
              )}
            </CardContent>
            <CardFooter>
              <Button
                className="w-full h-11"
                onClick={handleCreate}
                disabled={auth.loading || !walletName.trim()}
              >
                {auth.loading ? (
                  <span className="flex items-center gap-2">
                    <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                    {effectivePrf ? "Authenticating..." : "Creating..."}
                  </span>
                ) : effectivePrf ? (
                  <>
                    <Fingerprint className="mr-2 h-4 w-4" />
                    Create with Passkey
                  </>
                ) : (
                  "Create Wallet"
                )}
              </Button>
            </CardFooter>
          </Card>
        )}

        {/* Import */}
        {step === "import" && (
          <Card>
            <CardHeader className="space-y-3">
              <Button
                variant="ghost"
                size="sm"
                className="w-fit -ml-2"
                onClick={() => setStep("welcome")}
              >
                <ArrowLeft className="mr-1 h-4 w-4" />
                Back
              </Button>
              <div className="space-y-1">
                <CardTitle className="text-lg">Import Wallet</CardTitle>
                <CardDescription>
                  Restore an existing wallet using a private key or recovery phrase.
                </CardDescription>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="import-name">Wallet Name</Label>
                <Input
                  id="import-name"
                  value={walletName}
                  onChange={(e) => setWalletName(e.target.value)}
                  placeholder="My Wallet"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="import-password">Encryption Password</Label>
                <Input
                  id="import-password"
                  type="password"
                  value={importPassword}
                  onChange={(e) => setImportPassword(e.target.value)}
                  placeholder="Password to encrypt stored key"
                />
              </div>

              <div className="flex items-start gap-2.5 rounded-lg bg-secondary p-3">
                <Lock className="h-3.5 w-3.5 text-muted-foreground mt-0.5 shrink-0" />
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Your imported key will be encrypted with this password and stored locally. If you forget the password, you can re-import using your private key or recovery phrase.
                </p>
              </div>

              <Tabs defaultValue="key" className="w-full">
                <TabsList className="w-full">
                  <TabsTrigger value="key" className="flex-1">
                    <KeyRound className="mr-1.5 h-3.5 w-3.5" />
                    Private Key
                  </TabsTrigger>
                  <TabsTrigger value="mnemonic" className="flex-1">
                    Recovery Phrase
                  </TabsTrigger>
                </TabsList>
                <TabsContent value="key" className="space-y-3 mt-3">
                  <Input
                    value={importKey}
                    onChange={(e) => setImportKey(e.target.value)}
                    placeholder="0x... or hex string (64 chars)"
                    className="font-mono text-xs"
                  />
                  <Button
                    className="w-full h-11"
                    onClick={handleImportKey}
                    disabled={auth.loading || !importKey.trim()}
                  >
                    {auth.loading ? (
                      <span className="flex items-center gap-2">
                        <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                        Importing...
                      </span>
                    ) : (
                      "Import Key"
                    )}
                  </Button>
                </TabsContent>
                <TabsContent value="mnemonic" className="space-y-3 mt-3">
                  <Textarea
                    value={importMnemonic}
                    onChange={(e) => setImportMnemonic(e.target.value)}
                    placeholder="Enter your 24-word recovery phrase, each word separated by a space..."
                    rows={4}
                    className="font-mono text-xs resize-none"
                  />
                  <Button
                    className="w-full h-11"
                    onClick={handleImportMnemonic}
                    disabled={auth.loading || !importMnemonic.trim()}
                  >
                    {auth.loading ? (
                      <span className="flex items-center gap-2">
                        <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                        Importing...
                      </span>
                    ) : (
                      "Import Recovery Phrase"
                    )}
                  </Button>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        )}

        {/* Backup Mnemonic */}
        {step === "backup" && mnemonic && (
          <Card>
            <CardHeader className="space-y-3">
              <span className="text-xs text-muted-foreground font-medium">
                Step 2 of {totalSteps}
              </span>
              <div className="space-y-1">
                <CardTitle className="text-lg">Save Your Recovery Phrase</CardTitle>
                <CardDescription>
                  Write down these 24 words in order and store them somewhere safe.
                </CardDescription>
              </div>

              {/* Critical warning banner */}
              <div className="flex items-start gap-3 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3.5">
                <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 shrink-0" />
                <div className="space-y-0.5">
                  <p className="text-xs font-medium text-amber-500">This is your only backup</p>
                  <p className="text-xs text-amber-500/80 leading-relaxed">
                    If you lose access to your passkey, this phrase is the only way to recover your wallet. Write it down on paper and store it safely.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Mnemonic grid */}
              <div className="rounded-xl border bg-secondary p-4">
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-x-3 gap-y-2.5">
                  {mnemonic.split(" ").map((word, i) => (
                    <div
                      key={i}
                      className="flex items-center gap-2 rounded-lg bg-card border px-2.5 py-2"
                    >
                      <span className="text-[10px] text-muted-foreground font-mono w-4 text-right shrink-0">
                        {i + 1}
                      </span>
                      <span className="font-mono text-xs">{word}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Copy button */}
              <div className="flex items-center justify-between rounded-lg border bg-card px-4 py-3">
                <span className="text-sm text-muted-foreground">Copy to clipboard</span>
                <CopyButton value={mnemonic} />
              </div>

              {/* Destructive warning */}
              <div className="flex items-start gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-3.5">
                <Lock className="h-4 w-4 text-destructive mt-0.5 shrink-0" />
                <p className="text-xs text-destructive/90 leading-relaxed">
                  Never share your recovery phrase with anyone. Anyone with these words has full access to your wallet and funds.
                </p>
              </div>
            </CardContent>
            <CardFooter className="flex-col gap-3">
              <label className="flex items-start gap-2.5 w-full cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={backupConfirmed}
                  onChange={(e) => setBackupConfirmed(e.target.checked)}
                  className="mt-0.5 h-4 w-4 rounded border-border accent-norn"
                />
                <span className="text-xs text-muted-foreground leading-relaxed">
                  I have written down my recovery phrase and stored it in a safe place.
                </span>
              </label>
              <Button
                variant="outline"
                className="w-full h-11"
                onClick={() => setStep("success")}
                disabled={!backupConfirmed}
              >
                I&apos;ve Saved My Recovery Phrase
                <ChevronRight className="ml-1 h-4 w-4" />
              </Button>
            </CardFooter>
          </Card>
        )}

        {/* Success */}
        {step === "success" && (
          <div className="space-y-6">
            <div className="text-center space-y-6">
              {/* Animated check */}
              <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-emerald-500/10 border border-emerald-500/20">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-emerald-500/15">
                  <Check className="h-6 w-6 text-emerald-500" />
                </div>
              </div>
              <div className="space-y-2">
                <h1 className="text-2xl font-semibold tracking-tight">You&apos;re All Set</h1>
                <p className="text-sm text-muted-foreground max-w-xs mx-auto">
                  Your wallet is ready. Get started by requesting test tokens from the faucet.
                </p>
              </div>
            </div>

            {/* Address card */}
            {createdAddress && (
              <div className="rounded-xl border bg-card p-5">
                <div className="flex items-center justify-between mb-3">
                  <p className="text-xs text-muted-foreground font-medium uppercase tracking-wider">
                    Your Address
                  </p>
                  <CopyButton value={createdAddress} />
                </div>
                <p className="font-mono text-sm break-all text-foreground leading-relaxed">
                  {createdAddress}
                </p>
              </div>
            )}

            {/* Helpful tip */}
            <div className="flex items-start gap-3 rounded-xl border bg-card p-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-secondary">
                <Sparkles className="h-4 w-4 text-muted-foreground" />
              </div>
              <div className="space-y-0.5">
                <p className="text-xs font-medium">Quick Start</p>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Head to your dashboard and use the faucet to get free test tokens to explore the network.
                </p>
              </div>
            </div>

            {/* CTA */}
            <Button
              className="w-full h-12"
              onClick={handleGoToDashboard}
            >
              Go to Dashboard
              <ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
