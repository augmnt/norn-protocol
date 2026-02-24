import { useState, useEffect } from "react";
import { Settings2, Plus } from "lucide-react";
import { toast } from "sonner";
import { buildNameRecordUpdate } from "@norn-protocol/sdk";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { rpc } from "@/lib/rpc";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

const RECORD_KEYS = [
  "avatar",
  "url",
  "description",
  "twitter",
  "github",
  "email",
  "discord",
] as const;

const MAX_VALUE_LENGTH = 256;

export function NameRecords() {
  const [records, setRecords] = useState<Record<string, string>>({});
  const [recordsLoading, setRecordsLoading] = useState(true);
  const [selectedKey, setSelectedKey] = useState<string>(RECORD_KEYS[0]);
  const [value, setValue] = useState("");
  const [loading, setLoading] = useState(false);

  const activeWallet = useWalletStore((s) => s.activeWallet);
  const params = useNavigationStore((s) => s.params);

  const name = (params.name as string) ?? "";

  useEffect(() => {
    if (!name) return;
    loadRecords();
  }, [name]);

  const loadRecords = async () => {
    setRecordsLoading(true);
    try {
      const result = await rpc.getNameRecords(name);
      setRecords(result);
    } catch {
      // Silently handle — empty records is fine
    } finally {
      setRecordsLoading(false);
    }
  };

  const isValid =
    name.length > 0 &&
    selectedKey.length > 0 &&
    value.length > 0 &&
    value.length <= MAX_VALUE_LENGTH &&
    !loading;

  const handleSubmit = async () => {
    if (!activeWallet || !isValid) return;

    setLoading(true);
    try {
      const updateHex = buildNameRecordUpdate(activeWallet, {
        name,
        key: selectedKey,
        value,
      });
      const result = await rpc.setNameRecord(
        name,
        selectedKey,
        value,
        activeWallet.addressHex,
        updateHex,
      );
      if (!result.success) {
        toast.error(result.reason ?? "Failed to update record");
        return;
      }
      toast.success(`Record "${selectedKey}" updated`);
      setValue("");
      await loadRecords();
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to update record",
      );
    } finally {
      setLoading(false);
    }
  };

  const recordEntries = Object.entries(records);

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 scrollbar-thin">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold">Name Records</h2>
          <p className="text-sm text-muted-foreground">
            View and manage records for <span className="font-medium text-foreground">{name}</span>.
          </p>
        </div>

        {/* Avatar preview */}
        {!recordsLoading && records.avatar && (
          <div className="flex justify-center">
            <img
              src={records.avatar}
              alt=""
              className="h-16 w-16 rounded-full object-cover border border-border"
              onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
            />
          </div>
        )}

        {/* Existing records */}
        <div>
          <h3 className="mb-2 text-sm font-medium">Current Records</h3>
          {recordsLoading ? (
            <div className="flex justify-center py-4">
              <Spinner size="sm" />
            </div>
          ) : recordEntries.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-6 text-muted-foreground animate-fade-in">
              <Settings2 className="h-5 w-5" />
              <p className="text-sm">No records set</p>
            </div>
          ) : (
            <Card>
              <CardContent className="divide-y divide-border p-0">
                {recordEntries.map(([key, val], i) => (
                  <div
                    key={key}
                    className="flex flex-col gap-0.5 px-4 py-2.5 animate-slide-in"
                    style={{ animationDelay: `${i * 50}ms`, animationFillMode: "backwards" }}
                  >
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {key}
                    </span>
                    <span className="break-all text-sm font-medium">
                      {val}
                    </span>
                  </div>
                ))}
              </CardContent>
            </Card>
          )}
        </div>

        {/* Add / update record form */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Set Record</h3>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Key</label>
            <select
              value={selectedKey}
              onChange={(e) => setSelectedKey(e.target.value)}
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-norn/50 focus-visible:border-norn/50"
            >
              {RECORD_KEYS.map((k) => (
                <option key={k} value={k} className="bg-background text-foreground">
                  {k}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Value</label>
            <Input
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={`Enter ${selectedKey} value`}
              maxLength={MAX_VALUE_LENGTH}
            />
            <p className="text-xs text-muted-foreground">
              {value.length}/{MAX_VALUE_LENGTH} characters
            </p>
          </div>

          <Button
            className="w-full"
            disabled={!isValid}
            onClick={handleSubmit}
          >
            {loading ? (
              <Spinner size="sm" />
            ) : (
              <>
                <Plus className="h-4 w-4" />
                {records[selectedKey] ? "Update Record" : "Add Record"}
              </>
            )}
          </Button>
        </div>
      </div>

      <BottomNav />
    </div>
  );
}
