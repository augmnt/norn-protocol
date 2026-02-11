import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Blocks",
  description: "Browse all blocks on the Norn network. View block height, hash, transaction count, and timestamps.",
};

export default function BlocksLayout({ children }: { children: React.ReactNode }) {
  return children;
}
