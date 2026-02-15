"use client";

import { useState } from "react";
import { useLoomOps } from "@/hooks/use-loom-ops";
import { useLoomsList } from "@/hooks/use-looms-list";
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
import { DataTable } from "@/components/ui/data-table";
import { Pagination } from "@/components/ui/pagination";
import { ErrorState } from "@/components/ui/error-state";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { FieldError } from "@/components/ui/field-error";
import { explorerContractUrl } from "@/lib/explorer";
import { truncateHash, timeAgo } from "@/lib/format";
import { APPS } from "@/lib/apps-config";
import { getAppTypeForCodeHash } from "@/lib/code-hash-registry";
import { PAGE_SIZE } from "@/lib/constants";
import {
  FileCode, Search, Play, Terminal, AlertCircle, X, Bookmark, BookmarkPlus,
  Trash2, Info, Layers, CheckCircle, XCircle, ChevronDown, ChevronUp,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import type { LoomInfo } from "@/types";

function isValidLoomId(value: string): boolean {
  return /^[a-fA-F0-9]{64}$/.test(value);
}

function isValidHex(value: string): boolean {
  if (!value) return true; // empty is OK (optional)
  return /^([a-fA-F0-9]{2})*$/.test(value);
}

/** Map of app type IDs to app names. */
const APP_NAMES = new Map(APPS.map((app) => [app.id, app.name]));

export default function ContractsPage() {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();
  const savedContracts = useSavedContractsStore((s) => s.contracts);
  const saveContract = useSavedContractsStore((s) => s.save);
  const removeContract = useSavedContractsStore((s) => s.remove);

  const [activeTab, setActiveTab] = useState("interact");
  const [loomId, setLoomId] = useState("");
  const [inputHex, setInputHex] = useState("");
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [resultRaw, setResultRaw] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  const [saveLabel, setSaveLabel] = useState("");
  const [showSaveInput, setShowSaveInput] = useState(false);
  const [showRawJson, setShowRawJson] = useState(false);
  const [browsePage, setBrowsePage] = useState(1);

  // Validation state
  const loomIdTouched = loomId.length > 0;
  const loomIdValid = isValidLoomId(loomId);
  const inputHexTouched = inputHex.length > 0;
  const inputHexValid = isValidHex(inputHex);

  const isSaved = savedContracts.some(
    (c) => c.loomId.toLowerCase() === loomId.toLowerCase()
  );

  // Browse tab data
  const { data: looms, isLoading: loomsLoading, error: loomsError, refetch: loomsRefetch } = useLoomsList(browsePage);

  const handleQuery = async () => {
    setIsError(false);
    setShowRawJson(false);
    try {
      const res = await queryLoom(loomId, inputHex);
      setResult(res as unknown as Record<string, unknown>);
      setResultRaw(JSON.stringify(res, null, 2));
    } catch {
      setIsError(true);
      setResult(null);
      setResultRaw(error || "Query failed");
    }
  };

  const handleExecute = async () => {
    setIsError(false);
    setShowRawJson(false);
    try {
      const res = await executeLoom(loomId, inputHex);
      setResult(res as unknown as Record<string, unknown>);
      setResultRaw(JSON.stringify(res, null, 2));
    } catch {
      setIsError(true);
      setResult(null);
      setResultRaw(error || "Execution failed");
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
    setResultRaw(null);
    setIsError(false);
  };

  const selectFromBrowse = (loom: LoomInfo) => {
    setLoomId(loom.loom_id);
    setActiveTab("interact");
  };

  return (
    <PageContainer title="Contracts" description="Interact with deployed Loom smart contracts">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="max-w-2xl">
        <TabsList>
          <TabsTrigger value="interact">
            <FileCode className="h-3.5 w-3.5 mr-1.5" />
            Interact
          </TabsTrigger>
          <TabsTrigger value="browse">
            <Layers className="h-3.5 w-3.5 mr-1.5" />
            Browse
          </TabsTrigger>
        </TabsList>

        {/* ─── Interact Tab ─── */}
        <TabsContent value="interact" className="space-y-4">
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
                          className="h-9 w-9 md:h-7 md:w-7 text-muted-foreground opacity-100 md:opacity-0 md:group-hover:opacity-100"
                          onClick={(e) => {
                            e.stopPropagation();
                            removeContract(c.loomId);
                            toast.success("Contract removed");
                          }}
                        >
                          <Trash2 className="h-3.5 w-3.5 md:h-3 md:w-3" />
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
                          className="h-7 w-32 text-xs"
                          onKeyDown={(e) => e.key === "Enter" && handleSave()}
                          autoFocus
                        />
                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleSave}>
                          <BookmarkPlus className="h-3 w-3" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setShowSaveInput(false)}>
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
                  className={cn(
                    "font-mono text-sm",
                    loomIdTouched && !loomIdValid && "border-destructive"
                  )}
                />
                <FieldError
                  message="Loom ID must be exactly 64 hex characters"
                  show={loomIdTouched && !loomIdValid}
                />
              </div>

              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">Input Message (hex)</Label>
                <Textarea
                  value={inputHex}
                  onChange={(e) => setInputHex(e.target.value)}
                  placeholder="Hex-encoded borsh-serialized input message"
                  className={cn(
                    "font-mono text-sm min-h-[80px] resize-y",
                    inputHexTouched && !inputHexValid && "border-destructive"
                  )}
                  rows={3}
                />
                <FieldError
                  message="Input must be valid hex (even number of 0-9, a-f characters)"
                  show={inputHexTouched && !inputHexValid}
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
                  disabled={loading || !loomIdValid || (inputHexTouched && !inputHexValid)}
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
                  disabled={loading || !loomIdValid || (inputHexTouched && !inputHexValid)}
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

          {/* Structured Result Card */}
          {(result || resultRaw) && (
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
                      <Terminal className="h-4 w-4 text-foreground" />
                    )}
                    <CardTitle className={cn(
                      "text-sm",
                      isError ? "text-destructive" : "text-foreground"
                    )}>
                      {isError ? "Error" : "Result"}
                    </CardTitle>
                    {!isError && result && (
                      <Badge variant={result.success ? "norn" : "destructive"} className="text-[10px]">
                        {result.success ? "Success" : "Failed"}
                      </Badge>
                    )}
                  </div>
                  <div className="flex items-center gap-1">
                    {!isError && resultRaw && (
                      <CopyButton value={resultRaw} />
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
              <CardContent className="pt-0 space-y-3">
                {isError ? (
                  <pre className="rounded-lg p-4 text-xs font-mono overflow-auto max-h-72 whitespace-pre-wrap scrollbar-thin bg-destructive/5 text-destructive">
                    {resultRaw}
                  </pre>
                ) : result ? (
                  <>
                    {/* Gas used */}
                    {typeof result.gas_used === "number" && (
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <span>Gas used:</span>
                        <span className="font-mono tabular-nums">{(result.gas_used as number).toLocaleString()}</span>
                      </div>
                    )}

                    {/* Output */}
                    {result.output && (
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Output</span>
                        <pre className="rounded-lg p-3 text-xs font-mono overflow-auto max-h-48 whitespace-pre-wrap scrollbar-thin bg-secondary/50 text-foreground">
                          {typeof result.output === "string" ? result.output : JSON.stringify(result.output, null, 2)}
                        </pre>
                      </div>
                    )}

                    {/* Events */}
                    {Array.isArray(result.events) && (result.events as Array<Record<string, unknown>>).length > 0 && (
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Events ({(result.events as unknown[]).length})</span>
                        <div className="space-y-1.5">
                          {(result.events as Array<Record<string, unknown>>).map((evt, i) => (
                            <div key={i} className="rounded-lg bg-secondary/50 p-2.5 text-xs">
                              <span className="font-medium">{String(evt.type || evt.kind || `Event ${i}`)}</span>
                              {Array.isArray(evt.attributes) && (
                                <div className="mt-1 space-y-0.5 font-mono text-muted-foreground">
                                  {(evt.attributes as Array<{ key: string; value: string }>).map((attr, j) => (
                                    <div key={j}>{attr.key}: {attr.value}</div>
                                  ))}
                                </div>
                              )}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Logs */}
                    {Array.isArray(result.logs) && (result.logs as string[]).length > 0 && (
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Logs</span>
                        <div className="rounded-lg bg-secondary/50 p-3 text-xs font-mono space-y-0.5">
                          {(result.logs as string[]).map((log, i) => (
                            <div key={i} className="text-muted-foreground">{log}</div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Raw JSON toggle */}
                    <button
                      className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
                      onClick={() => setShowRawJson(!showRawJson)}
                    >
                      {showRawJson ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
                      {showRawJson ? "Hide" : "Show"} raw JSON
                    </button>
                    {showRawJson && (
                      <pre className="rounded-lg p-4 text-xs font-mono overflow-auto max-h-72 whitespace-pre-wrap scrollbar-thin bg-secondary/50 text-foreground">
                        {resultRaw}
                      </pre>
                    )}
                  </>
                ) : null}
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* ─── Browse Tab ─── */}
        <TabsContent value="browse" className="space-y-4">
          <Card>
            <CardHeader className="pb-4">
              <div className="flex items-center gap-3">
                <div className="flex h-9 w-9 items-center justify-center rounded-full bg-secondary">
                  <Layers className="h-4 w-4 text-muted-foreground" />
                </div>
                <div>
                  <CardTitle className="text-base">Deployed Contracts</CardTitle>
                  <CardDescription>
                    Browse on-chain contracts. Click a row to interact.
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              {loomsLoading ? (
                <div className="space-y-3">
                  {Array.from({ length: 5 }).map((_, i) => (
                    <Skeleton key={i} className="h-12 w-full" />
                  ))}
                </div>
              ) : loomsError ? (
                <ErrorState
                  message="Failed to load contracts"
                  retry={() => loomsRefetch()}
                />
              ) : !looms || looms.length === 0 ? (
                <EmptyState
                  icon={FileCode}
                  title="No contracts deployed yet"
                />
              ) : (
                <>
                  <DataTable<LoomInfo>
                    columns={[
                      {
                        header: "Name",
                        key: "name",
                        render: (loom) => {
                          const appType = loom.code_hash ? getAppTypeForCodeHash(loom.code_hash) : undefined;
                          const knownApp = appType ? APP_NAMES.get(appType) : undefined;
                          return (
                            <div className="flex items-center gap-2">
                              <span className="font-medium text-sm truncate max-w-[140px]">
                                {loom.name || "Unnamed"}
                              </span>
                              {knownApp && (
                                <Badge variant="norn" className="text-[9px] px-1.5 py-0 shrink-0">
                                  {knownApp}
                                </Badge>
                              )}
                            </div>
                          );
                        },
                      },
                      {
                        header: "Loom ID",
                        key: "loom_id",
                        hideOnMobile: true,
                        render: (loom) => (
                          <span className="font-mono text-xs text-muted-foreground">
                            {truncateHash(loom.loom_id, 8)}
                          </span>
                        ),
                      },
                      {
                        header: "Status",
                        key: "status",
                        render: (loom) => (
                          <div className="flex items-center gap-1.5">
                            {loom.active ? (
                              <CheckCircle className="h-3 w-3 text-emerald-500" />
                            ) : (
                              <XCircle className="h-3 w-3 text-muted-foreground" />
                            )}
                            <span className="text-xs">{loom.active ? "Active" : "Inactive"}</span>
                          </div>
                        ),
                      },
                      {
                        header: "Bytecode",
                        key: "bytecode",
                        hideOnMobile: true,
                        render: (loom) => (
                          <Badge variant={loom.has_bytecode ? "secondary" : "outline"} className="text-[10px]">
                            {loom.has_bytecode ? "Uploaded" : "None"}
                          </Badge>
                        ),
                      },
                      {
                        header: "Deployed",
                        key: "deployed",
                        hideOnMobile: true,
                        render: (loom) => (
                          <span className="text-xs text-muted-foreground">
                            {loom.deployed_at ? timeAgo(loom.deployed_at) : "–"}
                          </span>
                        ),
                      },
                    ]}
                    data={looms}
                    keyExtractor={(loom) => loom.loom_id}
                    onRowClick={selectFromBrowse}
                    emptyMessage="No contracts found"
                  />
                  <Pagination
                    page={browsePage}
                    hasNext={looms.length >= PAGE_SIZE}
                    onPageChange={setBrowsePage}
                    className="mt-4"
                  />
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
