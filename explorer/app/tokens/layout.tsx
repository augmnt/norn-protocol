import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Tokens",
  description: "Browse all NT-1 tokens deployed on the Norn network. View token metadata, supply, and holders.",
};

export default function TokensLayout({ children }: { children: React.ReactNode }) {
  return children;
}
