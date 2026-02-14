// Devnet-deployed loom IDs — hardcoded as defaults so the hosted wallet works out of the box.
// Override via NEXT_PUBLIC_*_LOOM_ID env vars if redeployed to a different chain.
export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "16a9f99e4c1be51c4723be40daecd3918b8c79acf24786042d2d0b2eb6b3ae6f";
export const TREASURY_LOOM_ID = process.env.NEXT_PUBLIC_TREASURY_LOOM_ID || "10f0ce975f736edee8ca6cf1fa74b3060c2094315dc25923fcba42ec542fdccd";
export const VESTING_LOOM_ID = process.env.NEXT_PUBLIC_VESTING_LOOM_ID || "f3b8fb5e8ed90105b78c884faafd330bfd7f84d88c780b553235fee0f759913e";
export const LAUNCHPAD_LOOM_ID = process.env.NEXT_PUBLIC_LAUNCHPAD_LOOM_ID || "0b17e0474c5c55e3e0e0ae0316f0241eca91a709dc5df92bd60202bfa0a54501";
export const SPLITTER_LOOM_ID = process.env.NEXT_PUBLIC_SPLITTER_LOOM_ID || "6a912ff8b740aa8632e91f52916c0915839d37b5d37dc84b0bc0e268d3129134";
export const CROWDFUND_LOOM_ID = process.env.NEXT_PUBLIC_CROWDFUND_LOOM_ID || "88988861e80d95dd58e3c68a5ac8bb81513ece16d85805f43885b981ccde4cca";
export const GOVERNANCE_LOOM_ID = process.env.NEXT_PUBLIC_GOVERNANCE_LOOM_ID || "a3259772d8d5793f0671b0422c71166fa3527c84d0e89607541e157d126f1903";
export const STAKING_LOOM_ID = process.env.NEXT_PUBLIC_STAKING_LOOM_ID || "2cae114b457dd6706a31d012955e3f376b447f9a1d057cb7982b1ddfca88cbd4";
export const SWAP_LOOM_ID = process.env.NEXT_PUBLIC_SWAP_LOOM_ID || "a4fa9ca2fd378abb36d80b283085075e1b3142e201aba12eb4712acb5ee82f00";
export const AIRDROP_LOOM_ID = process.env.NEXT_PUBLIC_AIRDROP_LOOM_ID || "2d24cdf01637bb9b96ef51a11d0bc6ac947955415058c60a868436bf1862a00a";
export const TIMELOCK_LOOM_ID = process.env.NEXT_PUBLIC_TIMELOCK_LOOM_ID || "304380b3667de1e9c9d07db92ff5f807bf7bd668fe77437a3096805cf7ea8d47";

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
    icon: "Hourglass",
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
    icon: "GitFork",
  },
  {
    id: "crowdfund",
    name: "Crowdfund",
    description:
      "All-or-nothing fundraising with a goal and deadline. If the goal is met, the creator gets the funds. Otherwise, contributors are refunded.",
    loomId: CROWDFUND_LOOM_ID,
    href: "/apps/crowdfund",
    icon: "HandCoins",
  },
  {
    id: "governance",
    name: "DAO Governance",
    description:
      "On-chain voting on proposals with quorum requirements. Create proposals, vote for or against, and finalize outcomes.",
    loomId: GOVERNANCE_LOOM_ID,
    href: "/apps/governance",
    icon: "Vote",
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
