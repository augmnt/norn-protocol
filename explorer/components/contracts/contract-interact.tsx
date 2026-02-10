"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { Play, Search, Loader2 } from "lucide-react";
import type { QueryResult, ExecutionResult } from "@/types";

interface ContractInteractProps {
  loomId: string;
}

export function ContractInteract({ loomId }: ContractInteractProps) {
  const [queryInput, setQueryInput] = useState("");
  const [querySender, setQuerySender] = useState("");
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [queryLoading, setQueryLoading] = useState(false);
  const [queryError, setQueryError] = useState<string | null>(null);

  const [execInput, setExecInput] = useState("");
  const [execSender, setExecSender] = useState("");
  const [execResult, setExecResult] = useState<ExecutionResult | null>(null);
  const [execLoading, setExecLoading] = useState(false);
  const [execError, setExecError] = useState<string | null>(null);

  const handleQuery = async () => {
    setQueryLoading(true);
    setQueryError(null);
    setQueryResult(null);
    try {
      const inputHex = queryInput.startsWith("0x")
        ? strip0x(queryInput)
        : queryInput;
      const senderHex = querySender ? strip0x(querySender) : "0".repeat(40);
      const result = await rpcCall<QueryResult>("norn_queryLoom", [
        strip0x(loomId),
        inputHex,
        senderHex,
      ]);
      setQueryResult(result);
    } catch (e) {
      setQueryError(e instanceof Error ? e.message : "Query failed");
    } finally {
      setQueryLoading(false);
    }
  };

  const handleExecute = async () => {
    setExecLoading(true);
    setExecError(null);
    setExecResult(null);
    try {
      const inputHex = execInput.startsWith("0x")
        ? strip0x(execInput)
        : execInput;
      const senderHex = execSender ? strip0x(execSender) : "0".repeat(40);
      const result = await rpcCall<ExecutionResult>("norn_executeLoom", [
        strip0x(loomId),
        inputHex,
        senderHex,
      ]);
      setExecResult(result);
    } catch (e) {
      setExecError(e instanceof Error ? e.message : "Execution failed");
    } finally {
      setExecLoading(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm font-medium">
          Contract Interaction
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="query">
          <TabsList>
            <TabsTrigger value="query">Query</TabsTrigger>
            <TabsTrigger value="execute">Execute</TabsTrigger>
          </TabsList>

          <TabsContent value="query" className="space-y-3 mt-3">
            <div>
              <label className="text-xs text-muted-foreground uppercase tracking-wider mb-1 block">
                Input (hex)
              </label>
              <textarea
                value={queryInput}
                onChange={(e) => setQueryInput(e.target.value)}
                placeholder='JSON message as hex, e.g. {"get_count":{}}'
                className="w-full rounded-md border bg-transparent px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring min-h-[80px] resize-y"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground uppercase tracking-wider mb-1 block">
                Sender Address (optional)
              </label>
              <input
                type="text"
                value={querySender}
                onChange={(e) => setQuerySender(e.target.value)}
                placeholder="0x..."
                className="w-full rounded-md border bg-transparent px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              />
            </div>
            <Button
              size="sm"
              onClick={handleQuery}
              disabled={queryLoading || !queryInput.trim()}
              className="gap-1.5"
            >
              {queryLoading ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Search className="h-3.5 w-3.5" />
              )}
              Query
            </Button>

            <ResultDisplay
              result={queryResult}
              error={queryError}
            />
          </TabsContent>

          <TabsContent value="execute" className="space-y-3 mt-3">
            <div>
              <label className="text-xs text-muted-foreground uppercase tracking-wider mb-1 block">
                Input (hex)
              </label>
              <textarea
                value={execInput}
                onChange={(e) => setExecInput(e.target.value)}
                placeholder='JSON message as hex, e.g. {"increment":{}}'
                className="w-full rounded-md border bg-transparent px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring min-h-[80px] resize-y"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground uppercase tracking-wider mb-1 block">
                Sender Address
              </label>
              <input
                type="text"
                value={execSender}
                onChange={(e) => setExecSender(e.target.value)}
                placeholder="0x..."
                className="w-full rounded-md border bg-transparent px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              />
            </div>
            <Button
              size="sm"
              onClick={handleExecute}
              disabled={execLoading || !execInput.trim()}
              className="gap-1.5"
            >
              {execLoading ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Play className="h-3.5 w-3.5" />
              )}
              Execute
            </Button>

            <ResultDisplay
              result={execResult}
              error={execError}
            />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}

function ResultDisplay({
  result,
  error,
}: {
  result: QueryResult | ExecutionResult | null;
  error: string | null;
}) {
  if (error) {
    return (
      <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {error}
      </div>
    );
  }

  if (!result) return null;

  return (
    <div className="space-y-2 rounded-md border px-3 py-3">
      <div className="flex items-center gap-2">
        <Badge variant={result.success ? "default" : "destructive"}>
          {result.success ? "Success" : "Failed"}
        </Badge>
        <span className="text-xs text-muted-foreground">
          Gas: <span className="font-mono">{result.gas_used.toLocaleString()}</span>
        </span>
      </div>
      {result.reason && (
        <p className="text-sm text-muted-foreground">{result.reason}</p>
      )}
      {result.output_hex && (
        <div>
          <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
            Output
          </p>
          <pre className="rounded bg-muted px-2 py-1.5 text-xs font-mono overflow-x-auto">
            {result.output_hex}
          </pre>
        </div>
      )}
      {result.logs.length > 0 && (
        <div>
          <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
            Logs
          </p>
          <div className="space-y-1">
            {result.logs.map((log, i) => (
              <pre
                key={i}
                className="rounded bg-muted px-2 py-1 text-xs font-mono"
              >
                {log}
              </pre>
            ))}
          </div>
        </div>
      )}
      {result.events.length > 0 && (
        <div>
          <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
            Events ({result.events.length})
          </p>
          <div className="space-y-1">
            {result.events.map((evt, i) => (
              <pre
                key={i}
                className="rounded bg-muted px-2 py-1 text-xs font-mono overflow-x-auto"
              >
                {JSON.stringify(evt, null, 2)}
              </pre>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
