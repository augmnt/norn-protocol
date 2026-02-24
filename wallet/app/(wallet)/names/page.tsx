"use client";

import { useState } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { useNames } from "@/hooks/use-names";
import { useNameRegister } from "@/hooks/use-name-register";
import { useNameTransfer } from "@/hooks/use-name-transfer";
import { useNameRecords, useSetNameRecord } from "@/hooks/use-name-records";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { TimeAgo } from "@/components/ui/time-ago";
import { Skeleton } from "@/components/ui/skeleton";
import {
  AtSign, Plus, Fingerprint, CheckCircle2, XCircle, Loader2,
  Search, UserPlus, ArrowRightLeft, Settings2,
} from "lucide-react";
import { NnsAvatar } from "@/components/ui/nns-avatar";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
  DialogDescription, DialogFooter,
} from "@/components/ui/dialog";
import { rpcCall } from "@/lib/rpc";
import { useContactsStore } from "@/stores/contacts-store";
import { AddressDisplay } from "@/components/ui/address-display";
import { explorerAddressUrl } from "@/lib/explorer";
import { truncateAddress } from "@/lib/format";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import type { NameResolution } from "@/types";

const RECORD_KEYS = ["avatar", "url", "description", "twitter", "github", "email", "discord"];

export default function NamesPage() {
  const { activeAddress } = useWallet();
  const { data: names, isLoading } = useNames(activeAddress ?? undefined);
  const { registerName, loading: registerLoading } = useNameRegister();
  const { transferName, loading: transferLoading } = useNameTransfer();
  const { setNameRecord, loading: recordLoading } = useSetNameRecord();

  const { addContact, isContact } = useContactsStore();

  // Register dialog
  const [registerOpen, setRegisterOpen] = useState(false);
  const [newName, setNewName] = useState("");
  const [checking, setChecking] = useState(false);
  const [available, setAvailable] = useState<boolean | null>(null);

  // Transfer dialog
  const [transferOpen, setTransferOpen] = useState(false);
  const [transferTargetName, setTransferTargetName] = useState("");
  const [transferTo, setTransferTo] = useState("");

  // Records dialog
  const [recordsOpen, setRecordsOpen] = useState(false);
  const [recordsTargetName, setRecordsTargetName] = useState("");
  const [recordKey, setRecordKey] = useState("avatar");
  const [recordValue, setRecordValue] = useState("");

  // Lookup
  const [lookupName, setLookupName] = useState("");
  const [lookupLoading, setLookupLoading] = useState(false);
  const [lookupResult, setLookupResult] = useState<NameResolution | null>(null);
  const [lookupNotFound, setLookupNotFound] = useState(false);

  const checkAvailability = async (name: string) => {
    setNewName(name);
    setAvailable(null);
    if (name.length < 3) return;
    setChecking(true);
    try {
      const result = await rpcCall<NameResolution | null>("norn_resolveName", [name]);
      setAvailable(!result);
    } catch {
      setAvailable(null);
    } finally {
      setChecking(false);
    }
  };

  const handleRegister = async () => {
    try {
      await registerName(newName);
      setRegisterOpen(false);
      setNewName("");
      toast.success(`Name "${newName}" registered`);
    } catch {
      toast.error("Registration failed");
    }
  };

  const handleTransfer = async () => {
    try {
      await transferName(transferTargetName, transferTo);
      setTransferOpen(false);
      setTransferTo("");
      toast.success(`Name "${transferTargetName}" transfer submitted`);
    } catch {
      toast.error("Transfer failed");
    }
  };

  const handleSetRecord = async () => {
    try {
      await setNameRecord(recordsTargetName, recordKey, recordValue);
      setRecordValue("");
      toast.success(`Record "${recordKey}" updated`);
    } catch {
      toast.error("Update failed");
    }
  };

  const handleLookup = async () => {
    if (lookupName.length < 3) return;
    setLookupLoading(true);
    setLookupResult(null);
    setLookupNotFound(false);
    try {
      const result = await rpcCall<NameResolution | null>("norn_resolveName", [lookupName]);
      if (result) {
        setLookupResult(result);
      } else {
        setLookupNotFound(true);
      }
    } catch {
      toast.error("Lookup failed");
    } finally {
      setLookupLoading(false);
    }
  };

  return (
    <PageContainer
      title="Norn Name Service"
      description="Register, manage, and transfer your on-chain identity"
      action={
        <Button size="sm" onClick={() => setRegisterOpen(true)}>
          <Plus className="mr-1.5 h-3.5 w-3.5" />
          Register Name
        </Button>
      }
    >
      <div className="space-y-4">
      <Card>
        <CardContent className="p-0">
          {isLoading ? (
            <div className="p-4 space-y-1">
              {Array.from({ length: 2 }).map((_, i) => (
                <div key={i} className="flex items-center gap-3 rounded-lg px-3 py-4">
                  <Skeleton className="h-10 w-10 rounded-full shrink-0" />
                  <div className="flex-1 space-y-1.5">
                    <Skeleton className="h-4 w-28" />
                    <Skeleton className="h-3 w-20" />
                  </div>
                  <Skeleton className="h-3 w-16" />
                </div>
              ))}
            </div>
          ) : !names || names.length === 0 ? (
            <EmptyState
              icon={AtSign}
              title="No names registered"
              description="Claim your unique NornName for just 1 NORN"
              className="py-16"
            />
          ) : (
            <div className="divide-y divide-border">
              {names.map((n) => (
                <NameListItem
                  key={n.name}
                  name={n.name}
                  address={activeAddress!}
                  onRecords={() => {
                    setRecordsTargetName(n.name);
                    setRecordKey("avatar");
                    setRecordValue("");
                    setRecordsOpen(true);
                  }}
                  onTransfer={() => {
                    setTransferTargetName(n.name);
                    setTransferTo("");
                    setTransferOpen(true);
                  }}
                  registeredAt={n.registered_at}
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Lookup */}
      <Card>
        <CardContent className="pt-5 space-y-3">
          <div className="flex items-center gap-2">
            <Search className="h-3.5 w-3.5 text-muted-foreground" />
            <p className="text-sm font-medium">Lookup</p>
          </div>
          <p className="text-xs text-muted-foreground">
            Find someone by their NornName and add them to your contacts.
          </p>
          <div className="flex gap-2">
            <Input
              value={lookupName}
              onChange={(e) => {
                setLookupName(e.target.value.toLowerCase());
                setLookupResult(null);
                setLookupNotFound(false);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleLookup();
              }}
              placeholder="Enter a NornName"
              className="text-sm h-9"
            />
            <Button
              size="sm"
              variant="outline"
              onClick={handleLookup}
              disabled={lookupName.length < 3 || lookupLoading}
              className="shrink-0 h-9"
            >
              {lookupLoading ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Search className="h-3.5 w-3.5" />
              )}
            </Button>
          </div>

          {lookupNotFound && (
            <div className="flex items-center gap-2 rounded-lg border border-border bg-secondary/50 px-3 py-2.5">
              <XCircle className="h-4 w-4 text-muted-foreground shrink-0" />
              <p className="text-xs text-muted-foreground">
                No address found for <span className="font-medium text-foreground">{lookupName}</span>
              </p>
            </div>
          )}

          {lookupResult && (
            <div className="rounded-lg border border-border bg-secondary/50 px-3 py-3 space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <NnsAvatar
                    address={lookupResult.owner}
                    avatarUrl={lookupResult.records?.avatar}
                    size={32}
                  />
                  <div>
                    <p className="text-sm font-medium">{lookupName}</p>
                    <p className="font-mono text-xs text-muted-foreground">{truncateAddress(lookupResult.owner)}</p>
                  </div>
                </div>
                {lookupResult.owner.toLowerCase() === activeAddress?.toLowerCase() ? (
                  <Badge variant="secondary" className="text-[10px]">You</Badge>
                ) : isContact(lookupResult.owner) ? (
                  <Badge variant="secondary" className="text-[10px]">Saved</Badge>
                ) : (
                  <Button
                    size="sm"
                    variant="outline"
                    className="h-7 text-xs"
                    onClick={() => {
                      addContact(lookupResult.owner, lookupName, lookupName);
                      toast.success(`Added ${lookupName} to contacts`);
                    }}
                  >
                    <UserPlus className="mr-1.5 h-3 w-3" />
                    Add
                  </Button>
                )}
              </div>
              {lookupResult.records && Object.keys(lookupResult.records).length > 0 && (
                <div className="border-t border-border pt-2 mt-2 space-y-1">
                  {Object.entries(lookupResult.records).sort(([a], [b]) => a.localeCompare(b)).map(([k, v]) => (
                    <div key={k} className="flex items-center justify-between text-xs">
                      <span className="text-muted-foreground uppercase tracking-wider">{k}</span>
                      <span className="font-mono text-foreground truncate max-w-[200px]">{v}</span>
                    </div>
                  ))}
                </div>
              )}
              <div className="pt-1">
                <AddressDisplay
                  address={lookupResult.owner}
                  href={explorerAddressUrl(lookupResult.owner)}
                  className="text-xs"
                />
              </div>
            </div>
          )}
        </CardContent>
      </Card>
      </div>

      {/* Register Dialog */}
      <Dialog open={registerOpen} onOpenChange={(open) => {
        setRegisterOpen(open);
        if (!open) { setNewName(""); setAvailable(null); }
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <AtSign className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Register NornName
            </DialogTitle>
            <DialogDescription>
              Choose a unique name that will be linked to your address on-chain.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-1">
            <div className="space-y-2">
              <Label>Name</Label>
              <div className="relative">
                <Input
                  value={newName}
                  onChange={(e) => checkAvailability(e.target.value.toLowerCase())}
                  placeholder="myname"
                  className={cn(
                    "pr-10",
                    available === true && "border-green-500/50 focus-visible:ring-green-500/30",
                    available === false && "border-destructive/50 focus-visible:ring-destructive/30"
                  )}
                />
                {newName.length >= 3 && (
                  <div className="absolute right-3 top-1/2 -translate-y-1/2">
                    {checking ? (
                      <Loader2 className="h-4 w-4 text-muted-foreground animate-spin" />
                    ) : available === true ? (
                      <CheckCircle2 className="h-4 w-4 text-green-500" />
                    ) : available === false ? (
                      <XCircle className="h-4 w-4 text-destructive" />
                    ) : null}
                  </div>
                )}
              </div>
              {newName.length > 0 && newName.length < 3 && (
                <p className="text-[11px] text-muted-foreground">Name must be at least 3 characters</p>
              )}
              {available === true && (
                <p className="text-[11px] text-green-500 flex items-center gap-1">
                  <span className="inline-block h-1.5 w-1.5 rounded-full bg-green-500" />
                  Available
                </p>
              )}
              {available === false && (
                <p className="text-[11px] text-destructive flex items-center gap-1">
                  <span className="inline-block h-1.5 w-1.5 rounded-full bg-destructive" />
                  Already taken
                </p>
              )}
            </div>
            <div className="rounded-lg bg-secondary/50 px-3 py-2">
              <p className="text-xs text-muted-foreground">
                Registration fee: <span className="text-foreground font-medium">1 NORN</span>
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRegisterOpen(false)}>Cancel</Button>
            <Button onClick={handleRegister} disabled={registerLoading || !newName || available !== true}>
              {registerLoading ? (
                <span className="flex items-center gap-2">
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Registering...
                </span>
              ) : (
                <>
                  <Fingerprint className="mr-2 h-4 w-4" />
                  Register
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Transfer Dialog */}
      <Dialog open={transferOpen} onOpenChange={(open) => {
        setTransferOpen(open);
        if (!open) setTransferTo("");
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <ArrowRightLeft className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Transfer Name
            </DialogTitle>
            <DialogDescription>
              Transfer ownership of <span className="font-medium text-foreground">{transferTargetName}</span> to another address. This cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-1">
            <div className="space-y-2">
              <Label>Recipient Address</Label>
              <Input
                value={transferTo}
                onChange={(e) => setTransferTo(e.target.value)}
                placeholder="0x..."
                className="font-mono text-sm"
              />
            </div>
            <div className="rounded-lg bg-secondary/50 px-3 py-2">
              <p className="text-xs text-muted-foreground">
                Transfer fee: <span className="text-foreground font-medium">Free</span>
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTransferOpen(false)}>Cancel</Button>
            <Button
              variant="destructive"
              onClick={handleTransfer}
              disabled={transferLoading || !transferTo || transferTo.length < 40}
            >
              {transferLoading ? (
                <span className="flex items-center gap-2">
                  <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Transferring...
                </span>
              ) : (
                <>
                  <ArrowRightLeft className="mr-2 h-4 w-4" />
                  Transfer
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Records Dialog */}
      <Dialog open={recordsOpen} onOpenChange={(open) => {
        setRecordsOpen(open);
        if (!open) { setRecordValue(""); }
      }}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Settings2 className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              NNS Records
            </DialogTitle>
            <DialogDescription>
              Manage records for <span className="font-medium text-foreground">{recordsTargetName}</span>
            </DialogDescription>
          </DialogHeader>
          <NameRecordsContent
            name={recordsTargetName}
            recordKey={recordKey}
            setRecordKey={setRecordKey}
            recordValue={recordValue}
            setRecordValue={setRecordValue}
            onSubmit={handleSetRecord}
            loading={recordLoading}
          />
        </DialogContent>
      </Dialog>
    </PageContainer>
  );
}

function NameRecordsContent({
  name, recordKey, setRecordKey, recordValue, setRecordValue, onSubmit, loading,
}: {
  name: string;
  recordKey: string;
  setRecordKey: (k: string) => void;
  recordValue: string;
  setRecordValue: (v: string) => void;
  onSubmit: () => void;
  loading: boolean;
}) {
  const { data: records, isLoading } = useNameRecords(name || undefined);

  return (
    <div className="space-y-4 py-1">
      {!isLoading && records?.avatar && (
        <div className="flex justify-center">
          <img
            src={records.avatar}
            alt=""
            className="h-16 w-16 rounded-full object-cover border border-border"
            onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
          />
        </div>
      )}
      {isLoading ? (
        <div className="space-y-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-48" />
        </div>
      ) : records && Object.keys(records).length > 0 ? (
        <div className="rounded-lg border border-border divide-y divide-border">
          {Object.entries(records).sort(([a], [b]) => a.localeCompare(b)).map(([k, v]) => (
            <div key={k} className="flex items-center justify-between px-3 py-2.5">
              <span className="text-xs uppercase tracking-wider text-muted-foreground w-24">{k}</span>
              <span className="text-sm font-mono truncate max-w-[250px]">{v}</span>
            </div>
          ))}
        </div>
      ) : (
        <div className="rounded-lg bg-secondary/50 px-3 py-4 text-center">
          <p className="text-xs text-muted-foreground">No records set</p>
        </div>
      )}

      <div className="border-t border-border pt-4 space-y-3">
        <p className="text-sm font-medium">Add / Update Record</p>
        <div className="grid grid-cols-[120px_1fr] gap-2">
          <select
            value={recordKey}
            onChange={(e) => setRecordKey(e.target.value)}
            className="h-9 rounded-md border border-border bg-background px-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
          >
            {RECORD_KEYS.map((k) => (
              <option key={k} value={k}>{k}</option>
            ))}
          </select>
          <Input
            value={recordValue}
            onChange={(e) => setRecordValue(e.target.value)}
            placeholder="Value"
            className="text-sm h-9"
            maxLength={256}
          />
        </div>
        <Button
          size="sm"
          onClick={onSubmit}
          disabled={loading || !recordValue}
          className="w-full"
        >
          {loading ? (
            <span className="flex items-center gap-2">
              <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Saving...
            </span>
          ) : (
            "Save Record"
          )}
        </Button>
      </div>
    </div>
  );
}

function NameListItem({
  name, address, onRecords, onTransfer, registeredAt,
}: {
  name: string;
  address: string;
  onRecords: () => void;
  onTransfer: () => void;
  registeredAt: number;
}) {
  const { data: records } = useNameRecords(name);

  return (
    <div className="flex items-center justify-between px-4 py-3.5 hover:bg-muted/50 transition-colors">
      <div className="flex items-center gap-3">
        <NnsAvatar
          address={address}
          avatarUrl={records?.avatar}
          size={40}
          className="shrink-0"
        />
        <div>
          <p className="text-sm font-semibold">{name}</p>
          <p className="text-xs text-muted-foreground">NNS</p>
        </div>
      </div>
      <div className="flex items-center gap-1.5">
        <Button
          size="sm"
          variant="ghost"
          className="h-7 w-7 p-0"
          title="Records"
          onClick={onRecords}
        >
          <Settings2 className="h-3.5 w-3.5" />
        </Button>
        <Button
          size="sm"
          variant="ghost"
          className="h-7 w-7 p-0"
          title="Transfer"
          onClick={onTransfer}
        >
          <ArrowRightLeft className="h-3.5 w-3.5" />
        </Button>
        <TimeAgo timestamp={registeredAt} className="text-xs ml-1" />
      </div>
    </div>
  );
}
