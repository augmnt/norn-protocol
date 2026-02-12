import type {
  BlockInfo,
  BlockTransactionsInfo,
  WeaveStateInfo,
  ThreadInfo,
  ThreadStateInfo,
  HealthInfo,
  ValidatorSetInfo,
  FeeEstimateInfo,
  CommitmentProofInfo,
  TransactionHistoryEntry,
  NameResolution,
  NameInfo,
  TokenInfo,
  LoomInfo,
  ExecutionResult,
  QueryResult,
  SubmitResult,
  StakingInfo,
  StateProofInfo,
  NodeInfo,
  AddressHex,
  HashHex,
} from "./types.js";

/** Options for creating a NornClient. */
export interface NornClientOptions {
  /** RPC endpoint URL (e.g. "http://localhost:9944"). */
  url: string;
  /** Optional API key for authenticated (mutation) requests. */
  apiKey?: string;
  /** Request timeout in milliseconds (default: 10000). */
  timeout?: number;
}

/** JSON-RPC request body. */
interface JsonRpcRequest {
  jsonrpc: "2.0";
  method: string;
  params: unknown[];
  id: number;
}

/** JSON-RPC response body. */
interface JsonRpcResponse<T = unknown> {
  jsonrpc: "2.0";
  result?: T;
  error?: { code: number; message: string };
  id: number;
}

/**
 * TypeScript client for the Norn Protocol JSON-RPC API.
 *
 * Provides typed methods for all 40+ RPC endpoints.
 */
export class NornClient {
  private url: string;
  private apiKey?: string;
  private timeout: number;
  private nextId = 1;

  constructor(options: NornClientOptions) {
    this.url = options.url;
    this.apiKey = options.apiKey;
    this.timeout = options.timeout ?? 10_000;
  }

  // ── Internal RPC call ─────────────────────────────────────────────────

  private async call<T>(method: string, params: unknown[] = []): Promise<T> {
    const body: JsonRpcRequest = {
      jsonrpc: "2.0",
      method,
      params,
      id: this.nextId++,
    };

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };
    if (this.apiKey) {
      headers["Authorization"] = `Bearer ${this.apiKey}`;
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(this.url, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const json = (await response.json()) as JsonRpcResponse<T>;
      if (json.error) {
        throw new Error(`RPC error ${json.error.code}: ${json.error.message}`);
      }

      return json.result as T;
    } finally {
      clearTimeout(timer);
    }
  }

  // ── Read-only methods ─────────────────────────────────────────────────

  /** Get the balance of an address for a token. */
  async getBalance(address: AddressHex, tokenId: HashHex): Promise<string> {
    return this.call("norn_getBalance", [address, tokenId]);
  }

  /** Get block by height. */
  async getBlock(height: number): Promise<BlockInfo> {
    return this.call("norn_getBlock", [height]);
  }

  /** Get detailed transactions for a block by height. */
  async getBlockTransactions(
    height: number,
  ): Promise<BlockTransactionsInfo | null> {
    return this.call("norn_getBlockTransactions", [height]);
  }

  /** Get the latest block. */
  async getLatestBlock(): Promise<BlockInfo> {
    return this.call("norn_getLatestBlock");
  }

  /** Get the current weave state. */
  async getWeaveState(): Promise<WeaveStateInfo> {
    return this.call("norn_getWeaveState");
  }

  /** Get a thread by ID. */
  async getThread(threadId: HashHex): Promise<ThreadInfo> {
    return this.call("norn_getThread", [threadId]);
  }

  /** Get a thread's state with balances. */
  async getThreadState(threadId: HashHex): Promise<ThreadStateInfo> {
    return this.call("norn_getThreadState", [threadId]);
  }

  /** Health check. */
  async health(): Promise<HealthInfo> {
    return this.call("norn_health");
  }

  /** Get the validator set. */
  async getValidatorSet(): Promise<ValidatorSetInfo> {
    return this.call("norn_getValidatorSet");
  }

  /** Get fee estimate. */
  async getFeeEstimate(): Promise<FeeEstimateInfo> {
    return this.call("norn_getFeeEstimate");
  }

  /** Get a commitment proof for a thread. */
  async getCommitmentProof(threadId: HashHex): Promise<CommitmentProofInfo> {
    return this.call("norn_getCommitmentProof", [threadId]);
  }

  /** Get transaction history for an address. */
  async getTransactionHistory(
    address: AddressHex,
    limit = 20,
    offset = 0,
  ): Promise<TransactionHistoryEntry[]> {
    return this.call("norn_getTransactionHistory", [address, limit, offset]);
  }

  /** Resolve a registered name to its owner. */
  async resolveName(name: string): Promise<NameResolution | null> {
    return this.call("norn_resolveName", [name]);
  }

  /** List names owned by an address. */
  async listNames(address: AddressHex): Promise<NameInfo[]> {
    return this.call("norn_listNames", [address]);
  }

  /** Get Prometheus-style metrics. */
  async getMetrics(): Promise<string> {
    return this.call("norn_getMetrics");
  }

  /** Get node info. */
  async getNodeInfo(): Promise<NodeInfo> {
    return this.call("norn_getNodeInfo");
  }

  /** Get token information by ID. */
  async getTokenInfo(tokenId: HashHex): Promise<TokenInfo | null> {
    return this.call("norn_getTokenInfo", [tokenId]);
  }

  /** Get token information by symbol. */
  async getTokenBySymbol(symbol: string): Promise<TokenInfo | null> {
    return this.call("norn_getTokenBySymbol", [symbol]);
  }

  /** List all tokens. */
  async listTokens(limit = 20, offset = 0): Promise<TokenInfo[]> {
    return this.call("norn_listTokens", [limit, offset]);
  }

  /** Get loom (contract) information by ID. */
  async getLoomInfo(loomId: HashHex): Promise<LoomInfo | null> {
    return this.call("norn_getLoomInfo", [loomId]);
  }

  /** List all deployed looms. */
  async listLooms(limit = 20, offset = 0): Promise<LoomInfo[]> {
    return this.call("norn_listLooms", [limit, offset]);
  }

  /** Query a loom contract (read-only). */
  async queryLoom(loomId: HashHex, inputHex: string): Promise<QueryResult> {
    return this.call("norn_queryLoom", [loomId, inputHex]);
  }

  /** Get staking information. */
  async getStakingInfo(pubkeyHex?: string): Promise<StakingInfo> {
    return this.call("norn_getStakingInfo", [pubkeyHex ?? null]);
  }

  /** Get the current state root. */
  async getStateRoot(): Promise<{ state_root: HashHex }> {
    return this.call("norn_getStateRoot");
  }

  /** Get a state proof for an address and token. */
  async getStateProof(
    address: AddressHex,
    tokenId?: HashHex,
  ): Promise<StateProofInfo> {
    const params: unknown[] = [address];
    if (tokenId) params.push(tokenId);
    return this.call("norn_getStateProof", params);
  }

  /** Get recent transfers. */
  async getRecentTransfers(
    limit = 20,
    offset = 0,
  ): Promise<TransactionHistoryEntry[]> {
    return this.call("norn_getRecentTransfers", [limit, offset]);
  }

  /** Get a single transaction by its knot ID (hex). */
  async getTransaction(
    knotId: string,
  ): Promise<TransactionHistoryEntry | null> {
    return this.call("norn_getTransaction", [knotId]);
  }

  // ── Mutation methods ──────────────────────────────────────────────────

  /** Submit a knot (transfer). */
  async submitKnot(knotHex: string): Promise<SubmitResult> {
    return this.call("norn_submitKnot", [knotHex]);
  }

  /** Register a name. */
  async registerName(
    name: string,
    ownerHex: string,
    knotHex: string,
  ): Promise<SubmitResult> {
    return this.call("norn_registerName", [name, ownerHex, knotHex]);
  }

  /** Create a token. */
  async createToken(definitionHex: string): Promise<SubmitResult> {
    return this.call("norn_createToken", [definitionHex]);
  }

  /** Mint tokens. */
  async mintToken(mintHex: string): Promise<SubmitResult> {
    return this.call("norn_mintToken", [mintHex]);
  }

  /** Burn tokens. */
  async burnToken(burnHex: string): Promise<SubmitResult> {
    return this.call("norn_burnToken", [burnHex]);
  }

  /** Deploy a loom (smart contract). */
  async deployLoom(registrationHex: string): Promise<SubmitResult> {
    return this.call("norn_deployLoom", [registrationHex]);
  }

  /** Upload bytecode to a loom. */
  async uploadBytecode(
    loomId: HashHex,
    bytecodeHex: string,
    initMsgHex?: string,
  ): Promise<SubmitResult> {
    const params: unknown[] = [loomId, bytecodeHex];
    if (initMsgHex) params.push(initMsgHex);
    return this.call("norn_uploadLoomBytecode", params);
  }

  /** Execute a loom contract. */
  async executeLoom(
    loomId: HashHex,
    inputHex: string,
    senderHex: AddressHex,
  ): Promise<ExecutionResult> {
    return this.call("norn_executeLoom", [loomId, inputHex, senderHex]);
  }

  /** Join a loom as a participant. */
  async joinLoom(
    loomId: string,
    participant: string,
    pubkey: string,
  ): Promise<SubmitResult> {
    return this.call("norn_joinLoom", [loomId, participant, pubkey]);
  }

  /** Leave a loom. */
  async leaveLoom(loomId: string, participant: string): Promise<SubmitResult> {
    return this.call("norn_leaveLoom", [loomId, participant]);
  }

  /** Submit a commitment. */
  async submitCommitment(commitmentHex: string): Promise<SubmitResult> {
    return this.call("norn_submitCommitment", [commitmentHex]);
  }

  /** Submit a registration. */
  async submitRegistration(registrationHex: string): Promise<SubmitResult> {
    return this.call("norn_submitRegistration", [registrationHex]);
  }

  /** Stake NORN. */
  async stake(stakeHex: string): Promise<SubmitResult> {
    return this.call("norn_stake", [stakeHex]);
  }

  /** Unstake NORN. */
  async unstake(unstakeHex: string): Promise<SubmitResult> {
    return this.call("norn_unstake", [unstakeHex]);
  }

  /** Request testnet faucet tokens. */
  async faucet(address: AddressHex): Promise<SubmitResult> {
    return this.call("norn_faucet", [address]);
  }

  /** Submit a fraud proof. */
  async submitFraudProof(proofHex: string): Promise<SubmitResult> {
    return this.call("norn_submitFraudProof", [proofHex]);
  }
}
