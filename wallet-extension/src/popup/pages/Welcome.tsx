import { Plus, Download, Terminal } from "lucide-react";
import { useNavigationStore } from "@/stores/navigation-store";
import { Button } from "../components/ui/button";

export function Welcome() {
  const navigate = useNavigationStore((s) => s.navigate);

  return (
    <div className="flex h-full flex-col items-center justify-center px-8">
      <div className="mb-8 flex flex-col items-center gap-2 animate-fade-in">
        <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-norn/20">
          <span className="text-2xl font-bold text-norn">N</span>
        </div>
        <h1 className="text-xl font-bold tracking-wider">NORN WALLET</h1>
        <p className="text-center text-sm text-muted-foreground">
          A secure wallet for the Norn Protocol blockchain
        </p>
      </div>

      <div className="flex w-full flex-col gap-3 animate-slide-in">
        <Button
          size="lg"
          className="w-full"
          onClick={() => navigate("create-wallet")}
        >
          <Plus className="h-4 w-4" />
          Create New Wallet
        </Button>

        <Button
          variant="outline"
          size="lg"
          className="w-full"
          onClick={() => navigate("import-wallet")}
        >
          <Download className="h-4 w-4" />
          Import Existing
        </Button>

        <Button
          variant="ghost"
          size="lg"
          className="w-full text-muted-foreground"
          onClick={() => navigate("import-cli")}
          style={{ animationDelay: "100ms" }}
        >
          <Terminal className="h-4 w-4" />
          Import from CLI
        </Button>
      </div>
    </div>
  );
}
