import { config } from "@/lib/config";

export function Footer() {
  return (
    <footer className="border-t">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <p className="text-xs text-muted-foreground">
          <span className="font-mono font-bold tracking-[-0.02em]">norn</span>{" "}
          explorer
        </p>
        <p className="text-xs text-muted-foreground font-mono">
          {config.chainName}
        </p>
      </div>
    </footer>
  );
}
