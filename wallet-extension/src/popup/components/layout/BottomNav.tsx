import { Home, Coins, Clock, Settings } from "lucide-react";
import { cn } from "@/lib/utils";
import { useNavigationStore } from "@/stores/navigation-store";
import type { Route } from "@/types";

const tabs: { route: Route; icon: typeof Home; label: string }[] = [
  { route: "dashboard", icon: Home, label: "Home" },
  { route: "tokens", icon: Coins, label: "Tokens" },
  { route: "activity", icon: Clock, label: "Activity" },
  { route: "settings", icon: Settings, label: "Settings" },
];

const HIDDEN_ROUTES = new Set<string>([
  "welcome",
  "create-wallet",
  "import-wallet",
  "import-cli",
  "unlock",
]);

export function BottomNav() {
  const currentRoute = useNavigationStore((s) => s.currentRoute);
  const reset = useNavigationStore((s) => s.reset);

  if (HIDDEN_ROUTES.has(currentRoute)) return null;

  return (
    <nav className="flex h-14 shrink-0 items-center border-t border-border/50">
      {tabs.map(({ route, icon: Icon, label }) => {
        const isActive = currentRoute === route;
        return (
          <button
            key={route}
            onClick={() => reset(route)}
            className={cn(
              "flex flex-1 flex-col items-center gap-0.5 py-1.5 text-xs transition-colors duration-200",
              isActive
                ? "text-foreground"
                : "text-muted-foreground/50 hover:text-muted-foreground",
            )}
          >
            <Icon className={cn("h-5 w-5", isActive && "text-norn")} />
            <span>{label}</span>
          </button>
        );
      })}
    </nav>
  );
}
