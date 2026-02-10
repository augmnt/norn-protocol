"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import { Check, Copy } from "lucide-react";

interface CodeBlockProps extends React.HTMLAttributes<HTMLDivElement> {
  title?: string;
}

export function CodeBlock({ className, title, children, ...props }: CodeBlockProps) {
  const [copied, setCopied] = React.useState(false);
  const preRef = React.useRef<HTMLPreElement>(null);

  const handleCopy = () => {
    const text = preRef.current?.textContent ?? "";
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      className={cn("group relative my-4 rounded-lg border bg-secondary/50", className)}
      {...props}
    >
      {title && (
        <div className="flex items-center justify-between border-b px-4 py-2">
          <span className="text-xs font-mono text-muted-foreground">{title}</span>
        </div>
      )}
      <div className="relative">
        <button
          onClick={handleCopy}
          className="absolute right-3 top-3 z-10 hidden rounded-md border bg-background/80 p-1.5 text-muted-foreground backdrop-blur transition-colors hover:text-foreground group-hover:block"
          aria-label="Copy code"
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
        </button>
        <pre ref={preRef} className="overflow-x-auto p-4 text-sm leading-relaxed">
          {children}
        </pre>
      </div>
    </div>
  );
}
