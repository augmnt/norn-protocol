/**
 * Centralized config decoder mapping for the discovery feed.
 * Maps each app type to its getConfig encoder, config decoder, and a summarize function
 * that extracts display-friendly fields for feed cards.
 *
 * For app types without a getConfig query (escrow, swap, timelock, vesting),
 * we use count-based queries instead.
 */

import {
  encodeGetConfig as encodeCrowdfundGetConfig,
  decodeCrowdfundConfig,
  encodeGetTotalRaised as encodeCrowdfundTotalRaised,
  decodeU128 as decodeCrowdfundU128,
} from "./borsh-crowdfund";
import {
  encodeGetConfig as encodeGovGetConfig,
  decodeGovConfig,
  encodeGetProposalCount as encodeGovProposalCount,
  decodeU64 as decodeGovU64,
} from "./borsh-governance";
import {
  encodeGetConfig as encodeTreasuryGetConfig,
  decodeTreasuryConfig,
  encodeGetProposalCount as encodeTreasuryProposalCount,
  decodeU64 as decodeTreasuryU64,
} from "./borsh-treasury";
import {
  encodeGetConfig as encodeLaunchGetConfig,
  decodeLaunchConfig,
  encodeGetTotalRaised as encodeLaunchTotalRaised,
  decodeU128 as decodeLaunchU128,
} from "./borsh-launchpad";
import {
  encodeGetConfig as encodeSplitterGetConfig,
  decodeSplitterConfig,
} from "./borsh-splitter";
import {
  encodeGetConfig as encodeStakingGetConfig,
  decodeStakingConfig,
  encodeGetTotalStaked,
  decodeU128 as decodeStakingU128,
} from "./borsh-staking";
import {
  encodeGetConfig as encodeAirdropGetConfig,
  decodeAirdropConfig,
} from "./borsh-airdrop";
import {
  encodeGetDealCount,
  decodeU64 as decodeEscrowU64,
} from "./borsh-escrow";
import {
  encodeGetOrderCount,
  decodeU64 as decodeSwapU64,
} from "./borsh-swap";
import {
  encodeGetLockCount,
  decodeU64 as decodeTimelockU64,
} from "./borsh-timelock";
import {
  encodeGetScheduleCount,
  decodeU64 as decodeVestingU64,
} from "./borsh-vesting";
import {
  encodeGetPoolCount as encodeAmmGetPoolCount,
  encodeGetConfig as encodeAmmGetConfig,
  decodeU64 as decodeAmmU64,
  decodeAmmConfig,
} from "./borsh-amm";

export interface FeedSummary {
  /** Primary label for the card (title, name, or app type name) */
  title: string;
  /** One-line description or status */
  subtitle: string;
  /** Key-value stats to show on the card */
  stats: { label: string; value: string }[];
  /** Optional status badge text */
  status?: string;
  /** Status variant for styling */
  statusVariant?: "norn" | "secondary" | "destructive";
  /** Optional progress value 0-100 */
  progress?: number;
}

type QueryFn = (
  queryLoom: (loomId: string, inputHex: string) => Promise<{ output_hex?: string }>
) => Promise<FeedSummary>;

export interface FeedConfigDecoder {
  /** Function that queries the loom and returns a summary */
  fetchSummary: (
    loomId: string,
    queryLoom: (loomId: string, inputHex: string) => Promise<{ output_hex?: string }>
  ) => Promise<FeedSummary>;
}

function fmt(raw: bigint, decimals = 12, maxFrac = 2): string {
  const divisor = BigInt(10 ** decimals);
  const whole = raw / divisor;
  const frac = raw % divisor;
  if (frac === 0n) return whole.toLocaleString();
  const fracStr = frac.toString().padStart(decimals, "0").slice(0, maxFrac).replace(/0+$/, "");
  return fracStr ? `${whole.toLocaleString()}.${fracStr}` : whole.toLocaleString();
}

export const FEED_DECODERS: Record<string, FeedConfigDecoder> = {
  crowdfund: {
    async fetchSummary(loomId, queryLoom) {
      const [cfgRes, raisedRes] = await Promise.all([
        queryLoom(loomId, encodeCrowdfundGetConfig()),
        queryLoom(loomId, encodeCrowdfundTotalRaised()),
      ]);
      if (!cfgRes?.output_hex) return fallback("Crowdfund");
      const cfg = decodeCrowdfundConfig(cfgRes.output_hex);
      const raised = raisedRes?.output_hex ? decodeCrowdfundU128(raisedRes.output_hex) : 0n;
      const pct = cfg.goal > 0n ? Number((raised * 100n) / cfg.goal) : 0;
      return {
        title: cfg.title || "Untitled Campaign",
        subtitle: cfg.description || "Crowdfunding campaign",
        stats: [
          { label: "Goal", value: `${fmt(cfg.goal)} NORN` },
          { label: "Raised", value: `${fmt(raised)} NORN` },
        ],
        status: cfg.status,
        statusVariant: cfg.status === "Active" ? "norn" : cfg.status === "Succeeded" ? "secondary" : "destructive",
        progress: Math.min(pct, 100),
      };
    },
  },

  governance: {
    async fetchSummary(loomId, queryLoom) {
      const [cfgRes, countRes] = await Promise.all([
        queryLoom(loomId, encodeGovGetConfig()),
        queryLoom(loomId, encodeGovProposalCount()),
      ]);
      if (!cfgRes?.output_hex) return fallback("DAO Governance");
      const cfg = decodeGovConfig(cfgRes.output_hex);
      const count = countRes?.output_hex ? decodeGovU64(countRes.output_hex) : 0n;
      return {
        title: cfg.name || "DAO",
        subtitle: `${count.toString()} proposal${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Quorum", value: fmt(cfg.quorum) },
          { label: "Voting Period", value: `${(Number(cfg.votingPeriod) / 3600).toFixed(0)}h` },
          { label: "Proposals", value: count.toString() },
        ],
      };
    },
  },

  treasury: {
    async fetchSummary(loomId, queryLoom) {
      const [cfgRes, countRes] = await Promise.all([
        queryLoom(loomId, encodeTreasuryGetConfig()),
        queryLoom(loomId, encodeTreasuryProposalCount()),
      ]);
      if (!cfgRes?.output_hex) return fallback("Multisig Treasury");
      const cfg = decodeTreasuryConfig(cfgRes.output_hex);
      const count = countRes?.output_hex ? decodeTreasuryU64(countRes.output_hex) : 0n;
      return {
        title: cfg.name || "Treasury",
        subtitle: `${cfg.owners.length} owner${cfg.owners.length !== 1 ? "s" : ""}, ${cfg.requiredApprovals.toString()} required`,
        stats: [
          { label: "Owners", value: cfg.owners.length.toString() },
          { label: "Required", value: `${cfg.requiredApprovals.toString()}/${cfg.owners.length}` },
          { label: "Proposals", value: count.toString() },
        ],
      };
    },
  },

  launchpad: {
    async fetchSummary(loomId, queryLoom) {
      const [cfgRes, raisedRes] = await Promise.all([
        queryLoom(loomId, encodeLaunchGetConfig()),
        queryLoom(loomId, encodeLaunchTotalRaised()),
      ]);
      if (!cfgRes?.output_hex) return fallback("Token Launchpad");
      const cfg = decodeLaunchConfig(cfgRes.output_hex);
      const raised = raisedRes?.output_hex ? decodeLaunchU128(raisedRes.output_hex) : 0n;
      const pct = cfg.hardCap > 0n ? Number((raised * 100n) / cfg.hardCap) : 0;
      return {
        title: "Token Sale",
        subtitle: cfg.finalized ? "Sale finalized" : "Sale active",
        stats: [
          { label: "Price", value: fmt(cfg.price) },
          { label: "Hard Cap", value: `${fmt(cfg.hardCap)} NORN` },
          { label: "Raised", value: `${fmt(raised)} NORN` },
        ],
        status: cfg.finalized ? "Finalized" : "Active",
        statusVariant: cfg.finalized ? "secondary" : "norn",
        progress: Math.min(pct, 100),
      };
    },
  },

  splitter: {
    async fetchSummary(loomId, queryLoom) {
      const cfgRes = await queryLoom(loomId, encodeSplitterGetConfig());
      if (!cfgRes?.output_hex) return fallback("Payment Splitter");
      const cfg = decodeSplitterConfig(cfgRes.output_hex);
      return {
        title: cfg.name || "Splitter",
        subtitle: `${cfg.recipients.length} recipient${cfg.recipients.length !== 1 ? "s" : ""}`,
        stats: [
          { label: "Recipients", value: cfg.recipients.length.toString() },
        ],
      };
    },
  },

  staking: {
    async fetchSummary(loomId, queryLoom) {
      const [cfgRes, stakedRes] = await Promise.all([
        queryLoom(loomId, encodeStakingGetConfig()),
        queryLoom(loomId, encodeGetTotalStaked()),
      ]);
      if (!cfgRes?.output_hex) return fallback("Staking Vault");
      const cfg = decodeStakingConfig(cfgRes.output_hex);
      const staked = stakedRes?.output_hex ? decodeStakingU128(stakedRes.output_hex) : 0n;
      const lockHours = Number(cfg.minLockPeriod) / 3600;
      return {
        title: "Staking Vault",
        subtitle: `${fmt(staked)} staked`,
        stats: [
          { label: "Total Staked", value: fmt(staked) },
          { label: "Reward Rate", value: fmt(cfg.rewardRate) },
          { label: "Min Lock", value: lockHours >= 24 ? `${(lockHours / 24).toFixed(0)}d` : `${lockHours.toFixed(0)}h` },
        ],
      };
    },
  },

  airdrop: {
    async fetchSummary(loomId, queryLoom) {
      const cfgRes = await queryLoom(loomId, encodeAirdropGetConfig());
      if (!cfgRes?.output_hex) return fallback("Airdrop");
      const cfg = decodeAirdropConfig(cfgRes.output_hex);
      const pct = cfg.totalAmount > 0n ? Number((cfg.claimedAmount * 100n) / cfg.totalAmount) : 0;
      return {
        title: "Token Airdrop",
        subtitle: cfg.finalized ? `${cfg.recipientCount.toString()} recipients` : "Pending finalization",
        stats: [
          { label: "Total", value: fmt(cfg.totalAmount) },
          { label: "Claimed", value: fmt(cfg.claimedAmount) },
          { label: "Recipients", value: cfg.recipientCount.toString() },
        ],
        status: cfg.finalized ? "Finalized" : "Pending",
        statusVariant: cfg.finalized ? "norn" : "secondary",
        progress: Math.min(pct, 100),
      };
    },
  },

  escrow: {
    async fetchSummary(loomId, queryLoom) {
      const countRes = await queryLoom(loomId, encodeGetDealCount());
      const count = countRes?.output_hex ? decodeEscrowU64(countRes.output_hex) : 0n;
      return {
        title: "P2P Escrow",
        subtitle: `${count.toString()} deal${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Deals", value: count.toString() },
        ],
      };
    },
  },

  swap: {
    async fetchSummary(loomId, queryLoom) {
      const countRes = await queryLoom(loomId, encodeGetOrderCount());
      const count = countRes?.output_hex ? decodeSwapU64(countRes.output_hex) : 0n;
      return {
        title: "OTC Swap",
        subtitle: `${count.toString()} order${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Orders", value: count.toString() },
        ],
      };
    },
  },

  timelock: {
    async fetchSummary(loomId, queryLoom) {
      const countRes = await queryLoom(loomId, encodeGetLockCount());
      const count = countRes?.output_hex ? decodeTimelockU64(countRes.output_hex) : 0n;
      return {
        title: "Time-locked Vault",
        subtitle: `${count.toString()} lock${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Locks", value: count.toString() },
        ],
      };
    },
  },

  vesting: {
    async fetchSummary(loomId, queryLoom) {
      const countRes = await queryLoom(loomId, encodeGetScheduleCount());
      const count = countRes?.output_hex ? decodeVestingU64(countRes.output_hex) : 0n;
      return {
        title: "Token Vesting",
        subtitle: `${count.toString()} schedule${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Schedules", value: count.toString() },
        ],
      };
    },
  },

  "amm-pool": {
    async fetchSummary(loomId, queryLoom) {
      const [countRes, cfgRes] = await Promise.all([
        queryLoom(loomId, encodeAmmGetPoolCount()),
        queryLoom(loomId, encodeAmmGetConfig()),
      ]);
      const count = countRes?.output_hex ? decodeAmmU64(countRes.output_hex) : 0n;
      const feeBps = cfgRes?.output_hex ? decodeAmmConfig(cfgRes.output_hex).feeBps : 30;
      const feeStr = (feeBps / 100).toFixed(feeBps % 100 === 0 ? 0 : 1) + "%";
      return {
        title: "AMM Pool",
        subtitle: `${count.toString()} pool${count !== 1n ? "s" : ""}`,
        stats: [
          { label: "Pools", value: count.toString() },
          { label: "Fee", value: feeStr },
        ],
      };
    },
  },
};

function fallback(name: string): FeedSummary {
  return {
    title: name,
    subtitle: "Unable to load config",
    stats: [],
  };
}
