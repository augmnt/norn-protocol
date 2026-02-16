import { Download, Terminal, Shield, Lock, ChevronRight } from "lucide-react";
import { useNavigationStore } from "@/stores/navigation-store";
import { Button } from "../components/ui/button";

export function Welcome() {
  const navigate = useNavigationStore((s) => s.navigate);

  return (
    <div className="flex h-full flex-col overflow-y-auto scrollbar-thin">
      <div className="flex flex-1 flex-col items-center justify-center px-6 py-6">
        <div className="w-full max-w-sm space-y-6">
          {/* Brand */}
          <div className="text-center space-y-4 animate-fade-in">
            <div className="flex items-center justify-center gap-1.5">
              <span className="font-mono text-2xl font-bold tracking-[-0.02em] text-foreground">
                norn
              </span>
              <span className="text-xs text-muted-foreground">wallet</span>
            </div>
            <div className="space-y-1.5">
              <h1 className="text-lg font-semibold tracking-tight">
                Welcome to Norn
              </h1>
              <p className="text-sm text-muted-foreground">
                A secure wallet for the Norn Protocol blockchain.
              </p>
            </div>
          </div>

          {/* Feature pills */}
          <div className="grid gap-2.5 animate-slide-in">
            <div className="flex items-start gap-3 rounded-xl border bg-card p-3">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-secondary">
                <Shield className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              <div className="space-y-0.5">
                <p className="text-sm font-medium">Self-Custodial</p>
                <p className="text-xs text-muted-foreground">
                  Your keys, your crypto. Only you control your funds.
                </p>
              </div>
            </div>
            <div className="flex items-start gap-3 rounded-xl border bg-card p-3">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-secondary">
                <Lock className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              <div className="space-y-0.5">
                <p className="text-sm font-medium">Password Secured</p>
                <p className="text-xs text-muted-foreground">
                  Your keys are encrypted and stored locally in the browser.
                </p>
              </div>
            </div>
          </div>

          {/* CTAs */}
          <div className="flex flex-col gap-2.5 animate-slide-in">
            <Button
              className="w-full"
              onClick={() => navigate("create-wallet")}
            >
              Create New Wallet
              <ChevronRight className="ml-1 h-4 w-4" />
            </Button>

            <Button
              variant="outline"
              className="w-full"
              onClick={() => navigate("import-wallet")}
            >
              <Download className="h-4 w-4" />
              Import Existing
            </Button>

            <Button
              variant="ghost"
              className="w-full text-muted-foreground"
              onClick={() => navigate("import-cli")}
            >
              <Terminal className="h-4 w-4" />
              Import from CLI
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
