export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "";
export const TREASURY_LOOM_ID = process.env.NEXT_PUBLIC_TREASURY_LOOM_ID || "";
export const VESTING_LOOM_ID = process.env.NEXT_PUBLIC_VESTING_LOOM_ID || "";
export const LAUNCHPAD_LOOM_ID = process.env.NEXT_PUBLIC_LAUNCHPAD_LOOM_ID || "";
export const SPLITTER_LOOM_ID = process.env.NEXT_PUBLIC_SPLITTER_LOOM_ID || "";
export const CROWDFUND_LOOM_ID = process.env.NEXT_PUBLIC_CROWDFUND_LOOM_ID || "";
export const GOVERNANCE_LOOM_ID = process.env.NEXT_PUBLIC_GOVERNANCE_LOOM_ID || "";
export const STAKING_LOOM_ID = process.env.NEXT_PUBLIC_STAKING_LOOM_ID || "";
export const SWAP_LOOM_ID = process.env.NEXT_PUBLIC_SWAP_LOOM_ID || "";
export const AIRDROP_LOOM_ID = process.env.NEXT_PUBLIC_AIRDROP_LOOM_ID || "";
export const TIMELOCK_LOOM_ID = process.env.NEXT_PUBLIC_TIMELOCK_LOOM_ID || "";

export interface AppConfig {
  id: string;
  name: string;
  description: string;
  loomId: string;
  href: string;
  icon: string;
}

export const APPS: AppConfig[] = [
  {
    id: "escrow",
    name: "P2P Escrow",
    description:
      "Create secure peer-to-peer deals with automatic escrow. Funds are held by the contract until both parties confirm.",
    loomId: ESCROW_LOOM_ID,
    href: "/apps/escrow",
    icon: "ShieldCheck",
  },
  {
    id: "treasury",
    name: "Multisig Treasury",
    description:
      "Shared treasury requiring multiple approvals for outgoing transfers. Create proposals, vote, and manage funds collectively.",
    loomId: TREASURY_LOOM_ID,
    href: "/apps/treasury",
    icon: "Vault",
  },
  {
    id: "vesting",
    name: "Token Vesting",
    description:
      "Time-locked token releases with cliff periods. Create vesting schedules for team members, advisors, or investors.",
    loomId: VESTING_LOOM_ID,
    href: "/apps/vesting",
    icon: "Timer",
  },
  {
    id: "launchpad",
    name: "Token Launchpad",
    description:
      "Fixed-price token sale with hard cap. Deposit tokens, set a price, and let buyers contribute during the sale window.",
    loomId: LAUNCHPAD_LOOM_ID,
    href: "/apps/launchpad",
    icon: "Rocket",
  },
  {
    id: "splitter",
    name: "Payment Splitter",
    description:
      "Route incoming payments to multiple recipients by percentage. Set once for teams, royalties, or revenue sharing.",
    loomId: SPLITTER_LOOM_ID,
    href: "/apps/splitter",
    icon: "Split",
  },
  {
    id: "crowdfund",
    name: "Crowdfund",
    description:
      "All-or-nothing fundraising with a goal and deadline. If the goal is met, the creator gets the funds. Otherwise, contributors are refunded.",
    loomId: CROWDFUND_LOOM_ID,
    href: "/apps/crowdfund",
    icon: "Heart",
  },
  {
    id: "governance",
    name: "DAO Governance",
    description:
      "On-chain voting on proposals with quorum requirements. Create proposals, vote for or against, and finalize outcomes.",
    loomId: GOVERNANCE_LOOM_ID,
    href: "/apps/governance",
    icon: "Scale",
  },
  {
    id: "staking",
    name: "Staking Vault",
    description:
      "Stake tokens for a lock period and earn rewards over time. Operators fund the reward pool, stakers claim proportional rewards.",
    loomId: STAKING_LOOM_ID,
    href: "/apps/staking",
    icon: "Landmark",
  },
  {
    id: "swap",
    name: "OTC Swap",
    description:
      "Post offers to trade token A for token B at a fixed rate. Counterparties can browse and fill open orders.",
    loomId: SWAP_LOOM_ID,
    href: "/apps/swap",
    icon: "ArrowLeftRight",
  },
  {
    id: "airdrop",
    name: "Airdrop",
    description:
      "Distribute tokens to a list of addresses. Upload recipients and amounts, then let them claim their allocations.",
    loomId: AIRDROP_LOOM_ID,
    href: "/apps/airdrop",
    icon: "Gift",
  },
  {
    id: "timelock",
    name: "Time-locked Vault",
    description:
      "Deposit tokens with a future unlock date. Self-custody with a forced hold period for commitment or savings.",
    loomId: TIMELOCK_LOOM_ID,
    href: "/apps/timelock",
    icon: "Clock",
  },
];
