// Devnet-deployed loom IDs — hardcoded as defaults so the hosted wallet works out of the box.
// Override via NEXT_PUBLIC_*_LOOM_ID env vars if redeployed to a different chain.
export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "879f57ae1fdf6fa3cabc98174b725e07a3639eca727a95fb73f2a07d1c45cfc2";
export const TREASURY_LOOM_ID = process.env.NEXT_PUBLIC_TREASURY_LOOM_ID || "cafe2379d02feb1208ba963dd51c46b96a5da1ff70fb82785a55e7b0ac650f57";
export const VESTING_LOOM_ID = process.env.NEXT_PUBLIC_VESTING_LOOM_ID || "1198d469de7e4c1682745ab817b9de9c249a1eee149ecd1e63c4ad7522c0550c";
export const LAUNCHPAD_LOOM_ID = process.env.NEXT_PUBLIC_LAUNCHPAD_LOOM_ID || "dea9a0b7ab0272ec03bec51655e24a7a94de0e366082ebf8bbdc0ce06cf32612";
export const SPLITTER_LOOM_ID = process.env.NEXT_PUBLIC_SPLITTER_LOOM_ID || "dc3491d185ee648d32000529c6471b51a827bdfad664e0e5b63dcf2dbded6f13";
export const CROWDFUND_LOOM_ID = process.env.NEXT_PUBLIC_CROWDFUND_LOOM_ID || "e6ae99b63282f44a14b403f298d96b02b8cc9237522fb087e921978b2655b42e";
export const GOVERNANCE_LOOM_ID = process.env.NEXT_PUBLIC_GOVERNANCE_LOOM_ID || "e0d7a19a1b044d39286d7a0ecc1c07a60b9f088ab57c0ebbf55eb4b7dda3e9be";
export const STAKING_LOOM_ID = process.env.NEXT_PUBLIC_STAKING_LOOM_ID || "28d7b3d7f3e11ad05a5b95fe5c1412351cc4aaa6d6306e957a3565a1ca3e8ad8";
export const SWAP_LOOM_ID = process.env.NEXT_PUBLIC_SWAP_LOOM_ID || "5803a47b80295bc0599b1936d250e566f3f5f38e4d3ffab94d32fb64b78b1010";
export const AIRDROP_LOOM_ID = process.env.NEXT_PUBLIC_AIRDROP_LOOM_ID || "82cd86207a9e5eb89b630d6ee69d102db5133b255421c89622153b3942445040";
export const TIMELOCK_LOOM_ID = process.env.NEXT_PUBLIC_TIMELOCK_LOOM_ID || "4ebc7260fdcd1f75da2e9bc0de3c39ce87e86b823e0e62699569510bb5deff5a";

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
