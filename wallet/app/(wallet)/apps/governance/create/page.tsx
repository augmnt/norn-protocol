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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { GOVERNANCE_LOOM_ID } from "@/lib/apps-config";
import { useGovernance } from "@/hooks/use-governance";
import { ArrowLeft, Scale, Loader2 } from "lucide-react";
import { toast } from "sonner";

export default function CreateProposalPage() {
  const router = useRouter();
  const { propose, loading } = useGovernance(GOVERNANCE_LOOM_ID);

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");

  const canSubmit =
    title.trim().length > 0 && description.trim().length > 0;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    try {
      await propose(title.trim(), description.trim());
      toast.success("Proposal created successfully");
      router.push("/apps/governance");
    } catch (e) {
      toast.error(
        e instanceof Error ? e.message : "Failed to create proposal"
      );
    }
  };

  return (
    <PageContainer
      title="Create Proposal"
      action={
        <Link href="/apps/governance">
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
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-norn/10">
                <Scale className="h-4 w-4 text-norn" />
              </div>
              <div>
                <CardTitle className="text-base">
                  New Governance Proposal
                </CardTitle>
                <CardDescription>
                  Submit a proposal for the community to vote on. Proposals
                  are open for the configured voting period.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Title</Label>
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Proposal title"
                maxLength={128}
                className="text-sm"
              />
              <p className="text-[10px] text-muted-foreground text-right">
                {title.length}/128
              </p>
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                Description
              </Label>
              <Textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Describe what this proposal is about..."
                className="text-sm min-h-[120px] resize-y"
                maxLength={512}
                rows={5}
              />
              <p className="text-[10px] text-muted-foreground text-right">
                {description.length}/512
              </p>
            </div>

            <Button
              onClick={handleSubmit}
              disabled={!canSubmit || loading}
              className="w-full"
            >
              {loading ? (
                <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
              ) : (
                <Scale className="mr-2 h-3.5 w-3.5" />
              )}
              Create Proposal
            </Button>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
