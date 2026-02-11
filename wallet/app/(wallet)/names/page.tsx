"use client";

import { useState } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { useNames } from "@/hooks/use-names";
import { useNameRegister } from "@/hooks/use-name-register";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { TimeAgo } from "@/components/ui/time-ago";
import { Skeleton } from "@/components/ui/skeleton";
import { AtSign, Plus, Fingerprint, CheckCircle2, XCircle, Loader2, Search, UserPlus } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { rpcCall } from "@/lib/rpc";
import { useContactsStore } from "@/stores/contacts-store";
import { AddressDisplay } from "@/components/ui/address-display";
import { explorerAddressUrl } from "@/lib/explorer";
import { truncateAddress } from "@/lib/format";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import type { NameResolution } from "@/types";

export default function NamesPage() {
  const { activeAddress } = useWallet();
  const { data: names, isLoading } = useNames(activeAddress ?? undefined);
  const { registerName, loading } = useNameRegister();

  const { addContact, isContact } = useContactsStore();

  const [registerOpen, setRegisterOpen] = useState(false);
  const [newName, setNewName] = useState("");
  const [checking, setChecking] = useState(false);
  const [available, setAvailable] = useState<boolean | null>(null);

  // Lookup state
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
      title="NornNames"
      description="Register and manage your on-chain identity"
      action={
        <Button size="sm" onClick={() => setRegisterOpen(true)}>
          <Plus className="mr-1.5 h-3.5 w-3.5" />
          Register Name
        </Button>
      }
    >
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
                <div
                  key={n.name}
                  className="flex items-center justify-between px-4 py-3.5 hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-secondary">
                      <AtSign className="h-4 w-4 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm font-semibold">{n.name}</p>
                      <p className="text-xs text-muted-foreground">NornName</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-[10px]">
                      Active
                    </Badge>
                    <TimeAgo timestamp={n.registered_at} className="text-xs" />
                  </div>
                </div>
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
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-background border border-border">
                    <AtSign className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
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
                <p className="text-[11px] text-muted-foreground">
                  Name must be at least 3 characters
                </p>
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
            <Button variant="outline" onClick={() => setRegisterOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleRegister} disabled={loading || !newName || available !== true}>
              {loading ? (
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
    </PageContainer>
  );
}
