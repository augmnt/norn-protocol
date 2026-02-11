import { formatDistanceToNowStrict } from "date-fns";

export function truncateHash(hash: string, chars = 6): string {
  if (!hash) return "";
  if (hash.length <= chars * 2 + 2) return hash;
  return `${hash.slice(0, chars + 2)}...${hash.slice(-chars)}`;
}

export function truncateAddress(address: string): string {
  return truncateHash(address, 4);
}

export function formatAmount(
  amount: string | bigint,
  decimals = 12,
  maxFractionDigits = 4
): string {
  const raw = typeof amount === "bigint" ? amount : BigInt((typeof amount === "string" ? amount.trim() : amount) || "0");
  const divisor = BigInt(10 ** decimals);
  const whole = raw / divisor;
  const frac = raw % divisor;

  if (frac === 0n) return whole.toLocaleString();

  const fracStr = frac.toString().padStart(decimals, "0");
  const trimmed = fracStr.slice(0, maxFractionDigits).replace(/0+$/, "");
  if (!trimmed) return whole.toLocaleString();
  return `${whole.toLocaleString()}.${trimmed}`;
}

export function formatNorn(amount: string): string {
  return formatAmount(amount, 12, 4);
}

export function formatNumber(n: number): string {
  return n.toLocaleString();
}

export function timeAgo(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return formatDistanceToNowStrict(date, { addSuffix: true });
}

export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function isValidAddress(value: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(value);
}

export function isValidHash(value: string): boolean {
  return /^0x[a-fA-F0-9]{64}$/.test(value);
}

export function strip0x(hex: string): string {
  return hex.startsWith("0x") ? hex.slice(2) : hex;
}
