"use client";

import { use } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { HashDisplay } from "@/components/ui/hash-display";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { TimeAgo } from "@/components/ui/time-ago";
import { DataTable } from "@/components/ui/data-table";
import { CardSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { Breadcrumbs } from "@/components/ui/breadcrumbs";
import { BlockCompositionChart } from "@/components/charts/block-composition-chart";
import { useBlock, useBlockTransactions } from "@/hooks/use-block";
import { useWeaveState } from "@/hooks/use-weave-state";
import { formatNumber, formatTimestamp } from "@/lib/format";
import { NATIVE_TOKEN_ID } from "@/lib/constants";
import type {
  BlockTransferInfo,
  BlockTokenDefinitionInfo,
  BlockTokenMintInfo,
  BlockTokenBurnInfo,
  BlockNameRegistrationInfo,
  BlockNameTransferInfo,
  BlockNameRecordUpdateInfo,
  BlockLoomDeployInfo,
} from "@/types";

const transferColumns = [
  {
    header: "Tx Hash",
    key: "knot_id",
    render: (tx: BlockTransferInfo) => (
      <HashDisplay
        hash={tx.knot_id}
        href={`/tx/${tx.knot_id}`}
        chars={6}
        copy={false}
      />
    ),
  },
  {
    header: "From",
    key: "from",
    render: (tx: BlockTransferInfo) => <AddressDisplay address={tx.from} />,
  },
  {
    header: "To",
    key: "to",
    render: (tx: BlockTransferInfo) => <AddressDisplay address={tx.to} />,
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (tx: BlockTransferInfo) => (
      <AmountDisplay
        amount={tx.amount}
        humanReadable={tx.human_readable}
        symbol={tx.symbol || (tx.token_id === NATIVE_TOKEN_ID ? "NORN" : tx.token_id.slice(0, 8) + "\u2026")}
      />
    ),
  },
  {
    header: "Memo",
    key: "memo",
    render: (tx: BlockTransferInfo) => (
      <span className="text-sm text-muted-foreground">{tx.memo || "—"}</span>
    ),
  },
  {
    header: "Time",
    key: "time",
    className: "text-right",
    render: (tx: BlockTransferInfo) => <TimeAgo timestamp={tx.timestamp} />,
  },
];

const tokenDefColumns = [
  {
    header: "Name",
    key: "name",
    render: (t: BlockTokenDefinitionInfo) => (
      <span className="font-medium text-sm">{t.name}</span>
    ),
  },
  {
    header: "Symbol",
    key: "symbol",
    render: (t: BlockTokenDefinitionInfo) => (
      <Badge variant="outline">{t.symbol}</Badge>
    ),
  },
  {
    header: "Decimals",
    key: "decimals",
    className: "text-right",
    render: (t: BlockTokenDefinitionInfo) => (
      <span className="font-mono text-sm">{t.decimals}</span>
    ),
  },
  {
    header: "Creator",
    key: "creator",
    render: (t: BlockTokenDefinitionInfo) => (
      <AddressDisplay address={t.creator} />
    ),
  },
];

const tokenMintColumns = [
  {
    header: "Token",
    key: "token_id",
    render: (t: BlockTokenMintInfo) => (
      <Link href={`/token/${t.token_id}`} className="text-norn hover:underline">
        {t.symbol || <HashDisplay hash={t.token_id} chars={6} copy={false} />}
      </Link>
    ),
  },
  {
    header: "To",
    key: "to",
    render: (t: BlockTokenMintInfo) => <AddressDisplay address={t.to} />,
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (t: BlockTokenMintInfo) => (
      <AmountDisplay
        amount={t.amount}
        humanReadable={t.human_readable}
        symbol={t.symbol || ""}
      />
    ),
  },
];

const tokenBurnColumns = [
  {
    header: "Token",
    key: "token_id",
    render: (t: BlockTokenBurnInfo) => (
      <Link href={`/token/${t.token_id}`} className="text-norn hover:underline">
        {t.symbol || <HashDisplay hash={t.token_id} chars={6} copy={false} />}
      </Link>
    ),
  },
  {
    header: "Burner",
    key: "burner",
    render: (t: BlockTokenBurnInfo) => (
      <AddressDisplay address={t.burner} />
    ),
  },
  {
    header: "Amount",
    key: "amount",
    className: "text-right",
    render: (t: BlockTokenBurnInfo) => (
      <AmountDisplay
        amount={t.amount}
        humanReadable={t.human_readable}
        symbol={t.symbol || ""}
      />
    ),
  },
];

const nameRegColumns = [
  {
    header: "Name",
    key: "name",
    render: (n: BlockNameRegistrationInfo) => (
      <span className="font-medium text-sm">{n.name}</span>
    ),
  },
  {
    header: "Owner",
    key: "owner",
    render: (n: BlockNameRegistrationInfo) => (
      <AddressDisplay address={n.owner} />
    ),
  },
  {
    header: "Fee",
    key: "fee_paid",
    className: "text-right",
    render: (n: BlockNameRegistrationInfo) => (
      <AmountDisplay amount={n.fee_paid} />
    ),
  },
];

const nameTransferColumns = [
  {
    header: "Name",
    key: "name",
    render: (n: BlockNameTransferInfo) => (
      <span className="font-medium text-sm">{n.name}</span>
    ),
  },
  {
    header: "From",
    key: "from",
    render: (n: BlockNameTransferInfo) => <AddressDisplay address={n.from} />,
  },
  {
    header: "To",
    key: "to",
    render: (n: BlockNameTransferInfo) => <AddressDisplay address={n.to} />,
  },
];

const nameRecordUpdateColumns = [
  {
    header: "Name",
    key: "name",
    render: (n: BlockNameRecordUpdateInfo) => (
      <span className="font-medium text-sm">{n.name}</span>
    ),
  },
  {
    header: "Key",
    key: "key",
    render: (n: BlockNameRecordUpdateInfo) => (
      <Badge variant="outline">{n.key}</Badge>
    ),
  },
  {
    header: "Value",
    key: "value",
    render: (n: BlockNameRecordUpdateInfo) => (
      <span className="text-sm truncate max-w-[200px] inline-block">
        {n.value}
      </span>
    ),
  },
  {
    header: "Owner",
    key: "owner",
    render: (n: BlockNameRecordUpdateInfo) => (
      <AddressDisplay address={n.owner} />
    ),
  },
];

const loomDeployColumns = [
  {
    header: "Name",
    key: "name",
    render: (l: BlockLoomDeployInfo) => (
      <span className="font-medium text-sm">{l.name}</span>
    ),
  },
  {
    header: "Operator",
    key: "operator",
    render: (l: BlockLoomDeployInfo) => (
      <span className="font-mono text-sm">{l.operator}</span>
    ),
  },
];

export default function BlockDetailPage({
  params,
}: {
  params: Promise<{ height: string }>;
}) {
  const { height: heightStr } = use(params);
  const router = useRouter();
  const height = parseInt(heightStr, 10);
  const { data: block, isLoading, error, refetch } = useBlock(height);
  const { data: blockTxs } = useBlockTransactions(height);
  const { data: weave } = useWeaveState();

  if (isLoading) {
    return (
      <PageContainer title={`Block #${formatNumber(height)}`}>
        <CardSkeleton />
      </PageContainer>
    );
  }

  if (error || !block) {
    return (
      <PageContainer title={`Block #${formatNumber(height)}`}>
        <ErrorState
          message="Block not found or failed to load"
          retry={() => refetch()}
        />
      </PageContainer>
    );
  }

  const activityCounts = [
    { label: "Transfers", count: block.transfer_count },
    { label: "Names", count: block.name_registration_count },
    { label: "Name Transfers", count: block.name_transfer_count ?? 0 },
    { label: "Record Updates", count: block.name_record_update_count ?? 0 },
    { label: "Token Defs", count: block.token_definition_count },
    { label: "Mints", count: block.token_mint_count },
    { label: "Burns", count: block.token_burn_count },
    { label: "Looms", count: block.loom_deploy_count },
    { label: "Stake Ops", count: block.stake_operation_count },
  ];

  const hasTransactions = blockTxs && (
    blockTxs.transfers.length > 0 ||
    blockTxs.token_definitions.length > 0 ||
    blockTxs.token_mints.length > 0 ||
    blockTxs.token_burns.length > 0 ||
    blockTxs.name_registrations.length > 0 ||
    (blockTxs.name_transfers?.length ?? 0) > 0 ||
    (blockTxs.name_record_updates?.length ?? 0) > 0 ||
    blockTxs.loom_deploys.length > 0
  );

  return (
    <PageContainer
      title={`Block #${formatNumber(height)}`}
      action={
        <div className="flex items-center gap-2">
          <Button variant="outline" size="icon" asChild disabled={height <= 0}>
            <Link href={height > 0 ? `/block/${height - 1}` : "#"}>
              <ChevronLeft className="h-4 w-4" />
            </Link>
          </Button>
          <Button variant="outline" size="icon" asChild disabled={!weave || height >= weave.height}>
            <Link href={weave && height < weave.height ? `/block/${height + 1}` : "#"}>
              <ChevronRight className="h-4 w-4" />
            </Link>
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        <Breadcrumbs
          items={[
            { label: "Blocks", href: "/blocks" },
            { label: `Block #${formatNumber(height)}` },
          ]}
        />
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">
              Block Information
            </CardTitle>
          </CardHeader>
          <CardContent>
            <dl className="grid gap-4 sm:grid-cols-2">
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Height
                </dt>
                <dd className="font-mono text-sm tabular-nums">
                  {formatNumber(block.height)}
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Timestamp
                </dt>
                <dd className="text-sm">
                  {formatTimestamp(block.timestamp)}
                  <span className="ml-2">
                    <TimeAgo timestamp={block.timestamp} />
                  </span>
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Hash
                </dt>
                <dd>
                  <HashDisplay hash={block.hash} truncate={false} />
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Previous Hash
                </dt>
                <dd>
                  <HashDisplay
                    hash={block.prev_hash}
                    href={height > 0 ? `/block/${height - 1}` : undefined}
                    truncate={false}
                  />
                </dd>
              </div>
              <div className="sm:col-span-2">
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  State Root
                </dt>
                <dd>
                  <HashDisplay hash={block.state_root} truncate={false} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Proposer
                </dt>
                <dd>
                  <HashDisplay hash={block.proposer} chars={8} />
                </dd>
              </div>
              <div>
                <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                  Commitments
                </dt>
                <dd className="font-mono text-sm">
                  {block.commitment_count}
                </dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        <div className="grid gap-6 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">
                Block Activity
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {activityCounts.map((item) => (
                  <Badge
                    key={item.label}
                    variant={item.count > 0 ? "default" : "secondary"}
                  >
                    {item.label}: {item.count}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">
                Composition
              </CardTitle>
            </CardHeader>
            <CardContent>
              <BlockCompositionChart block={block} />
            </CardContent>
          </Card>
        </div>

        {hasTransactions && (
          <div className="space-y-6">
            {blockTxs.transfers.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Transfers ({blockTxs.transfers.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={transferColumns}
                    data={blockTxs.transfers}
                    keyExtractor={(tx) => tx.knot_id}
                    onRowClick={(tx) => router.push(`/tx/${tx.knot_id}`)}
                  />
                </CardContent>
              </Card>
            )}

            {blockTxs.token_definitions.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Token Definitions ({blockTxs.token_definitions.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={tokenDefColumns}
                    data={blockTxs.token_definitions}
                    keyExtractor={(t) => `${t.symbol}-${t.timestamp}`}
                  />
                </CardContent>
              </Card>
            )}

            {blockTxs.token_mints.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Token Mints ({blockTxs.token_mints.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={tokenMintColumns}
                    data={blockTxs.token_mints}
                    keyExtractor={(t) => `${t.token_id}-${t.to}-${t.timestamp}`}
                  />
                </CardContent>
              </Card>
            )}

            {blockTxs.token_burns.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Token Burns ({blockTxs.token_burns.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={tokenBurnColumns}
                    data={blockTxs.token_burns}
                    keyExtractor={(t) => `${t.token_id}-${t.burner}-${t.timestamp}`}
                  />
                </CardContent>
              </Card>
            )}

            {blockTxs.name_registrations.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Name Registrations ({blockTxs.name_registrations.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={nameRegColumns}
                    data={blockTxs.name_registrations}
                    keyExtractor={(n) => n.name}
                  />
                </CardContent>
              </Card>
            )}

            {(blockTxs.name_transfers?.length ?? 0) > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Name Transfers ({blockTxs.name_transfers!.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={nameTransferColumns}
                    data={blockTxs.name_transfers!}
                    keyExtractor={(n) => `${n.name}-${n.from}-${n.to}`}
                  />
                </CardContent>
              </Card>
            )}

            {(blockTxs.name_record_updates?.length ?? 0) > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Name Record Updates ({blockTxs.name_record_updates!.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={nameRecordUpdateColumns}
                    data={blockTxs.name_record_updates!}
                    keyExtractor={(n) => `${n.name}-${n.key}-${n.timestamp}`}
                  />
                </CardContent>
              </Card>
            )}

            {blockTxs.loom_deploys.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Loom Deploys ({blockTxs.loom_deploys.length})
                  </CardTitle>
                </CardHeader>
                <CardContent className="px-0">
                  <DataTable
                    columns={loomDeployColumns}
                    data={blockTxs.loom_deploys}
                    keyExtractor={(l) => `${l.name}-${l.timestamp}`}
                  />
                </CardContent>
              </Card>
            )}
          </div>
        )}
      </div>
    </PageContainer>
  );
}
