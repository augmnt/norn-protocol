"use client";

import { useWallet } from "@/hooks/use-wallet";
import { useNames } from "@/hooks/use-names";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { QRCodeDisplay } from "@/components/ui/qr-code";
import { CopyButton } from "@/components/ui/copy-button";
import { Badge } from "@/components/ui/badge";

export default function ReceivePage() {
  const { activeAddress } = useWallet();
  const { data: names } = useNames(activeAddress ?? undefined);

  return (
    <PageContainer title="Receive" description="Share your address to receive NORN">
      <div className="max-w-sm mx-auto">
        <Card>
          <CardContent className="pt-8 pb-8 flex flex-col items-center gap-6">
            {activeAddress && (
              <>
                {/* Instruction */}
                <p className="text-xs text-muted-foreground text-center max-w-[260px] leading-relaxed">
                  Share this address or QR code to receive NORN tokens
                </p>

                {/* QR Code */}
                <div className="rounded-xl bg-white p-4 shadow-sm">
                  <QRCodeDisplay value={activeAddress} size={200} />
                </div>

                {/* Address Container */}
                <div className="w-full rounded-lg bg-secondary border px-3 py-3">
                  <div className="flex items-start justify-between gap-2">
                    <p className="font-mono text-xs break-all flex-1 leading-relaxed text-foreground select-all">
                      {activeAddress}
                    </p>
                    <CopyButton value={activeAddress} className="shrink-0 mt-0.5" />
                  </div>
                </div>

                {/* NornNames */}
                {names && names.length > 0 && (
                  <div className="w-full flex flex-col items-center gap-2">
                    <p className="text-xs text-muted-foreground uppercase tracking-wider font-medium">
                      NornNames
                    </p>
                    <div className="flex flex-wrap justify-center gap-1.5">
                      {names.map((n) => (
                        <Badge
                          key={n.name}
                          variant="secondary"
                          className="text-xs font-normal px-2.5 py-0.5"
                        >
                          {n.name}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
