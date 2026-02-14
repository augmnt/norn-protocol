"use client";

import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { APPS } from "@/lib/apps-config";
import {
  ShieldCheck,
  Vault,
  Timer,
  Rocket,
  Split,
  Heart,
  Scale,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
  ArrowRight,
  type LucideIcon,
} from "lucide-react";

const ICON_MAP: Record<string, LucideIcon> = {
  ShieldCheck,
  Vault,
  Timer,
  Rocket,
  Split,
  Heart,
  Scale,
  Landmark,
  ArrowLeftRight,
  Gift,
  Clock,
};

export default function AppsPage() {
  return (
    <PageContainer
      title="Apps"
      description="Decentralized applications powered by Loom smart contracts"
    >
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {APPS.map((app) => {
          const Icon = ICON_MAP[app.icon] ?? ShieldCheck;
          return (
            <Link key={app.id} href={app.href}>
              <Card className="group h-full transition-colors hover:border-norn/40">
                <CardContent className="p-6">
                  <div className="flex items-start justify-between">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-norn/10">
                      <Icon className="h-5 w-5 text-norn" />
                    </div>
                    {app.loomId ? (
                      <Badge variant="norn" className="text-[10px]">
                        Live
                      </Badge>
                    ) : (
                      <Badge variant="secondary" className="text-[10px]">
                        Coming Soon
                      </Badge>
                    )}
                  </div>
                  <h3 className="mt-4 text-sm font-semibold">{app.name}</h3>
                  <p className="mt-1.5 text-xs text-muted-foreground leading-relaxed">
                    {app.description}
                  </p>
                  <div className="mt-4 flex items-center gap-1 text-xs text-norn opacity-0 transition-opacity group-hover:opacity-100">
                    Open app
                    <ArrowRight className="h-3 w-3" />
                  </div>
                </CardContent>
              </Card>
            </Link>
          );
        })}
      </div>

      {APPS.length === 0 && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-sm text-muted-foreground">No apps available yet.</p>
        </div>
      )}
    </PageContainer>
  );
}
