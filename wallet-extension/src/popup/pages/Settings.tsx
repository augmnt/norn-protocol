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
} from "lucide-react";
import { toast } from "sonner";
import { useNavigationStore } from "@/stores/navigation-store";
import { useNetworkStore } from "@/stores/network-store";
import { useWalletStore } from "@/stores/wallet-store";
import { getAutoLockMinutes, setAutoLockMinutes, exportPrivateKey } from "@/lib/keystore";
import { Header } from "../components/layout/Header";
import { BottomNav } from "../components/layout/BottomNav";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { CopyButton } from "../components/ui/copy-button";

export function Settings() {
  const [rpcUrl, setRpcUrl] = useState("");
  const [wsUrl, setWsUrl] = useState("");
  const [autoLock, setAutoLock] = useState(15);
  const [showExport, setShowExport] = useState(false);
  const [exportPassword, setExportPassword] = useState("");
  const [exportedKey, setExportedKey] = useState("");
  const [showKey, setShowKey] = useState(false);

  const navigate = useNavigationStore((s) => s.navigate);
  const network = useNetworkStore();
  const lock = useWalletStore((s) => s.lock);
  const activeAccountId = useWalletStore((s) => s.activeAccountId);

  useEffect(() => {
    setRpcUrl(network.rpcUrl);
    setWsUrl(network.wsUrl);
    getAutoLockMinutes().then(setAutoLock);
  }, [network.rpcUrl, network.wsUrl]);

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

  return (
    <div className="flex h-full flex-col">
      <Header />

      <div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4 scrollbar-thin">
        <h2 className="text-lg font-semibold">Settings</h2>

        <Card>
          <CardHeader className="p-4 pb-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Globe className="h-4 w-4 text-norn" />
              Network
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 p-4 pt-3">
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
      </div>

      <BottomNav />
    </div>
  );
}
