"use client";

import { useState } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { usePasskeyAuth } from "@/hooks/use-passkey-auth";
import { useNetwork } from "@/hooks/use-network";
import { useSettingsStore } from "@/stores/settings-store";
import { NETWORKS } from "@/lib/networks";
import { PageContainer } from "@/components/ui/page-container";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { AddressDisplay } from "@/components/ui/address-display";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { explorerAddressUrl } from "@/lib/explorer";
import { rpcCall } from "@/lib/rpc";
import type { NameResolution } from "@/types";
import { exportPrivateKeyHex, exportMnemonic, renameAccount, exportWalletBackup, changePassword } from "@/lib/wallet-manager";
import { useWalletStore } from "@/stores/wallet-store";
import { useContactsStore } from "@/stores/contacts-store";
import { isValidAddress, truncateAddress } from "@/lib/format";
import { Identicon } from "@/components/ui/identicon";
import { Settings, Network, Shield, Trash2, AlertTriangle, Key, Copy, Eye, EyeOff, Fingerprint, Pencil, Check, X, Download, Users, Plus, Loader2, AtSign } from "lucide-react";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { cn } from "@/lib/utils";

export default function SettingsPage() {
  const router = useRouter();
  const { activeAddress, activeAccount, meta, accounts } = useWallet();
  const { deleteWallet } = usePasskeyAuth();
  const { activeNetworkId, setNetwork, setCustomRpc } = useNetwork();
  const autoLockTimeout = useSettingsStore((s) => s.autoLockTimeout);
  const setAutoLockTimeout = useSettingsStore((s) => s.setAutoLockTimeout);

  const walletMeta = useWalletStore((s) => s.meta);
  const activeAccountIndex = useWalletStore((s) => s.activeAccountIndex);

  const setMeta = useWalletStore((s) => s.setMeta);

  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState("");
  const [customRpcUrl, setCustomRpcUrl] = useState("");
  const [customWsUrl, setCustomWsUrl] = useState("");
  const [editingName, setEditingName] = useState(false);
  const [nameInput, setNameInput] = useState("");
  const contacts = useContactsStore((s) => s.contacts);
  const addContact = useContactsStore((s) => s.addContact);
  const removeContact = useContactsStore((s) => s.removeContact);
  const [addContactOpen, setAddContactOpen] = useState(false);
  const [newContactAddr, setNewContactAddr] = useState("");
  const [newContactLabel, setNewContactLabel] = useState("");
  const [newContactNornName, setNewContactNornName] = useState("");
  const [resolvingContact, setResolvingContact] = useState(false);
  const [exportLoading, setExportLoading] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [exportedKey, setExportedKey] = useState<string | null>(null);
  const [exportedMnemonic, setExportedMnemonic] = useState<string | null>(null);
  const [changePasswordOpen, setChangePasswordOpen] = useState(false);
  const [currentPw, setCurrentPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [confirmPw, setConfirmPw] = useState("");
  const [changePwLoading, setChangePwLoading] = useState(false);

  const sessionPassword = useWalletStore((s) => s.sessionPassword);
  const setSessionPassword = useWalletStore((s) => s.setSessionPassword);

  const handleRename = async () => {
    const trimmed = nameInput.trim();
    if (!trimmed || !walletMeta) return;
    try {
      const updated = await renameAccount(walletMeta, activeAccountIndex, trimmed);
      setMeta(updated);
      setEditingName(false);
      toast.success("Account renamed");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Rename failed");
    }
  };

  const handleDeleteWallet = async () => {
    await deleteWallet();
    setDeleteConfirmOpen(false);
    toast.success("Wallet deleted");
    router.replace("/onboarding");
  };

  const handleSetCustomRpc = () => {
    if (!customRpcUrl) return;
    try {
      const parsed = new URL(customRpcUrl);
      if (!["http:", "https:"].includes(parsed.protocol)) {
        toast.error("RPC URL must use http or https");
        return;
      }
    } catch {
      toast.error("Invalid RPC URL");
      return;
    }
    setCustomRpc(customRpcUrl, customWsUrl || customRpcUrl.replace("http", "ws"));
    toast.success("Custom RPC set");
  };

  const handleExportPrivateKey = async () => {
    if (!walletMeta) return;
    setExportLoading(true);
    try {
      const hex = await exportPrivateKeyHex(walletMeta, activeAccountIndex);
      setExportedKey(hex);
      setShowPrivateKey(true);
      // Auto-hide after 30 seconds
      setTimeout(() => {
        setExportedKey(null);
        setShowPrivateKey(false);
      }, 30_000);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to export key");
    } finally {
      setExportLoading(false);
    }
  };

  const handleExportMnemonic = async () => {
    if (!walletMeta) return;
    setExportLoading(true);
    try {
      const mnemonic = await exportMnemonic(walletMeta, activeAccountIndex);
      setExportedMnemonic(mnemonic);
      setShowMnemonic(true);
      // Auto-hide after 60 seconds
      setTimeout(() => {
        setExportedMnemonic(null);
        setShowMnemonic(false);
      }, 60_000);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to export recovery phrase");
    } finally {
      setExportLoading(false);
    }
  };

  const handleExportBackup = async () => {
    if (!walletMeta) return;
    try {
      const json = await exportWalletBackup(walletMeta);
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `norn-wallet-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success("Backup downloaded");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to export backup");
    }
  };

  const handleChangePassword = async () => {
    if (!walletMeta || newPw !== confirmPw) return;
    setChangePwLoading(true);
    try {
      const updated = await changePassword(walletMeta, currentPw, newPw);
      setMeta(updated);
      setSessionPassword(newPw);
      setChangePasswordOpen(false);
      setCurrentPw("");
      setNewPw("");
      setConfirmPw("");
      toast.success("Password changed");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to change password");
    } finally {
      setChangePwLoading(false);
    }
  };

  const lockTimeoutOptions = [
    { label: "1 min", value: 60_000 },
    { label: "5 min", value: 300_000 },
    { label: "15 min", value: 900_000 },
    { label: "30 min", value: 1_800_000 },
    { label: "Never", value: 0 },
  ];

  return (
    <PageContainer title="Settings">
      <div className="max-w-2xl space-y-5">
        {/* Account Info */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2.5 text-sm font-medium">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Shield className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Account
            </CardTitle>
          </CardHeader>
          <CardContent>
            {activeAccount && (
              <div className="space-y-0">
                <div className="flex items-center gap-3 pb-3 mb-1 border-b border-border">
                  <Identicon address={activeAccount.address} size={36} className="shrink-0" />
                  <div className="min-w-0">
                    <p className="text-sm font-medium truncate">{activeAccount.label}</p>
                    <p className="text-xs font-mono text-muted-foreground">{truncateAddress(activeAccount.address)}</p>
                  </div>
                </div>
                <div className="flex items-center justify-between py-2.5 border-b border-border">
                  <span className="text-sm text-muted-foreground">Name</span>
                  {editingName ? (
                    <div className="flex items-center gap-1.5">
                      <Input
                        value={nameInput}
                        onChange={(e) => setNameInput(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleRename();
                          if (e.key === "Escape") setEditingName(false);
                        }}
                        className="h-9 w-48 md:h-7 md:w-40 text-sm"
                        maxLength={32}
                        autoFocus
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        onClick={handleRename}
                        disabled={!nameInput.trim()}
                        aria-label="Save name"
                      >
                        <Check className="h-3.5 w-3.5 md:h-3 md:w-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        onClick={() => setEditingName(false)}
                        aria-label="Cancel rename"
                      >
                        <X className="h-3.5 w-3.5 md:h-3 md:w-3" />
                      </Button>
                    </div>
                  ) : (
                    <button
                      className="flex items-center gap-1.5 group"
                      onClick={() => {
                        setNameInput(activeAccount.label);
                        setEditingName(true);
                      }}
                    >
                      <span className="text-sm font-medium">{activeAccount.label}</span>
                      <Pencil className="h-3 w-3 text-muted-foreground opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity" />
                    </button>
                  )}
                </div>
                <div className="flex items-center justify-between py-2.5 border-b border-border">
                  <span className="text-sm text-muted-foreground">Address</span>
                  <AddressDisplay
                    address={activeAccount.address}
                    href={explorerAddressUrl(activeAccount.address)}
                    className="text-xs"
                  />
                </div>
                <div className="flex items-center justify-between py-2.5 border-b border-border">
                  <span className="text-sm text-muted-foreground">Auth Method</span>
                  <Badge variant="secondary" className="font-normal">
                    {meta?.usesPrf ? "Passkey" : "Password"}
                  </Badge>
                </div>
                <div className="flex items-center justify-between py-2.5">
                  <span className="text-sm text-muted-foreground">Accounts</span>
                  <span className="text-sm font-medium tabular-nums">{accounts.length}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Network */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2.5 text-sm font-medium">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Network className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Network
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap gap-2">
              {Object.entries(NETWORKS).map(([id, net]) => (
                <Button
                  key={id}
                  variant="outline"
                  size="sm"
                  onClick={() => setNetwork(id)}
                  className={cn(
                    "transition-all",
                    activeNetworkId === id &&
                      "bg-accent text-accent-foreground"
                  )}
                >
                  {activeNetworkId === id && (
                    <span className="mr-1.5 inline-block h-1.5 w-1.5 rounded-full bg-foreground" />
                  )}
                  {net.name}
                </Button>
              ))}
            </div>

            <Separator />

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Custom RPC URL</Label>
              <div className="flex gap-2">
                <Input
                  value={customRpcUrl}
                  onChange={(e) => setCustomRpcUrl(e.target.value)}
                  placeholder="http://localhost:9944"
                  className="flex-1 text-sm"
                />
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleSetCustomRpc}
                  disabled={!customRpcUrl}
                >
                  Connect
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Address Book */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2.5 text-sm font-medium">
                <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                  <Users className="h-3.5 w-3.5 text-muted-foreground" />
                </div>
                Address Book
              </CardTitle>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs"
                onClick={() => setAddContactOpen(true)}
              >
                <Plus className="mr-1 h-3 w-3" />
                Add
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {contacts.length === 0 ? (
              <p className="text-xs text-muted-foreground py-4 text-center">
                No saved contacts. Add addresses you frequently send to.
              </p>
            ) : (
              <div className="space-y-0.5">
                {contacts.map((c) => (
                  <div
                    key={c.address}
                    className="flex items-center justify-between py-2 px-2 -mx-2 rounded-md hover:bg-muted/50 transition-colors group"
                  >
                    <div className="min-w-0">
                      <p className="text-sm font-medium truncate">{c.label}</p>
                      <div className="flex items-center gap-1.5">
                        <p className="font-mono text-xs text-muted-foreground">{truncateAddress(c.address)}</p>
                        {c.nornName && (
                          <span className="text-[10px] text-muted-foreground">@{c.nornName}</span>
                        )}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-9 w-9 md:h-7 md:w-7 text-muted-foreground opacity-100 md:opacity-0 md:group-hover:opacity-100"
                      onClick={() => {
                        removeContact(c.address);
                        toast.success("Contact removed");
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5 md:h-3 md:w-3" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Auto Lock */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2.5 text-sm font-medium">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Settings className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Security
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2.5">
              <Label className="text-xs text-muted-foreground">Auto-lock timeout</Label>
              <div className="flex flex-wrap gap-1.5">
                {lockTimeoutOptions.map((opt) => (
                  <Button
                    key={opt.value}
                    variant="outline"
                    size="sm"
                    onClick={() => setAutoLockTimeout(opt.value)}
                    className={cn(
                      "text-xs h-8 transition-all",
                      autoLockTimeout === opt.value &&
                        "bg-accent text-accent-foreground"
                    )}
                  >
                    {opt.label}
                  </Button>
                ))}
              </div>
            </div>

            {!meta?.usesPrf && (
              <>
                <Separator />
                <div className="space-y-2.5">
                  <Label className="text-xs text-muted-foreground">Password</Label>
                  <div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setChangePasswordOpen(true)}
                    >
                      <Key className="mr-2 h-3.5 w-3.5" />
                      Change Password
                    </Button>
                  </div>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        {/* Backup & Export */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2.5 text-sm font-medium">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Key className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Backup &amp; Export
            </CardTitle>
            <CardDescription className="text-xs">
              Export your private key or recovery phrase. Requires biometric authentication.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Recovery Phrase */}
            <div className="space-y-2">
              {showMnemonic && exportedMnemonic ? (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label className="text-xs text-muted-foreground">Recovery Phrase</Label>
                    <div className="flex gap-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        aria-label="Copy recovery phrase"
                        onClick={() => {
                          navigator.clipboard.writeText(exportedMnemonic);
                          toast.success("Copied to clipboard");
                        }}
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        aria-label="Hide recovery phrase"
                        onClick={() => {
                          setExportedMnemonic(null);
                          setShowMnemonic(false);
                        }}
                      >
                        <EyeOff className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                  <div className="grid grid-cols-3 gap-1.5 rounded-lg bg-secondary/50 p-3">
                    {exportedMnemonic.split(" ").map((word, i) => (
                      <div key={i} className="flex items-baseline gap-1">
                        <span className="text-[10px] text-muted-foreground tabular-nums w-4 text-right">{i + 1}.</span>
                        <span className="text-xs font-mono">{word}</span>
                      </div>
                    ))}
                  </div>
                  <p className="text-[11px] text-yellow-500">Auto-hides in 60 seconds. Never share this with anyone.</p>
                </div>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleExportMnemonic}
                  disabled={exportLoading}
                >
                  {exportLoading ? (
                    <span className="flex items-center gap-2">
                      <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                      Authenticating...
                    </span>
                  ) : (
                    <>
                      <Fingerprint className="mr-2 h-3.5 w-3.5" />
                      Show Recovery Phrase
                    </>
                  )}
                </Button>
              )}
            </div>

            <Separator />

            {/* Private Key */}
            <div className="space-y-2">
              {showPrivateKey && exportedKey ? (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label className="text-xs text-muted-foreground">Private Key</Label>
                    <div className="flex gap-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        aria-label="Copy private key"
                        onClick={() => {
                          navigator.clipboard.writeText(exportedKey);
                          toast.success("Copied to clipboard");
                        }}
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-9 w-9 md:h-7 md:w-7"
                        aria-label="Hide private key"
                        onClick={() => {
                          setExportedKey(null);
                          setShowPrivateKey(false);
                        }}
                      >
                        <EyeOff className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                  <div className="rounded-lg bg-secondary/50 p-3">
                    <p className="font-mono text-xs break-all select-all">{exportedKey}</p>
                  </div>
                  <p className="text-[11px] text-yellow-500">Auto-hides in 30 seconds. Never share this with anyone.</p>
                </div>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleExportPrivateKey}
                  disabled={exportLoading}
                >
                  {exportLoading ? (
                    <span className="flex items-center gap-2">
                      <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                      Authenticating...
                    </span>
                  ) : (
                    <>
                      <Fingerprint className="mr-2 h-3.5 w-3.5" />
                      Export Private Key
                    </>
                  )}
                </Button>
              )}
            </div>

            <Separator />

            {/* Wallet Backup File */}
            <div className="space-y-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleExportBackup}
              >
                <Download className="mr-2 h-3.5 w-3.5" />
                Download Wallet Backup
              </Button>
              <p className="text-[11px] text-muted-foreground">
                Downloads a JSON file containing your wallet metadata. Does not include private keys. Useful for restoring account info on another device.
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Danger Zone */}
        <Card className="border-destructive/20">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2.5 text-sm font-medium text-destructive">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Trash2 className="h-3.5 w-3.5 text-destructive" />
              </div>
              Danger Zone
            </CardTitle>
            <CardDescription className="text-xs">
              Permanently delete this wallet from this device. Ensure your recovery phrase is safely backed up before proceeding.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setDeleteConfirmOpen(true)}
              className="border-destructive/30 text-destructive hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="mr-2 h-3.5 w-3.5" />
              Delete Wallet
            </Button>
          </CardContent>
        </Card>
      </div>

      <Dialog open={deleteConfirmOpen} onOpenChange={(open) => {
        setDeleteConfirmOpen(open);
        if (!open) setDeleteConfirmText("");
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <AlertTriangle className="h-3.5 w-3.5 text-destructive" />
              </div>
              Delete Wallet
            </DialogTitle>
            <DialogDescription>
              This action is permanent and cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <div className="rounded-lg bg-destructive/5 border border-destructive/20 p-3.5 my-1">
            <p className="text-sm text-destructive/90 leading-relaxed">
              Your wallet data, including all accounts and encrypted keys, will be permanently removed from this device. If you haven&apos;t backed up your recovery phrase, you will lose access to your funds forever.
            </p>
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              Type &quot;delete&quot; to confirm
            </Label>
            <Input
              value={deleteConfirmText}
              onChange={(e) => setDeleteConfirmText(e.target.value)}
              placeholder="delete"
              className="text-sm"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirmOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDeleteWallet}
              disabled={deleteConfirmText !== "delete"}
            >
              <Trash2 className="mr-2 h-3.5 w-3.5" />
              Delete Permanently
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add Contact Dialog */}
      <Dialog open={addContactOpen} onOpenChange={(open) => {
        setAddContactOpen(open);
        if (!open) { setNewContactAddr(""); setNewContactLabel(""); setNewContactNornName(""); setResolvingContact(false); }
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add Contact</DialogTitle>
            <DialogDescription>
              Enter an address or NornName to save as a contact.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Label</Label>
              <Input
                value={newContactLabel}
                onChange={(e) => setNewContactLabel(e.target.value)}
                placeholder="e.g. Alice, Treasury"
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Address or NornName</Label>
              <div className="relative">
                <Input
                  value={newContactAddr}
                  onChange={async (e) => {
                    const val = e.target.value;
                    setNewContactAddr(val);
                    setNewContactNornName("");

                    // If it looks like a NornName (not an address, >= 3 chars), try resolving
                    if (!val.startsWith("0x") && val.length >= 3) {
                      setResolvingContact(true);
                      try {
                        const result = await rpcCall<NameResolution | null>("norn_resolveName", [val]);
                        if (result?.owner) {
                          setNewContactAddr(result.owner);
                          setNewContactNornName(val);
                          if (!newContactLabel.trim()) {
                            setNewContactLabel(val);
                          }
                        }
                      } catch {
                        // ignore
                      } finally {
                        setResolvingContact(false);
                      }
                    }
                  }}
                  placeholder="0x... or NornName"
                  className="font-mono text-xs pr-8"
                />
                {resolvingContact && (
                  <Loader2 className="absolute right-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground animate-spin" />
                )}
              </div>
              {newContactNornName && (
                <p className="text-[11px] text-muted-foreground flex items-center gap-1">
                  <AtSign className="h-3 w-3" />
                  Resolved from <span className="font-medium text-foreground">{newContactNornName}</span>
                </p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddContactOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                if (!isValidAddress(newContactAddr)) {
                  toast.error("Invalid address format");
                  return;
                }
                addContact(
                  newContactAddr,
                  newContactLabel.trim() || newContactNornName || truncateAddress(newContactAddr),
                  newContactNornName || undefined
                );
                setAddContactOpen(false);
                toast.success("Contact added");
              }}
              disabled={!newContactAddr || !newContactLabel.trim() || resolvingContact}
            >
              <Plus className="mr-2 h-3.5 w-3.5" />
              Add Contact
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {/* Change Password Dialog */}
      <Dialog open={changePasswordOpen} onOpenChange={(open) => {
        setChangePasswordOpen(open);
        if (!open) { setCurrentPw(""); setNewPw(""); setConfirmPw(""); }
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-full bg-secondary">
                <Key className="h-3.5 w-3.5 text-muted-foreground" />
              </div>
              Change Password
            </DialogTitle>
            <DialogDescription>
              Enter your current password and choose a new one.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Current Password</Label>
              <Input
                type="password"
                value={currentPw}
                onChange={(e) => setCurrentPw(e.target.value)}
                placeholder="Enter current password"
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">New Password</Label>
              <Input
                type="password"
                value={newPw}
                onChange={(e) => setNewPw(e.target.value)}
                placeholder="Enter new password"
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">Confirm New Password</Label>
              <Input
                type="password"
                value={confirmPw}
                onChange={(e) => setConfirmPw(e.target.value)}
                placeholder="Confirm new password"
                onKeyDown={(e) => {
                  if (e.key === "Enter" && newPw && newPw === confirmPw && currentPw) {
                    handleChangePassword();
                  }
                }}
              />
              {confirmPw && newPw !== confirmPw && (
                <p className="text-[11px] text-destructive">Passwords do not match</p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setChangePasswordOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleChangePassword}
              disabled={changePwLoading || !currentPw || !newPw || newPw !== confirmPw}
            >
              {changePwLoading ? (
                <span className="flex items-center gap-2">
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  Changing...
                </span>
              ) : (
                "Change Password"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  );
}
