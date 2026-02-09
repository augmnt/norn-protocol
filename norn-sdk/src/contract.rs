//! Core `Contract` trait and `Context` wrapper for loom smart contracts.
//!
//! The `Contract` trait defines the interface every loom contract implements.
//! The `Context` struct provides access to host functions (sender, block info,
//! logging, token transfers).

use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::ContractError;
use crate::response::ContractResult;
use crate::types::{Address, TokenId};

/// The core contract interface. Implement this trait to define your loom.
///
/// The SDK's `norn_entry!` macro generates the Wasm entry points (`init`,
/// `execute`, `query`) that deserialize messages, manage state persistence,
/// and call your trait methods.
pub trait Contract: BorshSerialize + BorshDeserialize {
    /// The message type for state-changing operations.
    type Exec: BorshDeserialize;
    /// The message type for read-only queries.
    type Query: BorshDeserialize;

    /// Initialize the contract. Called once when the loom is first set up.
    fn init(ctx: &Context) -> Self;

    /// Handle a state-changing execution message.
    fn execute(&mut self, ctx: &Context, msg: Self::Exec) -> ContractResult;

    /// Handle a read-only query message.
    fn query(&self, ctx: &Context, msg: Self::Query) -> ContractResult;
}

// ---------------------------------------------------------------------------
// Context — wasm32 implementation (real host calls)
// ---------------------------------------------------------------------------

/// Execution context providing access to the Norn runtime environment.
///
/// On `wasm32` targets this calls real host functions. On native targets (for
/// testing) it reads from thread-local mock state set via [`TestEnv`](crate::testing::TestEnv)
/// or [`Context::mock()`].
#[cfg(target_arch = "wasm32")]
pub struct Context {
    _private: (),
}

#[cfg(target_arch = "wasm32")]
impl Context {
    /// Create a context for use inside the Wasm runtime.
    #[doc(hidden)]
    pub fn new() -> Self {
        Context { _private: () }
    }

    /// Address of the account that submitted the transaction.
    pub fn sender(&self) -> Address {
        crate::host::sender()
    }

    /// Current block height.
    pub fn block_height(&self) -> u64 {
        crate::host::block_height()
    }

    /// Current block timestamp (unix seconds).
    pub fn timestamp(&self) -> u64 {
        crate::host::timestamp()
    }

    /// Emit a log message visible in execution results.
    pub fn log(&self, msg: &str) {
        crate::host::log(msg);
    }

    /// Transfer tokens between accounts.
    pub fn transfer(&self, from: &Address, to: &Address, token: &TokenId, amount: u128) {
        crate::host::transfer(from, to, token, amount);
    }

    /// Assert that the sender matches `expected`, returning `Unauthorized` if not.
    pub fn require_sender(&self, expected: &Address) -> Result<(), ContractError> {
        if self.sender() != *expected {
            Err(ContractError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Assert a condition, returning the given error if false.
    pub fn require(&self, cond: bool, err: ContractError) -> Result<(), ContractError> {
        if cond {
            Ok(())
        } else {
            Err(err)
        }
    }
}

// ---------------------------------------------------------------------------
// Context — native implementation (reads from mock thread-locals)
// ---------------------------------------------------------------------------

/// Execution context backed by thread-local mock state for unit testing.
///
/// On native targets, `Context::new()` reads the current sender, block height,
/// and timestamp from thread-local state managed by [`host`](crate::host) mock
/// functions. Use [`TestEnv`](crate::testing::TestEnv) or [`Context::mock()`]
/// to configure these values.
#[cfg(not(target_arch = "wasm32"))]
pub struct Context {
    sender_addr: Address,
    block_height_val: u64,
    timestamp_val: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl Context {
    /// Create a context from the current thread-local mock state.
    #[doc(hidden)]
    pub fn new() -> Self {
        Context {
            sender_addr: crate::host::sender(),
            block_height_val: crate::host::block_height(),
            timestamp_val: crate::host::timestamp(),
        }
    }

    /// Start building a mock context for unit tests.
    ///
    /// The builder also sets thread-local state so that `Item`/`Map` storage
    /// operations and `host::sender()` return consistent values.
    ///
    /// ```ignore
    /// let ctx = Context::mock().sender([1u8; 20]).block_height(42).build();
    /// assert_eq!(ctx.block_height(), 42);
    /// ```
    pub fn mock() -> MockContextBuilder {
        MockContextBuilder {
            sender_addr: [0u8; 20],
            block_height_val: 0,
            timestamp_val: 0,
        }
    }

    /// Address of the account that submitted the transaction.
    pub fn sender(&self) -> Address {
        self.sender_addr
    }

    /// Current block height.
    pub fn block_height(&self) -> u64 {
        self.block_height_val
    }

    /// Current block timestamp (unix seconds).
    pub fn timestamp(&self) -> u64 {
        self.timestamp_val
    }

    /// Emit a log message (captured in thread-local logs, accessible via `TestEnv::logs()`).
    pub fn log(&self, msg: &str) {
        crate::host::log(msg);
    }

    /// Transfer tokens (captured in thread-local log for test assertions).
    pub fn transfer(&self, from: &Address, to: &Address, token: &TokenId, amount: u128) {
        crate::host::transfer(from, to, token, amount);
    }

    /// Assert that the sender matches `expected`, returning `Unauthorized` if not.
    pub fn require_sender(&self, expected: &Address) -> Result<(), ContractError> {
        if self.sender() != *expected {
            Err(ContractError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Assert a condition, returning the given error if false.
    pub fn require(&self, cond: bool, err: ContractError) -> Result<(), ContractError> {
        if cond {
            Ok(())
        } else {
            Err(err)
        }
    }
}

/// Builder for constructing a mock [`Context`] in unit tests.
///
/// Also sets thread-local mock state so host functions and storage
/// primitives see the configured values.
#[cfg(not(target_arch = "wasm32"))]
pub struct MockContextBuilder {
    sender_addr: Address,
    block_height_val: u64,
    timestamp_val: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl MockContextBuilder {
    /// Set the sender address.
    pub fn sender(mut self, addr: Address) -> Self {
        self.sender_addr = addr;
        self
    }

    /// Set the block height.
    pub fn block_height(mut self, h: u64) -> Self {
        self.block_height_val = h;
        self
    }

    /// Set the block timestamp.
    pub fn timestamp(mut self, t: u64) -> Self {
        self.timestamp_val = t;
        self
    }

    /// Build the mock context, also updating thread-local mock state.
    pub fn build(self) -> Context {
        crate::host::mock_set_sender(self.sender_addr);
        crate::host::mock_set_block_height(self.block_height_val);
        crate::host::mock_set_timestamp(self.timestamp_val);
        Context {
            sender_addr: self.sender_addr,
            block_height_val: self.block_height_val,
            timestamp_val: self.timestamp_val,
        }
    }
}

/// Helper: serialize a contract error into bytes for the output buffer.
/// Used by the `norn_entry!` macro for error output.
pub fn error_to_bytes(err: &ContractError) -> alloc::vec::Vec<u8> {
    let msg = err.message();
    let mut out = alloc::vec::Vec::with_capacity(1 + msg.len());
    out.push(1); // error marker byte
    out.extend_from_slice(msg.as_bytes());
    out
}
