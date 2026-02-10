"use client";

import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatCard } from "@/components/ui/stat-card";
import { DataTable } from "@/components/ui/data-table";
import { HashDisplay } from "@/components/ui/hash-display";
import { AddressDisplay } from "@/components/ui/address-display";
import { AmountDisplay } from "@/components/ui/amount-display";
import { Badge } from "@/components/ui/badge";
import { StatsSkeleton, TableSkeleton } from "@/components/ui/loading-skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { useStakingInfo } from "@/hooks/use-staking-info";
import { useValidatorSet } from "@/hooks/use-validator-set";
import { formatNorn, formatNumber } from "@/lib/format";
import { Shield, Coins, Users } from "lucide-react";
import type { ValidatorInfo } from "@/types";

const columns = [
  {
    header: "Validator",
    key: "pubkey",
    render: (v: ValidatorInfo) => (
      <HashDisplay hash={v.pubkey} chars={8} />
    ),
  },
  {
    header: "Address",
    key: "address",
    render: (v: ValidatorInfo) => <AddressDisplay address={v.address} />,
  },
  {
    header: "Stake",
    key: "stake",
    className: "text-right",
    render: (v: ValidatorInfo) => <AmountDisplay amount={v.stake} />,
  },
  {
    header: "Status",
    key: "status",
    className: "text-right",
    render: (v: ValidatorInfo) => (
      <Badge variant={v.active ? "default" : "secondary"}>
        {v.active ? "Active" : "Inactive"}
      </Badge>
    ),
  },
];

export default function ValidatorsPage() {
  const {
    data: staking,
    isLoading: stakingLoading,
    error: stakingError,
    refetch: refetchStaking,
  } = useStakingInfo();
  const {
    data: validatorSet,
    isLoading: vsLoading,
    error: vsError,
    refetch: refetchVs,
  } = useValidatorSet();

  const isLoading = stakingLoading || vsLoading;
  const error = stakingError || vsError;

  return (
    <PageContainer title="Validators">
      <div className="space-y-6">
        {isLoading ? (
          <StatsSkeleton count={3} />
        ) : error ? (
          <ErrorState
            message="Failed to load staking info"
            retry={() => {
              refetchStaking();
              refetchVs();
            }}
          />
        ) : (
          <>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <StatCard
                label="Total Staked"
                value={
                  staking
                    ? `${formatNorn(staking.total_staked)} NORN`
                    : "—"
                }
                icon={Coins}
              />
              <StatCard
                label="Validators"
                value={
                  validatorSet
                    ? formatNumber(validatorSet.validators.length)
                    : "—"
                }
                icon={Users}
              />
              <StatCard
                label="Epoch"
                value={
                  validatorSet ? formatNumber(validatorSet.epoch) : "—"
                }
                icon={Shield}
              />
            </div>

            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium">
                  Validator Set
                </CardTitle>
              </CardHeader>
              <CardContent className="px-0">
                <DataTable
                  columns={columns}
                  data={
                    validatorSet
                      ? [...validatorSet.validators].sort(
                          (a, b) =>
                            Number(BigInt(b.stake) - BigInt(a.stake))
                        )
                      : []
                  }
                  keyExtractor={(v) => v.pubkey}
                  emptyMessage="No validators"
                />
              </CardContent>
            </Card>

            {staking && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm font-medium">
                    Staking Parameters
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <dl className="grid gap-4 sm:grid-cols-2">
                    <div>
                      <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                        Minimum Stake
                      </dt>
                      <dd className="font-mono text-sm">
                        {formatNorn(staking.min_stake)} NORN
                      </dd>
                    </div>
                    <div>
                      <dt className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
                        Bonding Period
                      </dt>
                      <dd className="font-mono text-sm">
                        {staking.bonding_period} blocks
                      </dd>
                    </div>
                  </dl>
                </CardContent>
              </Card>
            )}
          </>
        )}
      </div>
    </PageContainer>
  );
}
