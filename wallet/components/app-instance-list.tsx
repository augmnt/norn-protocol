"use client";

import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { useAppInstances } from "@/hooks/use-app-instances";
import { APPS } from "@/lib/apps-config";
import { truncateHash, timeAgo } from "@/lib/format";
import {
  ArrowLeft,
  ArrowRight,
  Plus,
  Boxes,
  type LucideIcon,
  ShieldCheck,
  Vault,
  Hourglass,
  Rocket,
  GitFork,
  HandCoins,
  Vote,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
  Waves,
} from "lucide-react";

const ICON_MAP: Record<string, LucideIcon> = {
  ShieldCheck,
  Vault,
  Hourglass,
  Rocket,
  GitFork,
  HandCoins,
  Vote,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
  Waves,
};

interface AppInstanceListProps {
  appType: string;
}

export function AppInstanceList({ appType }: AppInstanceListProps) {
  const { data: instances, isLoading, error } = useAppInstances(appType);
  const appConfig = APPS.find((a) => a.id === appType);
  const Icon = appConfig ? ICON_MAP[appConfig.icon] ?? Boxes : Boxes;

  return (
    <PageContainer
      title={appConfig?.name ?? appType}
      description={appConfig?.description}
      action={
        <div className="flex items-center gap-2">
          <Link href="/discover">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
              All Apps
            </Button>
          </Link>
          <Link href={`/apps/${appType}/deploy`}>
            <Button size="sm">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Deploy New
            </Button>
          </Link>
        </div>
      }
    >
      {isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Card key={i}>
              <CardContent className="p-6">
                <Skeleton className="h-4 w-32 mb-3" />
                <Skeleton className="h-3 w-48 mb-2" />
                <Skeleton className="h-3 w-24" />
              </CardContent>
            </Card>
          ))}
        </div>
      ) : error ? (
        <Card>
          <CardContent className="p-6">
            <p className="text-sm text-destructive">
              Failed to load instances: {error.message}
            </p>
          </CardContent>
        </Card>
      ) : !instances || instances.length === 0 ? (
        <EmptyState
          icon={Icon}
          title={`No ${appConfig?.name ?? appType} instances found`}
          description="Deploy a new instance to get started."
          action={
            <Link href={`/apps/${appType}/deploy`}>
              <Button size="sm">
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                Deploy New
              </Button>
            </Link>
          }
        />
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {instances.map((loom) => (
            <Link key={loom.loom_id} href={`/apps/${appType}/${loom.loom_id}`}>
              <Card className="group h-full transition-colors hover:border-norn/40">
                <CardContent className="p-6">
                  <div className="flex items-start justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-norn/10">
                      <Icon className="h-5 w-5 text-norn" />
                    </div>
                    <Badge
                      variant={loom.active ? "norn" : "secondary"}
                      className="text-[10px]"
                    >
                      {loom.active ? "Active" : "Inactive"}
                    </Badge>
                  </div>
                  <h3 className="mt-4 text-sm font-semibold">{loom.name}</h3>
                  <div className="mt-2 space-y-1">
                    <p className="text-xs text-muted-foreground">
                      <span className="font-mono">{truncateHash(loom.loom_id, 8)}</span>
                    </p>
                    {loom.deployed_at > 0 && (
                      <p className="text-xs text-muted-foreground">
                        Deployed {timeAgo(loom.deployed_at)}
                      </p>
                    )}
                    {loom.participant_count > 0 && (
                      <p className="text-xs text-muted-foreground">
                        {loom.participant_count} participant{loom.participant_count !== 1 ? "s" : ""}
                      </p>
                    )}
                  </div>
                  <div className="mt-4 flex items-center gap-1 text-xs text-norn opacity-0 transition-opacity group-hover:opacity-100">
                    Open
                    <ArrowRight className="h-3 w-3" />
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </PageContainer>
  );
}
