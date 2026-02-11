import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Transactions",
  description: "Browse all transactions on the Norn network. View transfers, pending transactions, and live feed.",
};

export default function TransactionsLayout({ children }: { children: React.ReactNode }) {
  return children;
}
