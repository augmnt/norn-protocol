"use client";

import { AlertTriangle } from "lucide-react";
import { Button } from "./button";
import { cn } from "@/lib/utils";

interface ErrorStateProps {
  message?: string;
  retry?: () => void;
  className?: string;
}

export function ErrorState({
  message = "Something went wrong",
  retry,
  className,
}: ErrorStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center py-12 text-center",
        className
      )}
    >
      <AlertTriangle className="h-10 w-10 text-destructive/50 mb-4" />
      <h3 className="text-sm font-medium text-foreground">Error</h3>
      <p className="mt-1 text-sm text-muted-foreground">{message}</p>
      {retry && (
        <Button variant="outline" size="sm" onClick={retry} className="mt-4">
          Try again
        </Button>
      )}
    </div>
  );
}
