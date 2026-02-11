import type { TransactionHistoryEntry } from "@/types";
import { formatAmount } from "./format";

function csvEscape(field: string): string {
  if (field.includes(",") || field.includes('"') || field.includes("\n")) {
    return `"${field.replace(/"/g, '""')}"`;
  }
  return field;
}

export function exportTransactionsCSV(
  transactions: TransactionHistoryEntry[],
  address: string
) {
  const headers = ["Direction", "From", "To", "Amount (NORN)", "Timestamp"];
  const rows = transactions.map((tx) => [
    tx.direction,
    tx.from,
    tx.to,
    formatAmount(tx.amount, 12, 12),
    new Date(tx.timestamp * 1000).toISOString(),
  ]);

  const csv = [headers, ...rows].map((row) => row.map(csvEscape).join(",")).join("\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `norn-transactions-${address.slice(0, 10)}.csv`;
  link.click();
  URL.revokeObjectURL(url);
}
