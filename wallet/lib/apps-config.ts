// Devnet-deployed loom IDs — hardcoded as defaults so the hosted wallet works out of the box.
// Override via NEXT_PUBLIC_*_LOOM_ID env vars if redeployed to a different chain.
export const ESCROW_LOOM_ID = process.env.NEXT_PUBLIC_ESCROW_LOOM_ID || "fa25fc6413367a80f6636fc6009ba82140d53d2d73e2d9302936049f354781e5";
export const TREASURY_LOOM_ID = process.env.NEXT_PUBLIC_TREASURY_LOOM_ID || "9d2e1be8e0e49bd6a643d99d2ba8117ba72febd92488b927268c1f2b0bb697a3";
export const VESTING_LOOM_ID = process.env.NEXT_PUBLIC_VESTING_LOOM_ID || "d247e5c528f04f9ed71ac9ef4a512b627d1aa415bc8d2646209ae1804f13e7fe";
export const LAUNCHPAD_LOOM_ID = process.env.NEXT_PUBLIC_LAUNCHPAD_LOOM_ID || "9464c19c592e7018ef793d25e2d34421c06a2597de8057c794097ff2db994426";
export const SPLITTER_LOOM_ID = process.env.NEXT_PUBLIC_SPLITTER_LOOM_ID || "e24ddde0df511895e48313814983d4afe26b18d48e098c9670d37c7ca3652693";
export const CROWDFUND_LOOM_ID = process.env.NEXT_PUBLIC_CROWDFUND_LOOM_ID || "c5850b4c69695ebdeff6bbc39655b45370de3b4931bdedeee3a4462a6a2c0fcc";
export const GOVERNANCE_LOOM_ID = process.env.NEXT_PUBLIC_GOVERNANCE_LOOM_ID || "add7280e3927b8379cf24c6c1f5e1564f92e133b38b214e1f499b58311c53532";
export const STAKING_LOOM_ID = process.env.NEXT_PUBLIC_STAKING_LOOM_ID || "d0d19a55a72a93377d78c9a3281e89b6150ba275d7c41d12aca0790ac891502a";
export const SWAP_LOOM_ID = process.env.NEXT_PUBLIC_SWAP_LOOM_ID || "35dec9810190627c658f52b0876066c6543f59c85806935481d82b2351172382";
export const AIRDROP_LOOM_ID = process.env.NEXT_PUBLIC_AIRDROP_LOOM_ID || "b945488c5a4f03d31a373a4a61c384fa8e68c0ca00065c7d4cb5bd09fcfff2be";
export const TIMELOCK_LOOM_ID = process.env.NEXT_PUBLIC_TIMELOCK_LOOM_ID || "a8ac40574fb30a8e6993ad33936f72ac1cb6500eed8efce8894fbd4732b838dd";

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
