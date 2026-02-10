"use client";

import { PageContainer } from "@/components/ui/page-container";
import { StatsBar } from "@/components/dashboard/stats-bar";
import { NetworkInfo } from "@/components/dashboard/network-info";
import { RecentBlocks } from "@/components/dashboard/recent-blocks";
import { RecentTransactions } from "@/components/dashboard/recent-transactions";
import { BlockProductionChart } from "@/components/charts/block-production-chart";
import { TransactionVolumeChart } from "@/components/charts/transaction-volume-chart";
import { NetworkActivityChart } from "@/components/charts/network-activity-chart";
import { FavoritesBar } from "@/components/dashboard/favorites-bar";
import { useChartData } from "@/hooks/use-chart-data";

export default function DashboardPage() {
  const { chartData } = useChartData();

  return (
    <PageContainer>
      <div className="space-y-6">
        <StatsBar />
        <NetworkInfo />
        <div className="grid gap-6 lg:grid-cols-2">
          <BlockProductionChart data={chartData} />
          <TransactionVolumeChart data={chartData} />
        </div>
        <NetworkActivityChart data={chartData} />
        <FavoritesBar />
        <div className="grid gap-6 lg:grid-cols-2">
          <RecentBlocks />
          <RecentTransactions />
        </div>
      </div>
    </PageContainer>
  );
}
