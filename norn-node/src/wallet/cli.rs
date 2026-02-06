use clap::Subcommand;

/// Wallet subcommands.
#[derive(Subcommand)]
pub enum WalletCommand {
    /// Create a new wallet with a fresh mnemonic
    Create {
        /// Wallet name
        #[arg(long)]
        name: String,
        /// Optional BIP-39 passphrase (NOT the encryption password)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Import a wallet from mnemonic or private key
    Import {
        /// Import from mnemonic phrase
        #[arg(long, conflicts_with = "private_key")]
        mnemonic: bool,
        /// Import from hex-encoded private key (32 bytes)
        #[arg(long, conflicts_with = "mnemonic")]
        private_key: Option<String>,
        /// Wallet name
        #[arg(long, default_value = "imported")]
        name: String,
        /// Optional BIP-39 passphrase for mnemonic import
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Export wallet secrets (mnemonic or private key)
    Export {
        /// Wallet name (defaults to active wallet)
        #[arg(long)]
        name: Option<String>,
        /// Show the mnemonic phrase
        #[arg(long, conflicts_with = "show_private_key")]
        show_mnemonic: bool,
        /// Show the private key
        #[arg(long, conflicts_with = "show_mnemonic")]
        show_private_key: bool,
    },
    /// List all wallets
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set the active wallet
    Use {
        /// Wallet name to activate
        name: String,
    },
    /// Delete a wallet
    Delete {
        /// Wallet name to delete
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Show wallet address and public key
    Address {
        /// Wallet name (defaults to active wallet)
        #[arg(long)]
        name: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Query balance for an address
    Balance {
        /// Address to query (defaults to active wallet)
        #[arg(long)]
        address: Option<String>,
        /// Token ID (defaults to native NORN)
        #[arg(long)]
        token: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Transfer tokens to another address
    Transfer {
        /// Recipient address
        #[arg(long)]
        to: String,
        /// Amount to transfer (human-readable, e.g. "10.5")
        #[arg(long)]
        amount: String,
        /// Token ID (defaults to native NORN)
        #[arg(long)]
        token: Option<String>,
        /// Optional memo
        #[arg(long)]
        memo: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Register a thread on the weave
    Register {
        /// Wallet name (defaults to active wallet)
        #[arg(long)]
        name: Option<String>,
    },
    /// Commit pending thread state to the weave
    Commit {
        /// Wallet name (defaults to active wallet)
        #[arg(long)]
        name: Option<String>,
    },
    /// Show thread registration and commitment status
    Status {
        /// Wallet name (defaults to active wallet)
        #[arg(long)]
        name: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show transaction history
    History {
        /// Maximum entries to show
        #[arg(long, default_value = "20")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Request testnet tokens from faucet
    Faucet {
        /// Address to fund (defaults to active wallet)
        #[arg(long)]
        address: Option<String>,
    },
    /// Query a block by height
    Block {
        /// Block height (omit for latest)
        height: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show current weave state
    WeaveState {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get or set wallet configuration
    Config {
        /// Set RPC URL
        #[arg(long)]
        rpc_url: Option<String>,
        /// Show current config as JSON
        #[arg(long)]
        json: bool,
    },
}
