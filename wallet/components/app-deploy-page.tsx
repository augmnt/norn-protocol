"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { PageContainer } from "@/components/ui/page-container";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { FormButton } from "@/components/ui/form-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  TooltipProvider,
} from "@/components/ui/tooltip";
import { useDeployLoom } from "@/hooks/use-deploy-loom";
import { useAppInstances } from "@/hooks/use-app-instances";
import { useNetwork } from "@/hooks/use-network";
import { APPS } from "@/lib/apps-config";
import { Rocket, Loader2, HelpCircle, ArrowLeft, Info } from "lucide-react";
import { toast } from "sonner";

interface AppDeployPageProps {
  appType: string;
  /** Optional extra form fields rendered below the name input */
  children?: React.ReactNode;
  /** Called after deployment with loomId; return init msg hex or undefined */
  buildInitMsg?: () => string | undefined;
}

export function AppDeployPage({
  appType,
  children,
  buildInitMsg,
}: AppDeployPageProps) {
  const router = useRouter();
  const { deploy, deploying } = useDeployLoom();
  const { network, isTestnet } = useNetwork();
  const { data: instances } = useAppInstances(appType);
  const appConfig = APPS.find((a) => a.id === appType);
  const appName = appConfig?.name ?? appType;

  const [name, setName] = useState("");
  const [showConfirm, setShowConfirm] = useState(false);

  const nameValid =
    name.trim().length > 0 &&
    name.length <= 64 &&
    /^[a-z0-9-]+$/.test(name.trim());

  const disabledReason = !name.trim()
    ? "Enter a contract name"
    : !nameValid
      ? "Name must be lowercase alphanumeric with hyphens"
      : undefined;

  const handleDeploy = async () => {
    if (!nameValid) return;

    try {
      const wasmUrl = `/contracts/${appType}.wasm`;
      const response = await fetch(wasmUrl);
      if (!response.ok) {
        throw new Error(`Failed to load contract bytecode (${response.status})`);
      }
      const wasmBytes = new Uint8Array(await response.arrayBuffer());

      const initMsgHex = buildInitMsg?.();

      const loomId = await deploy({
        name: name.trim(),
        wasmBytes,
        initMsgHex,
      });

      toast.success("Contract deployed successfully");
      router.push(`/apps/${appType}/${loomId}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Deployment failed");
    }
  };

  const existingCount = instances?.length ?? 0;

  return (
    <PageContainer
      title={`Deploy ${appName}`}
      breadcrumb={[
        { label: "Apps", href: "/discover" },
        { label: appName, href: `/apps/${appType}` },
        { label: "Deploy" },
      ]}
      action={undefined}
    >
      <div className="max-w-lg">
        {/* J: Use Existing prompt */}
        {existingCount > 0 && (
          <Card className="mb-4 border-border bg-muted/50">
            <CardContent className="flex items-start gap-3 p-4">
              <Info className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
              <div className="text-sm text-muted-foreground">
                There {existingCount === 1 ? "is" : "are"} already{" "}
                <span className="font-medium text-foreground">
                  {existingCount}
                </span>{" "}
                {appName} contract{existingCount !== 1 ? "s" : ""} deployed.{" "}
                <Link
                  href={`/apps/${appType}`}
                  className="text-norn hover:underline"
                >
                  View existing
                </Link>
              </div>
            </CardContent>
          </Card>
        )}

        {!showConfirm ? (
          /* Step 1: Form */
          <Card>
            <CardHeader className="pb-4">
              <div className="flex items-center gap-3">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                  <Rocket className="h-4 w-4 text-norn" />
                </div>
                <div>
                  <CardTitle className="text-base">
                    Deploy New {appName}
                  </CardTitle>
                  <CardDescription>
                    Configure your contract, then review before deploying.
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <TooltipProvider>
                  <div className="flex items-center gap-1.5">
                    <Label className="text-xs text-muted-foreground">
                      Contract Name
                    </Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <HelpCircle className="h-3 w-3 text-muted-foreground/60 cursor-help" />
                      </TooltipTrigger>
                      <TooltipContent>
                        A unique identifier for your contract. Lowercase letters,
                        numbers, and hyphens.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                </TooltipProvider>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value.toLowerCase())}
                  placeholder="my-crowdfund"
                  maxLength={64}
                  className="font-mono text-sm"
                />
                <p className="text-[10px] text-muted-foreground">
                  Lowercase letters, numbers, and hyphens only.
                </p>
              </div>

              {children}

              <FormButton
                onClick={() => setShowConfirm(true)}
                disabled={!nameValid}
                disabledReason={disabledReason}
                className="w-full"
              >
                Review Deployment
              </FormButton>
            </CardContent>
          </Card>
        ) : (
          /* Step 2: Confirmation */
          <Card>
            <CardHeader className="pb-4">
              <div className="flex items-center gap-3">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                  <Rocket className="h-4 w-4 text-norn" />
                </div>
                <div>
                  <CardTitle className="text-base">
                    Confirm Deployment
                  </CardTitle>
                  <CardDescription>
                    Review the details before deploying on-chain.
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="rounded-lg border border-border bg-muted/30 p-4 space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Contract Type</span>
                  <span className="font-medium">{appName}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Contract Name</span>
                  <span className="font-mono font-medium">{name.trim()}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Network</span>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{network.name}</span>
                    {isTestnet && (
                      <Badge variant="outline" className="text-[10px]">
                        Testnet
                      </Badge>
                    )}
                  </div>
                </div>
                <div className="border-t border-border pt-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">
                      Deployment Fee
                    </span>
                    <span className="text-lg font-semibold font-mono tabular-nums">
                      50 NORN
                    </span>
                  </div>
                  <p className="mt-1 text-[10px] text-muted-foreground">
                    This fee registers the contract on-chain. It is not
                    refundable.
                  </p>
                </div>
              </div>

              <div className="flex gap-3">
                <Button
                  variant="ghost"
                  className="flex-1"
                  onClick={() => setShowConfirm(false)}
                  disabled={deploying}
                >
                  <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
                  Back
                </Button>
                <FormButton
                  onClick={handleDeploy}
                  disabled={deploying}
                  className="flex-1"
                >
                  {deploying ? (
                    <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Rocket className="mr-2 h-3.5 w-3.5" />
                  )}
                  Confirm &amp; Deploy
                </FormButton>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </PageContainer>
  );
}
