import { ArrowUpRight, ArrowDownLeft, ExternalLink } from "lucide-react";
import { toast } from "sonner";
import { useNavigationStore } from "@/stores/navigation-store";
import { useNetworkStore } from "@/stores/network-store";
import { truncateAddress, timeAgo, formatTimestamp } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { TransactionHistoryEntry } from "@/types";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Card, CardContent } from "../components/ui/card";
import { CopyButton } from "../components/ui/copy-button";

const NATIVE_TOKEN_ID = "0".repeat(64);

function getExplorerUrl(rpcUrl: string): string | null {
  if (rpcUrl.includes("seed.norn.network")) return "https://explorer.norn.network";
  if (rpcUrl.includes("localhost") || rpcUrl.includes("127.0.0.1")) return "http://localhost:3001";
  return null;
}

export function TransactionDetail() {
  const params = useNavigationStore((s) => s.params);
  const rpcUrl = useNetworkStore((s) => s.rpcUrl);
  const tx = params.tx as TransactionHistoryEntry | undefined;

  if (!tx) {
    return (
      <div className="flex h-full flex-col">
        <Header />
        <div className="flex flex-1 items-center justify-center text-muted-foreground">
          <p className="text-sm">Transaction not found</p>
        </div>
        <BottomNav />
      </div>
    );
  }

  const isSent = tx.direction === "sent";
  const isNative = !tx.token_id || tx.token_id === NATIVE_TOKEN_ID || tx.token_id === `0x${NATIVE_TOKEN_ID}`;
  const symbol = isNative ? "NORN" : (tx.symbol || "TOKEN");
  const explorerUrl = getExplorerUrl(rpcUrl);

  const handleCopy = (value: string) => {
    navigator.clipboard.writeText(value);
    toast.success("Copied to clipboard");
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="flex flex-col items-center gap-3 animate-fade-in">
          <div
            className={cn(
              "flex h-12 w-12 items-center justify-center rounded-full",
              isSent
                ? "bg-orange-500/10 text-orange-400"
                : "bg-emerald-500/10 text-emerald-400",
            )}
          >
            {isSent ? (
              <ArrowUpRight className="h-6 w-6" />
            ) : (
              <ArrowDownLeft className="h-6 w-6" />
            )}
          </div>

          <div className="text-center">
            <p
              className={cn(
                "font-mono text-xl font-semibold tabular-nums",
                isSent ? "text-orange-400" : "text-emerald-400",
              )}
            >
              {isSent ? "-" : "+"}
              {tx.human_readable} {symbol}
            </p>
            <p className="text-sm text-muted-foreground">
              {isSent ? "Sent" : "Received"} {tx.timestamp ? timeAgo(tx.timestamp) : ""}
            </p>
          </div>
        </div>

        <Card className="animate-slide-in">
          <CardContent className="space-y-3 p-4">
            <DetailRow
              label="From"
              value={tx.from}
              truncated={truncateAddress(tx.from)}
              onCopy={() => handleCopy(tx.from)}
              explorerUrl={explorerUrl ? `${explorerUrl}/address/${tx.from}` : undefined}
            />

            <DetailRow
              label="To"
              value={tx.to}
              truncated={truncateAddress(tx.to)}
              onCopy={() => handleCopy(tx.to)}
              explorerUrl={explorerUrl ? `${explorerUrl}/address/${tx.to}` : undefined}
            />

            <div className="border-t pt-3">
              <DetailRow
                label="Amount"
                value={`${tx.human_readable} ${symbol}`}
                mono
              />
            </div>

            {!isNative && (
              <DetailRow
                label="Token"
                value={symbol}
              />
            )}

            {tx.memo && (
              <DetailRow
                label="Memo"
                value={tx.memo}
              />
            )}

            {tx.block_height != null && (
              <DetailRow
                label="Block"
                value={`#${tx.block_height.toLocaleString()}`}
              />
            )}

            {tx.timestamp > 0 && (
              <DetailRow
                label="Time"
                value={formatTimestamp(tx.timestamp)}
              />
            )}

            <div className="border-t pt-3">
              <div className="flex items-start justify-between gap-2">
                <span className="text-xs uppercase tracking-wider text-muted-foreground shrink-0">
                  Tx Hash
                </span>
                <div className="flex items-center gap-1">
                  <span className="break-all text-right font-mono text-xs">
                    {truncateAddress(tx.knot_id)}
                  </span>
                  <CopyButton value={tx.knot_id} />
                </div>
              </div>
            </div>

            {explorerUrl && (
              <a
                href={`${explorerUrl}/tx/${tx.knot_id}`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center justify-center gap-1.5 rounded-lg border p-2.5 text-sm text-norn transition-colors duration-150 hover:bg-norn/10"
              >
                <ExternalLink className="h-3.5 w-3.5" />
                View on Explorer
              </a>
            )}
          </CardContent>
        </Card>
      </div>

      <BottomNav />
    </div>
  );
}

function DetailRow({
  label,
  value,
  truncated,
  mono,
  onCopy,
  explorerUrl,
}: {
  label: string;
  value: string;
  truncated?: string;
  mono?: boolean;
  onCopy?: () => void;
  explorerUrl?: string;
}) {
  return (
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs uppercase tracking-wider text-muted-foreground shrink-0">
        {label}
      </span>
      <div className="flex items-center gap-1">
        <span className={cn("text-sm text-right", mono && "font-mono tabular-nums")}>
          {truncated ?? value}
        </span>
        {onCopy && <CopyButton value={value} />}
        {explorerUrl && (
          <a
            href={explorerUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground transition-colors duration-150 hover:text-norn"
          >
            <ExternalLink className="h-3 w-3" />
          </a>
        )}
      </div>
    </div>
  );
}
