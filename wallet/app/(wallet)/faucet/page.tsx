"use client";

import { useState } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { useFaucet } from "@/hooks/use-faucet";
import { useBalance } from "@/hooks/use-balance";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { AddressDisplay } from "@/components/ui/address-display";
import { formatNorn } from "@/lib/format";
import { explorerAddressUrl } from "@/lib/explorer";
import { Droplets, Check, Info } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

export default function FaucetPage() {
  const { activeAddress } = useWallet();
  const { requestFaucet, loading, error } = useFaucet();
  const { data: balance } = useBalance(activeAddress ?? undefined);
  const [success, setSuccess] = useState(false);

  const handleRequest = async () => {
    setSuccess(false);
    try {
      await requestFaucet();
      setSuccess(true);
      toast.success("Faucet tokens received!");
    } catch {
      toast.error(error || "Faucet request failed");
    }
  };

  return (
    <PageContainer title="Faucet" description="Request testnet NORN tokens">
      <Card className="max-w-sm mx-auto overflow-hidden">
        <CardHeader className="text-center pb-4">
          <div className="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-secondary">
            <Droplets className="h-7 w-7 text-muted-foreground" />
          </div>
          <CardTitle className="text-lg">Devnet Faucet</CardTitle>
          <CardDescription>
            Get free NORN tokens for testing on the devnet.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-5 pt-2">
          {activeAddress && (
            <div className="rounded-lg bg-secondary/50 p-3.5 text-center space-y-1">
              <p className="text-[11px] text-muted-foreground uppercase tracking-wider font-medium">Your Address</p>
              <AddressDisplay address={activeAddress} href={explorerAddressUrl(activeAddress)} />
            </div>
          )}

          <div className="text-center space-y-1">
            <p className="text-[11px] text-muted-foreground uppercase tracking-wider font-medium">Current Balance</p>
            <p className="font-mono text-2xl font-semibold tabular-nums tracking-tight">
              {formatNorn(balance?.balance ?? "0")}
              <span className="text-sm text-muted-foreground ml-1.5">NORN</span>
            </p>
          </div>

          <Separator />

          <Button
            className="w-full h-11"
            onClick={handleRequest}
            disabled={loading}
          >
            {loading ? (
              <span className="flex items-center gap-2">
                <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                Requesting...
              </span>
            ) : (
              <>
                <Droplets className="mr-2 h-4 w-4" />
                Request Tokens
              </>
            )}
          </Button>

          {success && (
            <div className="rounded-lg bg-secondary/50 border border-border p-3.5">
              <div className="flex items-center gap-2.5">
                <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-secondary">
                  <Check className="h-3.5 w-3.5 text-muted-foreground" />
                </div>
                <div>
                  <p className="text-sm font-medium">Tokens received</p>
                  <p className="text-xs text-muted-foreground">
                    NORN has been sent to your wallet
                  </p>
                </div>
              </div>
            </div>
          )}

          <div className="flex items-start gap-2 rounded-lg bg-secondary/30 px-3 py-2.5">
            <Info className="h-3.5 w-3.5 text-muted-foreground mt-0.5 shrink-0" />
            <p className="text-[11px] text-muted-foreground leading-relaxed">
              Devnet tokens have no real value. Faucet requests may be rate-limited.
            </p>
          </div>
        </CardContent>
      </Card>
    </PageContainer>
  );
}
