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
import { useDeployLoom } from "@/hooks/use-deploy-loom";
import { getCodeHashForAppType } from "@/lib/code-hash-registry";
import { APPS } from "@/lib/apps-config";
import { ArrowLeft, Rocket, Loader2 } from "lucide-react";
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
  const appConfig = APPS.find((a) => a.id === appType);
  const [name, setName] = useState("");

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
      // Fetch the WASM bytecode for this app type
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

  return (
    <PageContainer
      title={`Deploy ${appConfig?.name ?? appType}`}
      action={
        <Link href={`/apps/${appType}`}>
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-1.5 h-3.5 w-3.5" />
            Back
          </Button>
        </Link>
      }
    >
      <div className="max-w-lg">
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-norn/10">
                <Rocket className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">
                  Deploy New {appConfig?.name ?? appType}
                </CardTitle>
                <CardDescription>
                  Deploy a new instance of this contract on-chain. This costs 50
                  NORN.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Contract Name
              </Label>
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
              onClick={handleDeploy}
              disabled={!nameValid || deploying}
              disabledReason={disabledReason}
              className="w-full"
            >
              {deploying ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <Rocket className="mr-2 h-3.5 w-3.5" />
              )}
              Deploy Contract
            </FormButton>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
