"use client";

import { use } from "react";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { HashDisplay } from "@/components/ui/hash-display";
import { AddressDisplay } from "@/components/ui/address-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { useTransaction } from "@/hooks/use-transaction";
import { formatTimestamp } from "@/lib/format";
import { strip0x } from "@/lib/format";

export default function TransactionDetailPage({
  params,
}: {
  params: Promise<{ hash: string }>;
}) {
  const { hash } = use(params);
  const knotId = strip0x(hash);
  const { data: tx, isLoading, error, refetch } = useTransaction(knotId);

  if (isLoading) {
    return (
      <PageContainer title="Transaction">
        <Breadcrumbs
          items={[
            { label: "Transactions", href: "/transactions" },
            { label: `Tx ${hash.slice(0, 10)}...` },
          ]}
        />
        <CardSkeleton />
      </PageContainer>
    );
  }

  if (error || tx === undefined) {
    return (
      <PageContainer title="Transaction">
        <Breadcrumbs
          items={[
            { label: "Transactions", href: "/transactions" },
            { label: `Tx ${hash.slice(0, 10)}...` },
          ]}
        />
        <ErrorState
          message="Failed to load transaction"
          retry={() => refetch()}
        />
      </PageContainer>
    );
  }

  if (tx === null) {
    return (
      <PageContainer title="Transaction">
        <div className="space-y-6">
          <Breadcrumbs
            items={[
              { label: "Transactions", href: "/transactions" },
              { label: `Tx ${hash.slice(0, 10)}...` },
            ]}
          />
          <Card>
            <CardContent className="pt-6">
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">
                  Transaction not found. It may have been pruned from memory or
                  the ID may be incorrect.
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </PageContainer>
    );
  }

  return (
    <PageContainer title="Transaction">
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Transactions", href: "/transactions" },
            { label: `Tx ${hash.slice(0, 10)}...` },
          ]}
        />

        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">
              Transaction Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            <dl className="grid gap-4">
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Transaction ID
                </dt>
                <dd>
                  <HashDisplay hash={tx.knot_id} truncate={false} />
                </dd>
              </div>

              <div className="grid gap-4 sm:grid-cols-2">
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    From
                  </dt>
                  <dd>
                    <AddressDisplay address={tx.from} />
                  </dd>
                </div>
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    To
                  </dt>
                  <dd>
                    <AddressDisplay address={tx.to} />
                  </dd>
                </div>
              </div>

              <div className="grid gap-4 sm:grid-cols-2">
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    Amount
                  </dt>
                  <dd className="font-mono text-sm">
                    {tx.human_readable}
                  </dd>
                </div>
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    Block
                  </dt>
                  <dd className="text-sm">
                    {tx.block_height != null ? (
                      <Link
                        href={`/block/${tx.block_height}`}
                        className="font-mono text-norn hover:underline tabular-nums"
                      >
                        #{tx.block_height}
                      </Link>
                    ) : (
                      <span className="text-muted-foreground">Pending</span>
                    )}
                  </dd>
                </div>
              </div>

              <div className="grid gap-4 sm:grid-cols-2">
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    Timestamp
                  </dt>
                  <dd className="text-sm">
                    {formatTimestamp(tx.timestamp)}
                    <span className="ml-2">
                      <TimeAgo timestamp={tx.timestamp} />
                    </span>
                  </dd>
                </div>
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    Token
                  </dt>
                  <dd>
                    <HashDisplay hash={tx.token_id} href={`/token/${tx.token_id}`} chars={8} />
                  </dd>
                </div>
              </div>

              {tx.memo && (
                <div>
                  <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                    Memo
                  </dt>
                  <dd className="text-sm break-all">{tx.memo}</dd>
                </div>
              )}
            </dl>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
