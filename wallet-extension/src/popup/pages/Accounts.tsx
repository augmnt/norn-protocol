import { useState } from "react";
import { Plus, Download, Terminal, Check, Trash2, Pencil } from "lucide-react";
import { toast } from "sonner";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { deleteAccount } from "@/lib/keystore";
import { truncateAddress } from "@/lib/format";
import { Header } from "../components/layout/Header";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent } from "../components/ui/card";
import { Spinner } from "../components/ui/spinner";

export function Accounts() {
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [password, setPassword] = useState("");
  const [creating, setCreating] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [deletePassword, setDeletePassword] = useState("");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const accounts = useWalletStore((s) => s.accounts);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);
  const createNewAccount = useWalletStore((s) => s.createNewAccount);
  const switchAccount = useWalletStore((s) => s.switchAccount);
  const renameAccount = useWalletStore((s) => s.renameAccount);
  const refreshAccounts = useWalletStore((s) => s.refreshAccounts);
  const navigate = useNavigationStore((s) => s.navigate);

  const handleCreate = async () => {
    if (!newName.trim() || !password) return;
    setCreating(true);
    try {
      await createNewAccount(newName.trim(), password);
      setShowCreate(false);
      setNewName("");
      setPassword("");
      toast.success("Account created");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to create account",
      );
    } finally {
      setCreating(false);
    }
  };

  const handleSwitch = async (accountId: string) => {
    if (accountId === activeAccountId) return;
    try {
      await switchAccount(accountId);
      navigate("dashboard");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to switch account",
      );
    }
  };

  const handleDelete = async (accountId: string) => {
    if (!deletePassword) return;
    try {
      await deleteAccount(accountId, deletePassword);
      await refreshAccounts();
      setDeletingId(null);
      setDeletePassword("");
      toast.success("Account deleted");

      if (accounts.length <= 1) {
        navigate("welcome");
      }
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to delete account",
      );
    }
  };

  const handleRename = async (accountId: string) => {
    const trimmed = renameValue.trim();
    if (!trimmed) return;
    try {
      await renameAccount(accountId, trimmed);
      setRenamingId(null);
      setRenameValue("");
      toast.success("Account renamed");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Failed to rename account",
      );
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4 scrollbar-thin">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Accounts</h2>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowCreate(!showCreate)}
          >
            <Plus className="h-4 w-4" />
            New
          </Button>
        </div>

        {showCreate && (
          <Card className="animate-slide-in">
            <CardContent className="space-y-2 p-3">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="Account name"
                className="h-8 text-xs"
              />
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Password"
                className="h-8 text-xs"
              />
              <div className="flex gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  className="flex-1"
                  onClick={() => setShowCreate(false)}
                >
                  Cancel
                </Button>
                <Button
                  size="sm"
                  className="flex-1"
                  disabled={!newName.trim() || !password || creating}
                  onClick={handleCreate}
                >
                  {creating ? <Spinner size="sm" /> : "Create"}
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        <div className="space-y-2">
          {accounts.map((account, i) => (
            <div
              key={account.id}
              className="animate-slide-in"
              style={{ animationDelay: `${i * 50}ms`, animationFillMode: "backwards" }}
            >
              <button
                onClick={() => handleSwitch(account.id)}
                className={`flex w-full items-center gap-3 rounded-lg border p-3 text-left transition-colors duration-150 hover:bg-accent ${
                  account.id === activeAccountId
                    ? "border-l-2 border-l-norn"
                    : ""
                }`}
              >
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-norn/20 text-xs font-bold text-norn">
                  {account.name.charAt(0).toUpperCase()}
                </div>
                <div className="flex flex-1 flex-col">
                  <span className="text-sm font-medium">{account.name}</span>
                  <span className="font-mono text-xs text-muted-foreground">
                    {truncateAddress(account.address)}
                  </span>
                </div>
                {account.id === activeAccountId && (
                  <Check className="h-4 w-4 text-emerald-400" />
                )}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    if (renamingId === account.id) {
                      setRenamingId(null);
                      setRenameValue("");
                    } else {
                      setRenamingId(account.id);
                      setRenameValue(account.name);
                      setDeletingId(null);
                    }
                  }}
                  className="rounded p-1 text-muted-foreground transition-colors duration-150 hover:text-norn"
                >
                  <Pencil className="h-3.5 w-3.5" />
                </button>
                {accounts.length > 1 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      if (deletingId === account.id) {
                        setDeletingId(null);
                        setDeletePassword("");
                      } else {
                        setDeletingId(account.id);
                        setDeletePassword("");
                        setRenamingId(null);
                      }
                    }}
                    className="rounded p-1 text-muted-foreground transition-colors duration-150 hover:text-destructive"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                )}
              </button>

              {renamingId === account.id && (
                <div className="mt-1 flex gap-2 animate-slide-in pl-11">
                  <Input
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    placeholder="New name"
                    className="h-7 text-xs"
                    maxLength={32}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleRename(account.id);
                      if (e.key === "Escape") { setRenamingId(null); setRenameValue(""); }
                    }}
                    autoFocus
                  />
                  <Button
                    size="sm"
                    className="h-7 shrink-0"
                    disabled={!renameValue.trim() || renameValue.trim() === account.name}
                    onClick={() => handleRename(account.id)}
                  >
                    Save
                  </Button>
                </div>
              )}

              {deletingId === account.id && (
                <div className="mt-1 flex gap-2 animate-slide-in pl-11">
                  <Input
                    type="password"
                    value={deletePassword}
                    onChange={(e) => setDeletePassword(e.target.value)}
                    placeholder="Password to confirm"
                    className="h-7 text-xs"
                  />
                  <Button
                    variant="destructive"
                    size="sm"
                    className="h-7 shrink-0"
                    disabled={!deletePassword}
                    onClick={() => handleDelete(account.id)}
                  >
                    Delete
                  </Button>
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="mt-1 space-y-2">
          <button
            onClick={() => navigate("import-wallet")}
            className="flex w-full items-center gap-2 rounded-lg border p-3 text-sm transition-colors duration-150 hover:bg-accent"
          >
            <Download className="h-4 w-4 text-norn" />
            <span>Import Private Key</span>
          </button>
          <button
            onClick={() => navigate("import-cli")}
            className="flex w-full items-center gap-2 rounded-lg border p-3 text-sm transition-colors duration-150 hover:bg-accent"
          >
            <Terminal className="h-4 w-4 text-norn" />
            <span>Import from CLI</span>
          </button>
        </div>
      </div>
    </div>
  );
}
