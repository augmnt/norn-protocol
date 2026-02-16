import { ArrowLeft, Lock } from "lucide-react";
import { useNavigationStore } from "@/stores/navigation-store";
import { useWalletStore } from "@/stores/wallet-store";
import { Button } from "../ui/button";

const NO_BACK_ROUTES = new Set([
  "welcome",
  "dashboard",
  "unlock",
]);

export function Header() {
  const currentRoute = useNavigationStore((s) => s.currentRoute);
  const goBack = useNavigationStore((s) => s.goBack);
  const lock = useWalletStore((s) => s.lock);
  const isLocked = useWalletStore((s) => s.isLocked);
  const navigate = useNavigationStore((s) => s.navigate);

  const showBack = !NO_BACK_ROUTES.has(currentRoute);
  const showLock =
    !isLocked && currentRoute !== "welcome" && currentRoute !== "unlock";

  const handleLock = () => {
    lock();
    navigate("unlock");
  };

  return (
    <header className="flex h-12 shrink-0 items-center border-b px-3">
      <div className="w-9">
        {showBack && (
          <Button
            variant="ghost"
            size="icon"
            onClick={goBack}
            className="h-8 w-8 transition-colors duration-150 hover:text-norn"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
        )}
      </div>

      <div className="flex flex-1 items-center justify-center gap-1.5">
        <span className="font-mono text-sm font-bold tracking-[-0.02em] text-foreground">
          norn
        </span>
        <span className="text-[10px] text-muted-foreground">wallet</span>
      </div>

      <div className="w-9">
        {showLock && (
          <Button
            variant="ghost"
            size="icon"
            onClick={handleLock}
            className="h-8 w-8 transition-colors duration-150 hover:text-norn"
          >
            <Lock className="h-4 w-4" />
          </Button>
        )}
      </div>
    </header>
  );
}
