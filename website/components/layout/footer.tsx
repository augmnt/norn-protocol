import { Github } from "lucide-react";

export function Footer() {
  return (
    <footer className="border-t">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <p className="text-xs text-muted-foreground">
          <span className="font-mono font-bold tracking-[-0.02em]">norn</span>{" "}
          protocol
        </p>

        <div className="flex items-center gap-4">
          <a
            href="https://explorer.norn.network"
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-muted-foreground transition-colors hover:text-foreground"
          >
            Explorer
          </a>
          <a
            href="https://github.com/augmnt/norn-protocol"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground transition-colors hover:text-foreground"
            aria-label="GitHub repository"
          >
            <Github className="h-4 w-4" />
          </a>
        </div>
      </div>
    </footer>
  );
}
