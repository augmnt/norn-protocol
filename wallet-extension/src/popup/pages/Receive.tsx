import { useWalletStore } from "@/stores/wallet-store";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { QRCodeDisplay } from "../components/ui/qr-code";
import { CopyButton } from "../components/ui/copy-button";

export function Receive() {
  const accounts = useWalletStore((s) => s.accounts);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);
  const activeAccount = accounts.find((a) => a.id === activeAccountId);
  const address = activeAccount?.address ?? "";

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col items-center justify-center gap-6 p-4">
        <div className="space-y-1 text-center">
          <h2 className="text-lg font-semibold">Receive NORN</h2>
          <p className="text-sm text-muted-foreground">
            Share this address to receive funds
          </p>
        </div>

        <div className="rounded-xl border bg-card p-6">
          <QRCodeDisplay value={address} size={180} />
        </div>

        <div className="flex max-w-full items-center gap-2 rounded-lg border px-3 py-2.5">
          <code className="break-all font-mono text-xs leading-relaxed">
            {address}
          </code>
          <CopyButton value={address} />
        </div>
      </div>

      <BottomNav />
    </div>
  );
}
