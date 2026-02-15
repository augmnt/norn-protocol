// Devnet-deployed loom IDs — hardcoded as defaults so the hosted wallet works out of the box.
// Override via NEXT_PUBLIC_*_LOOM_ID env vars if redeployed to a different chain.
export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "db400759c865c630d10837d8d98b7379d89a00a74243ca3a16252f80a55b6222";
export const TREASURY_LOOM_ID = process.env.NEXT_PUBLIC_TREASURY_LOOM_ID || "3467b88a791be9d59ebe6267b5bb4a80edfc80ffbb01c8899a831ecc28292489";
export const VESTING_LOOM_ID = process.env.NEXT_PUBLIC_VESTING_LOOM_ID || "22a5216fc09caf10522d68b5302b4b49ad047be90367b2133ef24ca33c6a7be2";
export const LAUNCHPAD_LOOM_ID = process.env.NEXT_PUBLIC_LAUNCHPAD_LOOM_ID || "6b9f1e0c874a418543cdfa7f376ae12a4d080447be48cfad81aab9ef9093dfd4";
export const SPLITTER_LOOM_ID = process.env.NEXT_PUBLIC_SPLITTER_LOOM_ID || "7ccf92f09d35851b84a252be68e8d97a0a3d3ebdfa4b4b30ce17bd3e33cd0a5b";
export const CROWDFUND_LOOM_ID = process.env.NEXT_PUBLIC_CROWDFUND_LOOM_ID || "bd89be7d37fff05b8e1d3c0bd335d2b83b6ba0a27c8c5ce75e30f01a9ada785d";
export const GOVERNANCE_LOOM_ID = process.env.NEXT_PUBLIC_GOVERNANCE_LOOM_ID || "33a72b64bd104d477491f25681ee37245f8fdc81e4e6594a7b78d7f3751574b1";
export const STAKING_LOOM_ID = process.env.NEXT_PUBLIC_STAKING_LOOM_ID || "6965a76bf254d5abb7b45d639139b3e1cdba4eb7a49642728c8ed0785874014b";
export const SWAP_LOOM_ID = process.env.NEXT_PUBLIC_SWAP_LOOM_ID || "3cdb524b9fc7e80f447555d7b3504ec2b56433127250e2a6d6275cd58605bd8c";
export const AIRDROP_LOOM_ID = process.env.NEXT_PUBLIC_AIRDROP_LOOM_ID || "729011463b335b6ee950dfb6b831647846156cbb917363cd678aa7c9bd909a16";
export const TIMELOCK_LOOM_ID = process.env.NEXT_PUBLIC_TIMELOCK_LOOM_ID || "cad0001e6f753858d00795a9b0dd2fcce09adfb76bd273400fde84db57517354";

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
