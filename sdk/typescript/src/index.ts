export {
  blake3Hash,
  blake3Kdf,
  ed25519Sign,
  ed25519Verify,
  publicKeyFromPrivate,
  publicKeyToAddress,
  toHex,
  fromHex,
  addressToHex,
  hexToAddress,
} from "./crypto.js";

export { Wallet } from "./wallet.js";

export {
  BorshWriter,
  BorshReader,
  transferSigningData,
  nameRegistrationSigningData,
  tokenDefinitionSigningData,
  tokenMintSigningData,
  tokenBurnSigningData,
  loomDeploySigningData,
} from "./borsh.js";

export { NornClient } from "./client.js";
export type { NornClientOptions } from "./client.js";

export {
  verifyStateProof,
  verifyBalanceProof,
  getBit,
  hashLeaf,
  hashInternal,
  smtKey,
  encodeU128LE,
} from "./merkle.js";

export {
  buildTransfer,
  buildNameRegistration,
  buildTokenDefinition,
  buildTokenMint,
  buildTokenBurn,
  buildLoomRegistration,
  parseAmount,
  formatAmount,
} from "./builders.js";

export {
  Subscription,
  subscribeNewBlocks,
  subscribeTransfers,
  subscribeTokenEvents,
  subscribeLoomEvents,
  subscribePendingTransactions,
  subscribeChatEvents,
} from "./subscriptions.js";
export type { SubscribeOptions } from "./subscriptions.js";

export {
  ed25519ToX25519Secret,
  ed25519ToX25519Public,
  encrypt,
  decrypt,
  deriveSharedSecret,
  symmetricEncrypt,
  symmetricDecrypt,
} from "./encryption.js";
export type { EncryptedMessage, SymmetricEncryptedMessage } from "./encryption.js";

export {
  createChatEvent,
  verifyChatEvent,
  encryptDmContent,
  decryptDmContent,
} from "./chat.js";

export type {
  AddressHex,
  HashHex,
  PubKeyHex,
  Amount,
  BlockInfo,
  BlockTransactionsInfo,
  BlockTransferInfo,
  BlockTokenDefinitionInfo,
  BlockTokenMintInfo,
  BlockTokenBurnInfo,
  BlockNameRegistrationInfo,
  BlockLoomDeployInfo,
  WeaveStateInfo,
  ThreadInfo,
  ThreadStateInfo,
  BalanceEntry,
  HealthInfo,
  ValidatorInfo,
  ValidatorSetInfo,
  FeeEstimateInfo,
  CommitmentProofInfo,
  TransactionHistoryEntry,
  NameResolution,
  NameInfo,
  TokenInfo,
  LoomInfo,
  AttributeInfo,
  EventInfo,
  ExecutionResult,
  QueryResult,
  SubmitResult,
  StakingInfo,
  ValidatorStakeInfo,
  StateProofInfo,
  NodeInfo,
  TransferEvent,
  TokenEvent,
  LoomExecutionEvent,
  PendingTransactionEvent,
  ChatEvent,
} from "./types.js";
