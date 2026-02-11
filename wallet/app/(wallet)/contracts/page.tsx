"use client";

import { useState } from "react";
import { useLoomOps } from "@/hooks/use-loom-ops";
import { useSavedContractsStore } from "@/stores/saved-contracts-store";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { CopyButton } from "@/components/ui/copy-button";
import { explorerContractUrl } from "@/lib/explorer";
import { truncateHash } from "@/lib/format";
import { FileCode, Search, Play, Terminal, AlertCircle, X, Bookmark, BookmarkPlus, Trash2, Info } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

export default function ContractsPage() {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();
  const savedContracts = useSavedContractsStore((s) => s.contracts);
  const saveContract = useSavedContractsStore((s) => s.save);
  const removeContract = useSavedContractsStore((s) => s.remove);

  const [loomId, setLoomId] = useState("");
  const [inputHex, setInputHex] = useState("");
  const [result, setResult] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  const [saveLabel, setSaveLabel] = useState("");
  const [showSaveInput, setShowSaveInput] = useState(false);

  const isSaved = savedContracts.some(
    (c) => c.loomId.toLowerCase() === loomId.toLowerCase()
  );

  const handleQuery = async () => {
    setIsError(false);
    try {
      const res = await queryLoom(loomId, inputHex);
      setResult(JSON.stringify(res, null, 2));
    } catch {
      setIsError(true);
      setResult(error || "Query failed");
    }
  };

  const handleExecute = async () => {
    setIsError(false);
    try {
      const res = await executeLoom(loomId, inputHex);
      setResult(JSON.stringify(res, null, 2));
    } catch {
      setIsError(true);
      setResult(error || "Execution failed");
    }
  };

  const handleSave = () => {
    if (!loomId || !saveLabel.trim()) return;
    saveContract(loomId, saveLabel.trim());
    setShowSaveInput(false);
    setSaveLabel("");
    toast.success("Contract saved");
  };

  const clearResult = () => {
    setResult(null);
    setIsError(false);
  };

  return (
    <PageContainer title="Contracts" description="Interact with deployed Loom smart contracts">
      <div className="max-w-2xl space-y-4">
        {/* Saved Contracts */}
        {savedContracts.length > 0 && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm font-medium">
                <Bookmark className="h-3.5 w-3.5 text-muted-foreground" />
                Saved Contracts
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-1">
                {savedContracts.map((c) => (
                  <div
                    key={c.loomId}
                    className="flex items-center justify-between py-2 px-2 -mx-2 rounded-md hover:bg-muted/50 transition-colors group cursor-pointer"
                    onClick={() => setLoomId(c.loomId)}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="text-sm font-medium truncate">{c.label}</span>
                      <span className="font-mono text-xs text-muted-foreground">
                        {truncateHash(c.loomId, 6)}
                      </span>
                    </div>
                    <div className="flex items-center gap-1">
                      <a
                        href={explorerContractUrl(c.loomId)}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-xs text-muted-foreground hover:text-norn"
                        onClick={(e) => e.stopPropagation()}
                      >
                        Explorer
                      </a>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6 text-muted-foreground opacity-0 group-hover:opacity-100"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeContract(c.loomId);
                          toast.success("Contract removed");
                        }}
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Input Card */}
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-full bg-secondary">
                <FileCode className="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <CardTitle className="text-base">Contract Interaction</CardTitle>
                <CardDescription>
                  Query state or execute transactions on a deployed Loom contract.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="text-xs text-muted-foreground">Loom ID</Label>
                {loomId && !isSaved && (
                  showSaveInput ? (
                    <div className="flex items-center gap-1.5">
                      <Input
                        value={saveLabel}
                        onChange={(e) => setSaveLabel(e.target.value)}
                        placeholder="Label"
                        className="h-6 w-32 text-xs"
                        onKeyDown={(e) => e.key === "Enter" && handleSave()}
                        autoFocus
                      />
                      <Button variant="ghost" size="icon" className="h-6 w-6" onClick={handleSave}>
                        <BookmarkPlus className="h-3 w-3" />
                      </Button>
                      <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setShowSaveInput(false)}>
                        <X className="h-3 w-3" />
                      </Button>
                    </div>
                  ) : (
                    <button
                      className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
                      onClick={() => setShowSaveInput(true)}
                    >
                      <BookmarkPlus className="h-3 w-3" />
                      Save
                    </button>
                  )
                )}
                {loomId && isSaved && (
                  <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                    <Bookmark className="h-2.5 w-2.5 mr-1" />
                    Saved
                  </Badge>
                )}
              </div>
              <Input
                value={loomId}
                onChange={(e) => setLoomId(e.target.value)}
                placeholder="64-character hex contract ID"
                className="font-mono text-sm"
              />
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Input Message (hex)</Label>
              <Textarea
                value={inputHex}
                onChange={(e) => setInputHex(e.target.value)}
                placeholder="Hex-encoded borsh-serialized input message"
                className="font-mono text-sm min-h-[80px] resize-y"
                rows={3}
              />
            </div>

            {/* Help text */}
            <div className="flex items-start gap-2 rounded-lg bg-secondary/50 p-3">
              <Info className="h-3.5 w-3.5 text-muted-foreground mt-0.5 shrink-0" />
              <div className="text-[11px] text-muted-foreground leading-relaxed space-y-1">
                <p><strong>Query</strong> reads contract state (free, no signature needed).</p>
                <p><strong>Execute</strong> modifies state (requires signing, may cost gas).</p>
                <p>Input must be hex-encoded borsh-serialized data matching the contract ABI.</p>
              </div>
            </div>

            <div className="flex gap-2 pt-1">
              <Button
                variant="outline"
                onClick={handleQuery}
                disabled={loading || !loomId}
                className="flex-1"
              >
                {loading ? (
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent mr-2" />
                ) : (
                  <Search className="mr-2 h-3.5 w-3.5" />
                )}
                Query
              </Button>
              <Button
                onClick={handleExecute}
                disabled={loading || !loomId}
                className="flex-1"
              >
                {loading ? (
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent mr-2" />
                ) : (
                  <Play className="mr-2 h-3.5 w-3.5" />
                )}
                Execute
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Result Card */}
        {result && (
          <Card className={cn(
            "overflow-hidden",
            isError && "border-destructive/30"
          )}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {isError ? (
                    <AlertCircle className="h-4 w-4 text-destructive" />
                  ) : (
                    <Terminal className="h-4 w-4 text-green-400" />
                  )}
                  <CardTitle className={cn(
                    "text-sm",
                    isError ? "text-destructive" : "text-green-400"
                  )}>
                    {isError ? "Error" : "Result"}
                  </CardTitle>
                </div>
                <div className="flex items-center gap-1">
                  {!isError && (
                    <CopyButton value={result} />
                  )}
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-muted-foreground"
                    onClick={clearResult}
                  >
                    <X className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <pre className={cn(
                "rounded-lg p-4 text-xs font-mono overflow-auto max-h-72 whitespace-pre-wrap scrollbar-thin",
                isError
                  ? "bg-destructive/5 text-destructive"
                  : "bg-secondary/50 text-foreground"
              )}>
                {result}
              </pre>
            </CardContent>
          </Card>
        )}
      </div>
    </PageContainer>
  );
}
