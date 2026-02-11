"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useWallet } from "@/hooks/use-wallet";
import { usePasskeyAuth } from "@/hooks/use-passkey-auth";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Fingerprint, Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";

export default function UnlockPage() {
  const router = useRouter();
  const { meta, prfSupported } = useWallet();
  const auth = usePasskeyAuth();
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  const usesPrf = meta?.usesPrf ?? false;

  const handleUnlock = async () => {
    try {
      let success: boolean;
      if (usesPrf) {
        success = await auth.unlock();
      } else {
        success = await auth.unlockWithPassword(password);
      }
      if (success) {
        router.replace("/dashboard");
      } else {
        toast.error(auth.error || "Unlock failed");
      }
    } catch {
      toast.error(auth.error || "Unlock failed");
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center p-4 bg-background">
      <div className="w-full max-w-sm space-y-8">
        {/* Logo and account info */}
        <div className="text-center space-y-5">
          <div className="mx-auto flex items-center justify-center space-x-2">
            <span className="font-mono text-2xl font-bold tracking-[-0.02em] text-foreground">norn</span>
            <span className="text-xs text-muted-foreground">wallet</span>
          </div>
          <p className="text-sm text-muted-foreground">
            {usesPrf
              ? "Authenticate with your passkey to unlock"
              : "Enter your password to unlock"}
          </p>
        </div>

        {/* Auth form */}
        <div className="space-y-4">
          {!usesPrf && (
            <div className="relative">
              <Input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Password"
                onKeyDown={(e) => e.key === "Enter" && handleUnlock()}
                className="h-12 pr-10"
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
          )}

          <Button
            className="w-full h-12"
            onClick={handleUnlock}
            disabled={auth.loading || (!usesPrf && !password)}
          >
            {auth.loading ? (
              <span className="flex items-center gap-2">
                <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                Unlocking...
              </span>
            ) : usesPrf ? (
              <>
                <Fingerprint className="mr-2 h-5 w-5" />
                Unlock with Passkey
              </>
            ) : (
              "Unlock"
            )}
          </Button>
        </div>

        {/* Recovery link */}
        <p className="text-center text-xs text-muted-foreground">
          Lost access?{" "}
          <Link
            href="/onboarding"
            className="text-norn hover:underline underline-offset-2"
          >
            Import with recovery phrase
          </Link>
        </p>
      </div>
    </div>
  );
}
