import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Smart Contracts",
  description: "Browse deployed Loom smart contracts on the Norn network. View contract metadata and participants.",
};

export default function ContractsLayout({ children }: { children: React.ReactNode }) {
  return children;
}
