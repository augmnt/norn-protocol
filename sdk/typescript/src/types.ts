/** 20-byte account address as hex string (with 0x prefix). */
export type AddressHex = string;

/** 32-byte hash/ID as hex string. */
export type HashHex = string;

/** 32-byte public key as hex string. */
export type PubKeyHex = string;

/** Amount as bigint (u128 in Rust, 12 decimal places for NORN). */
export type Amount = bigint;

/** Block information from the RPC. */
export interface BlockInfo {
  height: number;
  hash: HashHex;
  prev_hash: HashHex;
  timestamp: number;
  proposer: PubKeyHex;
  commitment_count: number;
  registration_count: number;
  anchor_count: number;
  fraud_proof_count: number;
  name_registration_count: number;
  transfer_count: number;
  token_definition_count: number;
  token_mint_count: number;
  token_burn_count: number;
  loom_deploy_count: number;
  stake_operation_count: number;
  state_root: HashHex;
  /** Block production time in microseconds (only present for blocks produced by the connected node). */
  production_us?: number | null;
}

/** Current weave state. */
export interface WeaveStateInfo {
  height: number;
  latest_hash: HashHex;
  threads_root: HashHex;
  thread_count: number;
  base_fee: string;
  fee_multiplier: number;
}

/** Thread information. */
export interface ThreadInfo {
  thread_id: HashHex;
  owner: PubKeyHex;
  version: number;
  state_hash: HashHex;
}

/** Thread state with balances. */
export interface ThreadStateInfo {
  thread_id: HashHex;
  owner: PubKeyHex;
  version: number;
  state_hash: HashHex;
  balances: BalanceEntry[];
}

/** A balance entry for a token. */
export interface BalanceEntry {
  token_id: HashHex;
  amount: string;
  human_readable: string;
}

/** Health check response. */
export interface HealthInfo {
  height: number;
  is_validator: boolean;
  thread_count: number;
  status: string;
  network: string;
  chain_id: string;
  version: string;
  block_time_target: number;
  last_block_production_us: number | null;
}

/** Validator info. */
export interface ValidatorInfo {
  pubkey: PubKeyHex;
  address: AddressHex;
  stake: string;
  active: boolean;
}

/** Validator set information. */
export interface ValidatorSetInfo {
  validators: ValidatorInfo[];
  total_stake: string;
  epoch: number;
}

/** Fee estimate response. */
export interface FeeEstimateInfo {
  fee_per_commitment: string;
  base_fee: string;
  fee_multiplier: number;
  /** Flat fee per transfer in nits (burned). */
  transfer_fee: string;
}

/** Commitment proof. */
export interface CommitmentProofInfo {
  thread_id: HashHex;
  key: HashHex;
  value: HashHex;
  siblings: HashHex[];
}

/** Transaction history entry. */
export interface TransactionHistoryEntry {
  knot_id: HashHex;
  from: AddressHex;
  to: AddressHex;
  token_id: HashHex;
  symbol: string;
  amount: string;
  human_readable: string;
  memo?: string;
  timestamp: number;
  block_height?: number;
  direction: "sent" | "received";
}

/** Name resolution result. */
export interface NameResolution {
  name: string;
  owner: AddressHex;
  registered_at: number;
  fee_paid: string;
}

/** Name info. */
export interface NameInfo {
  name: string;
  registered_at: number;
}

/** Token information. */
export interface TokenInfo {
  token_id: HashHex;
  name: string;
  symbol: string;
  decimals: number;
  max_supply: string;
  current_supply: string;
  creator: AddressHex;
  created_at: number;
}

/** Loom (smart contract) information. */
export interface LoomInfo {
  loom_id: HashHex;
  name: string;
  operator: PubKeyHex;
  active: boolean;
  deployed_at: number;
  has_bytecode: boolean;
  participant_count: number;
  code_hash?: string;
}

/** Event attribute. */
export interface AttributeInfo {
  key: string;
  value: string;
}

/** Structured event from a contract. */
export interface EventInfo {
  type: string;
  attributes: AttributeInfo[];
}

/** Loom execution result. */
export interface ExecutionResult {
  success: boolean;
  output_hex?: string;
  gas_used: number;
  logs: string[];
  events: EventInfo[];
  reason?: string;
}

/** Loom query result. */
export interface QueryResult {
  success: boolean;
  output_hex?: string;
  gas_used: number;
  logs: string[];
  events: EventInfo[];
  reason?: string;
}

/** Submit result. */
export interface SubmitResult {
  success: boolean;
  reason?: string;
}

/** Staking information. */
export interface StakingInfo {
  validators: ValidatorStakeInfo[];
  total_staked: string;
  min_stake: string;
  bonding_period: number;
}

/** Per-validator staking details. */
export interface ValidatorStakeInfo {
  pubkey: PubKeyHex;
  address: AddressHex;
  stake: string;
  active: boolean;
}

/** State proof for a balance. */
export interface StateProofInfo {
  address: AddressHex;
  token_id: HashHex;
  balance: string;
  state_root: HashHex;
  proof: HashHex[];
}

/** Node info response. */
export interface NodeInfo {
  version: string;
  network: string;
  chain_id: string;
  is_validator: boolean;
  height: number;
  peer_count: number;
}

/** Detailed block transactions returned by norn_getBlockTransactions. */
export interface BlockTransactionsInfo {
  height: number;
  hash: HashHex;
  timestamp: number;
  transfers: BlockTransferInfo[];
  token_definitions: BlockTokenDefinitionInfo[];
  token_mints: BlockTokenMintInfo[];
  token_burns: BlockTokenBurnInfo[];
  name_registrations: BlockNameRegistrationInfo[];
  loom_deploys: BlockLoomDeployInfo[];
}

/** A transfer within a block. */
export interface BlockTransferInfo {
  from: AddressHex;
  to: AddressHex;
  token_id: HashHex;
  symbol: string;
  amount: string;
  human_readable: string;
  memo?: string;
  knot_id: HashHex;
  timestamp: number;
}

/** A token definition within a block. */
export interface BlockTokenDefinitionInfo {
  name: string;
  symbol: string;
  decimals: number;
  max_supply: string;
  initial_supply: string;
  creator: AddressHex;
  timestamp: number;
}

/** A token mint within a block. */
export interface BlockTokenMintInfo {
  token_id: HashHex;
  symbol: string;
  to: AddressHex;
  amount: string;
  human_readable: string;
  timestamp: number;
}

/** A token burn within a block. */
export interface BlockTokenBurnInfo {
  token_id: HashHex;
  symbol: string;
  burner: AddressHex;
  amount: string;
  human_readable: string;
  timestamp: number;
}

/** A name registration within a block. */
export interface BlockNameRegistrationInfo {
  name: string;
  owner: AddressHex;
  fee_paid: string;
  timestamp: number;
}

/** A loom deployment within a block. */
export interface BlockLoomDeployInfo {
  name: string;
  operator: string;
  timestamp: number;
}

/** Real-time transfer event (WebSocket). */
export interface TransferEvent {
  from: AddressHex;
  to: AddressHex;
  amount: string;
  human_readable: string;
  token_id?: HashHex;
  symbol?: string;
  memo?: string;
  block_height?: number | null;
}

/** Real-time token event (WebSocket). */
export interface TokenEvent {
  event_type: "created" | "minted" | "burned";
  token_id: HashHex;
  symbol: string;
  actor: AddressHex;
  amount?: string;
  human_readable?: string;
  block_height: number;
}

/** Real-time loom execution event (WebSocket). */
export interface LoomExecutionEvent {
  loom_id: HashHex;
  caller: AddressHex;
  gas_used: number;
  events: EventInfo[];
  block_height: number;
}

/** A Nostr-inspired signed chat event (Ed25519 + BLAKE3). */
export interface ChatEvent {
  /** BLAKE3 hash of [pubkey, created_at, kind, tags_json, content] as hex. */
  id: string;
  /** Author's Ed25519 pubkey (hex). */
  pubkey: string;
  /** Unix timestamp in seconds. */
  created_at: number;
  /** Event kind (30000=profile, 30001=DM, 30002=channel create, 30003=channel message). */
  kind: number;
  /** Nostr-style tags. */
  tags: string[][];
  /** Plaintext or base64 ciphertext. */
  content: string;
  /** Ed25519 signature over id bytes (hex). */
  sig: string;
}

/** Filter for querying chat history from the node. */
export interface ChatHistoryFilter {
  /** Filter by event kinds. */
  kinds?: number[];
  /** Filter by channel ID (for channel messages). */
  channel_id?: string;
  /** Filter by pubkey (matches author or recipient tag). */
  pubkey?: string;
  /** Only return events after this timestamp. */
  since?: number;
  /** Max events to return (default 100, max 500). */
  limit?: number;
}

/** Real-time pending transaction event (WebSocket). */
export interface PendingTransactionEvent {
  tx_type: string;
  hash: HashHex;
  from: AddressHex;
  timestamp: number;
}
