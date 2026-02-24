import { useEffect, useState } from "react";
import { Toaster } from "sonner";
import { useWalletStore } from "@/stores/wallet-store";
import { useNavigationStore } from "@/stores/navigation-store";
import { useNetworkStore } from "@/stores/network-store";
import { Welcome } from "./pages/Welcome";
import { CreateWallet } from "./pages/CreateWallet";
import { ImportWallet } from "./pages/ImportWallet";
import { ImportFromCli } from "./pages/ImportFromCli";
import { Unlock } from "./pages/Unlock";
import { Dashboard } from "./pages/Dashboard";
import { Send } from "./pages/Send";
import { Confirm } from "./pages/Confirm";
import { Receive } from "./pages/Receive";
import { Activity } from "./pages/Activity";
import { Tokens } from "./pages/Tokens";
import { Settings } from "./pages/Settings";
import { Accounts } from "./pages/Accounts";
import { RegisterName } from "./pages/RegisterName";
import { TransferName } from "./pages/TransferName";
import { NameRecords } from "./pages/NameRecords";
import { CreateToken } from "./pages/CreateToken";
import { TokenDetail } from "./pages/TokenDetail";
import { MintToken } from "./pages/MintToken";
import { BurnToken } from "./pages/BurnToken";
import { TransactionDetail } from "./pages/TransactionDetail";
import { Spinner } from "./components/ui/spinner";
import { PageTransition } from "./components/ui/page-transition";

export function App() {
  const [booting, setBooting] = useState(true);
  const currentRoute = useNavigationStore((s) => s.currentRoute);
  const reset = useNavigationStore((s) => s.reset);
  const initialize = useWalletStore((s) => s.initialize);
  const loadNetwork = useNetworkStore((s) => s.loadSaved);

  useEffect(() => {
    async function boot() {
      await loadNetwork();
      const { hasWallet, isLocked } = await initialize();

      if (!hasWallet) {
        reset("welcome");
      } else if (isLocked) {
        reset("unlock");
      } else {
        reset("dashboard");
      }
      setBooting(false);
    }
    boot();
  }, []);

  if (booting) {
    return (
      <div className="flex h-full items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <Toaster
        theme="dark"
        position="top-center"
        toastOptions={{
          style: {
            background: "hsl(240 10% 7%)",
            border: "1px solid hsl(240 3.7% 20%)",
            color: "hsl(0 0% 98%)",
          },
        }}
      />
      <div className="flex-1 overflow-hidden">
        <PageTransition routeKey={currentRoute}>
          {renderRoute(currentRoute)}
        </PageTransition>
      </div>
    </div>
  );
}

function renderRoute(route: string) {
  switch (route) {
    case "welcome":
      return <Welcome />;
    case "create-wallet":
      return <CreateWallet />;
    case "import-wallet":
      return <ImportWallet />;
    case "import-cli":
      return <ImportFromCli />;
    case "unlock":
      return <Unlock />;
    case "dashboard":
      return <Dashboard />;
    case "send":
      return <Send />;
    case "confirm":
      return <Confirm />;
    case "receive":
      return <Receive />;
    case "activity":
      return <Activity />;
    case "transaction-detail":
      return <TransactionDetail />;
    case "tokens":
      return <Tokens />;
    case "create-token":
      return <CreateToken />;
    case "token-detail":
      return <TokenDetail />;
    case "mint-token":
      return <MintToken />;
    case "burn-token":
      return <BurnToken />;
    case "settings":
      return <Settings />;
    case "accounts":
      return <Accounts />;
    case "register-name":
      return <RegisterName />;
    case "transfer-name":
      return <TransferName />;
    case "name-records":
      return <NameRecords />;
    default:
      return <Welcome />;
  }
}
