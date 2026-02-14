export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "";

export interface AppConfig {
  id: string;
  name: string;
  description: string;
  loomId: string;
  href: string;
}

export const APPS: AppConfig[] = [
  {
    id: "escrow",
    name: "P2P Escrow",
    description:
      "Create secure peer-to-peer deals with automatic escrow. Funds are held by the contract until both parties confirm.",
    loomId: ESCROW_LOOM_ID,
    href: "/apps/escrow",
  },
];
