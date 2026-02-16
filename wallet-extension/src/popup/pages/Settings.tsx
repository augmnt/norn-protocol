import { useState, useEffect } from "react";
import {
  Globe,
  Shield,
  Key,
  Users,
  Eye,
  EyeOff,
  Lock,
  Coins,
  Trash2,
  KeyRound,
  ChevronDown,
} from "lucide-react";
import { toast } from "sonner";
import { useNavigationStore } from "@/stores/navigation-store";
import { useNetworkStore } from "@/stores/network-store";
import { useWalletStore } from "@/stores/wallet-store";
import { getAutoLockMinutes, setAutoLockMinutes, exportPrivateKey } from "@/lib/keystore";
import { changePassword, forgetWallet } from "@/lib/keystore";
import { NETWORK_PRESETS } from "@/lib/config";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { CopyButton } from "../components/ui/copy-button";
import { Spinner } from "../components/ui/spinner";
import { cn } from "@/lib/utils";

export function Settings() {
  const [rpcUrl, setRpcUrl] = useState("");
  const [wsUrl, setWsUrl] = useState("");
  const [autoLock, setAutoLock] = useState(15);
  const [showExport, setShowExport] = useState(false);
  const [exportPassword, setExportPassword] = useState("");
  const [exportedKey, setExportedKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [showPresets, setShowPresets] = useState(false);

  // Change password state
  const [showChangePassword, setShowChangePassword] = useState(false);
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [changingPassword, setChangingPassword] = useState(false);

  // Forget wallet state
  const [showForgetWallet, setShowForgetWallet] = useState(false);
  const [forgetPassword, setForgetPassword] = useState("");
  const [forgetting, setForgetting] = useState(false);

  const navigate = useNavigationStore((s) => s.navigate);
  const reset = useNavigationStore((s) => s.reset);
  const network = useNetworkStore();
  const lock = useWalletStore((s) => s.lock);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);

  useEffect(() => {
    setRpcUrl(network.rpcUrl);
    setWsUrl(network.wsUrl);
    getAutoLockMinutes().then(setAutoLock);
  }, [network.rpcUrl, network.wsUrl]);

  const handleSelectPreset = async (preset: typeof NETWORK_PRESETS[0]) => {
    setRpcUrl(preset.rpcUrl);
    setWsUrl(preset.wsUrl);
    await network.setNetwork(preset.rpcUrl, preset.wsUrl, preset.name);
    setShowPresets(false);
    toast.success(`Switched to ${preset.name}`);
  };

  const handleSaveNetwork = async () => {
    await network.setNetwork(rpcUrl, wsUrl, "Custom");
    toast.success("Network settings saved");
  };

  const handleSaveAutoLock = async () => {
    await setAutoLockMinutes(autoLock);
    toast.success("Auto-lock updated");
  };

  const handleLockNow = () => {
    lock();
    navigate("unlock");
  };

  const handleExportKey = async () => {
    if (!activeAccountId || !exportPassword) return;
    try {
      const key = await exportPrivateKey(activeAccountId, exportPassword);
      setExportedKey(key);
    } catch {
      toast.error("Incorrect password");
    }
  };

  const handleChangePassword = async () => {
    if (!currentPassword || !newPassword || newPassword !== confirmPassword) return;
    if (newPassword.length < 8) {
      toast.error("New password must be at least 8 characters");
      return;
    }
    setChangingPassword(true);
    try {
      await changePassword(currentPassword, newPassword);
      setShowChangePassword(false);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      toast.success("Password changed successfully");
      // Re-lock so user authenticates with new password
      lock();
      reset("unlock");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to change password");
    } finally {
      setChangingPassword(false);
    }
  };

  const handleForgetWallet = async () => {
    if (!forgetPassword) return;
    setForgetting(true);
    try {
      await forgetWallet(forgetPassword);
      toast.success("Wallet data erased");
      reset("welcome");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Incorrect password");
    } finally {
      setForgetting(false);
    }
  };

  const isCustomNetwork = !NETWORK_PRESETS.some(
    (p) => p.rpcUrl === network.rpcUrl && p.wsUrl === network.wsUrl,
  );

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4 scrollbar-thin">
        <h2 className="text-lg font-semibold">Settings</h2>

        {/* Network */}
        <Card>
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Globe className="h-4 w-4 text-norn" />
              Network
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
            {/* Preset selector */}
            <div className="relative">
              <button
                type="button"
                onClick={() => setShowPresets(!showPresets)}
                className="flex w-full items-center justify-between rounded-md border bg-background px-3 py-2 text-sm transition-colors duration-150 hover:bg-accent"
              >
                <span>{isCustomNetwork ? "Custom" : network.networkName}</span>
                <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", showPresets && "rotate-180")} />
              </button>
              {showPresets && (
                <div className="absolute left-0 right-0 top-full z-10 mt-1 rounded-md border bg-card shadow-lg animate-fade-in">
                  {NETWORK_PRESETS.map((preset) => (
                    <button
                      key={preset.name}
                      type="button"
                      onClick={() => handleSelectPreset(preset)}
                      className={cn(
                        "flex w-full items-center justify-between px-3 py-2 text-sm transition-colors duration-150 hover:bg-accent first:rounded-t-md last:rounded-b-md",
                        network.networkName === preset.name && !isCustomNetwork && "text-norn",
                      )}
                    >
                      <span>{preset.name}</span>
                      <span className="font-mono text-xs text-muted-foreground">
                        {new URL(preset.rpcUrl).host}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">RPC URL</label>
              <Input
                value={rpcUrl}
                onChange={(e) => setRpcUrl(e.target.value)}
                className="h-8 text-xs"
              />
            </div>
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">
                WebSocket URL
              </label>
              <Input
                value={wsUrl}
                onChange={(e) => setWsUrl(e.target.value)}
                className="h-8 text-xs"
              />
            </div>
            <Button size="sm" className="w-full" onClick={handleSaveNetwork}>
              Save Network
            </Button>
          </CardContent>
        </Card>

        {/* Security */}
        <Card>
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Shield className="h-4 w-4 text-norn" />
              Security
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
            <div className="flex items-center gap-2">
              <Input
                type="number"
                value={autoLock}
                onChange={(e) => setAutoLock(parseInt(e.target.value) || 0)}
                min={1}
                max={60}
                className="h-8 w-20 text-xs"
              />
              <span className="text-xs text-muted-foreground">
                minutes auto-lock
              </span>
              <Button
                size="sm"
                variant="secondary"
                className="ml-auto h-7"
                onClick={handleSaveAutoLock}
              >
                Save
              </Button>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={handleLockNow}
            >
              <Lock className="h-3.5 w-3.5" />
              Lock Now
            </Button>
          </CardContent>
        </Card>

        {/* Export Private Key */}
        <Card className="border-orange-500/20">
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Key className="h-4 w-4 text-orange-400" />
              Export Private Key
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
            {!showExport ? (
              <Button
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => setShowExport(true)}
              >
                Show Private Key
              </Button>
            ) : exportedKey ? (
              <div className="space-y-2 animate-fade-in">
                <div className="flex items-start gap-2 rounded-md border border-orange-500/20 bg-orange-500/5 p-2">
                  <code className="break-all font-mono text-xs leading-relaxed">
                    {showKey ? exportedKey : "••••••••••••••••"}
                  </code>
                  <div className="flex shrink-0 gap-1">
                    <button
                      onClick={() => setShowKey(!showKey)}
                      className="text-muted-foreground transition-colors duration-150 hover:text-foreground"
                    >
                      {showKey ? (
                        <EyeOff className="h-3.5 w-3.5" />
                      ) : (
                        <Eye className="h-3.5 w-3.5" />
                      )}
                    </button>
                    <CopyButton value={exportedKey} />
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  className="w-full"
                  onClick={() => {
                    setShowExport(false);
                    setExportedKey("");
                    setExportPassword("");
                    setShowKey(false);
                  }}
                >
                  Hide
                </Button>
              </div>
            ) : (
              <div className="flex gap-2 animate-fade-in">
                <Input
                  type="password"
                  value={exportPassword}
                  onChange={(e) => setExportPassword(e.target.value)}
                  placeholder="Enter password"
                  className="h-8 text-xs"
                />
                <Button
                  size="sm"
                  className="h-8"
                  onClick={handleExportKey}
                  disabled={!exportPassword}
                >
                  Confirm
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Change Password */}
        <Card>
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <KeyRound className="h-4 w-4 text-norn" />
              Change Password
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
            {!showChangePassword ? (
              <Button
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => setShowChangePassword(true)}
              >
                Change Password
              </Button>
            ) : (
              <div className="space-y-2 animate-fade-in">
                <Input
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  placeholder="Current password"
                  className="h-8 text-xs"
                />
                <Input
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder="New password (min 8 chars)"
                  className="h-8 text-xs"
                />
                <Input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="Confirm new password"
                  className="h-8 text-xs"
                />
                {newPassword && confirmPassword && newPassword !== confirmPassword && (
                  <p className="text-xs text-destructive animate-fade-in">Passwords do not match</p>
                )}
                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="flex-1"
                    onClick={() => {
                      setShowChangePassword(false);
                      setCurrentPassword("");
                      setNewPassword("");
                      setConfirmPassword("");
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    size="sm"
                    className="flex-1"
                    disabled={
                      !currentPassword ||
                      !newPassword ||
                      newPassword !== confirmPassword ||
                      newPassword.length < 8 ||
                      changingPassword
                    }
                    onClick={handleChangePassword}
                  >
                    {changingPassword ? <Spinner size="sm" /> : "Update"}
                  </Button>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Quick links */}
        <button
          onClick={() => navigate("accounts")}
          className="flex items-center gap-2 rounded-lg border p-3 text-sm transition-colors duration-150 hover:bg-accent"
        >
          <Users className="h-4 w-4 text-norn" />
          <span>Manage Accounts</span>
        </button>

        <button
          onClick={() => navigate("tokens")}
          className="flex items-center gap-2 rounded-lg border p-3 text-sm transition-colors duration-150 hover:bg-accent"
        >
          <Coins className="h-4 w-4 text-norn" />
          <span>View Tokens</span>
        </button>

        {/* Danger zone: Forget Wallet */}
        <Card className="border-destructive/30">
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm text-destructive">
              <Trash2 className="h-4 w-4" />
              Forget Wallet
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
            <p className="text-xs text-muted-foreground">
              This will erase all wallet data from this extension. Make sure you
              have backed up your private keys.
            </p>
            {!showForgetWallet ? (
              <Button
                variant="destructive"
                size="sm"
                className="w-full"
                onClick={() => setShowForgetWallet(true)}
              >
                Forget Wallet
              </Button>
            ) : (
              <div className="space-y-2 animate-fade-in">
                <Input
                  type="password"
                  value={forgetPassword}
                  onChange={(e) => setForgetPassword(e.target.value)}
                  placeholder="Enter password to confirm"
                  className="h-8 text-xs"
                />
                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="flex-1"
                    onClick={() => {
                      setShowForgetWallet(false);
                      setForgetPassword("");
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    className="flex-1"
                    disabled={!forgetPassword || forgetting}
                    onClick={handleForgetWallet}
                  >
                    {forgetting ? <Spinner size="sm" /> : "Erase Everything"}
                  </Button>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <BottomNav />
    </div>
  );
}
