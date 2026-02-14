//! Test harness for loom contracts.
//!
//! Provides [`TestEnv`] for setting up a mock environment with working storage,
//! sender/block/timestamp state, and log capture. Use with `Item`/`Map` and
//! the `Contract` trait for full native unit tests.
//!
//! ```ignore
//! use norn_sdk::testing::*;
//! use norn_sdk::prelude::*;
//!
//! #[test]
//! fn test_something() {
//!     let env = TestEnv::new().with_sender(ALICE);
//!     let ctx = env.ctx();
//!     // ... use ctx with your contract ...
//!     assert!(env.logs().iter().any(|l| l.contains("action")));
//! }
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

use borsh::BorshDeserialize;

use crate::contract::Context;
use crate::error::ContractError;
use crate::host;
use crate::response::Response;
use crate::types::Address;

// ═══════════════════════════════════════════════════════════════════════════
// Test address constants
// ═══════════════════════════════════════════════════════════════════════════

/// Test address constant for the first actor.
pub const ALICE: Address = [1u8; 20];
/// Test address constant for the second actor.
pub const BOB: Address = [2u8; 20];
/// Test address constant for the third actor.
pub const CHARLIE: Address = [3u8; 20];
/// Test address constant for the fourth actor.
pub const DAVE: Address = [4u8; 20];

// ═══════════════════════════════════════════════════════════════════════════
// TestEnv
// ═══════════════════════════════════════════════════════════════════════════

/// Test environment that resets all mock state on creation.
///
/// Each `TestEnv::new()` clears storage, logs, sender, block height, and
/// timestamp. Call builder methods to configure the initial state, then
/// use `ctx()` to get a `Context` for passing to contract methods.
pub struct TestEnv {
    _private: (),
}

impl TestEnv {
    /// Create a new test environment, resetting all mock state.
    pub fn new() -> Self {
        host::mock_reset();
        TestEnv { _private: () }
    }

    /// Set the sender address (builder, consuming).
    pub fn with_sender(self, addr: Address) -> Self {
        host::mock_set_sender(addr);
        self
    }

    /// Set the block height (builder, consuming).
    pub fn with_block_height(self, h: u64) -> Self {
        host::mock_set_block_height(h);
        self
    }

    /// Set the block timestamp (builder, consuming).
    pub fn with_timestamp(self, t: u64) -> Self {
        host::mock_set_timestamp(t);
        self
    }

    /// Change the sender address mid-test (non-consuming).
    pub fn set_sender(&self, addr: Address) {
        host::mock_set_sender(addr);
    }

    /// Change the block height mid-test (non-consuming).
    pub fn set_block_height(&self, h: u64) {
        host::mock_set_block_height(h);
    }

    /// Change the timestamp mid-test (non-consuming).
    pub fn set_timestamp(&self, t: u64) {
        host::mock_set_timestamp(t);
    }

    /// Set the contract's own address (for testing contract custody).
    pub fn with_contract_address(self, addr: Address) -> Self {
        host::mock_set_contract_address(addr);
        self
    }

    /// Change the contract address mid-test (non-consuming).
    pub fn set_contract_address(&self, addr: Address) {
        host::mock_set_contract_address(addr);
    }

    /// Build a `Context` from the current mock state.
    pub fn ctx(&self) -> Context {
        Context::new()
    }

    /// Get all log messages captured since the last reset.
    pub fn logs(&self) -> Vec<String> {
        host::mock_get_logs()
    }

    /// Clear captured log messages.
    pub fn clear_logs(&self) {
        host::mock_reset_logs();
    }

    /// Get all events captured since the last reset.
    pub fn events(&self) -> Vec<host::MockEvent> {
        host::mock_get_events()
    }

    /// Clear captured events.
    pub fn clear_events(&self) {
        host::mock_reset_events();
    }

    /// Get all transfers captured since the last reset.
    pub fn transfers(&self) -> Vec<host::MockTransfer> {
        host::mock_get_transfers()
    }

    /// Clear captured transfers.
    pub fn clear_transfers(&self) {
        host::mock_reset_transfers();
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Assertion helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Assert that a `Response` contains an attribute with the given key and value.
///
/// Panics with a descriptive message if the attribute is not found.
pub fn assert_attribute(response: &Response, key: &str, value: &str) {
    for attr in response.attributes() {
        if attr.key == key && attr.value == value {
            return;
        }
    }
    panic!(
        "expected attribute {}={}, found: [{}]",
        key,
        value,
        response
            .attributes()
            .iter()
            .map(|a| alloc::format!("{}={}", a.key, a.value))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Deserialize the data from a `Response` as a borsh-encoded value.
pub fn from_response<T: BorshDeserialize>(response: &Response) -> Result<T, ContractError> {
    BorshDeserialize::try_from_slice(response.data())
        .map_err(|e| ContractError::Custom(alloc::format!("deserialize response: {e}")))
}

/// Assert that a `Response` contains borsh-encoded data equal to `expected`.
///
/// Combines `from_response().unwrap()` + `assert_eq!` into one call.
pub fn assert_data<T: BorshDeserialize + Debug + PartialEq>(response: &Response, expected: &T) {
    let actual: T = from_response(response).expect("assert_data: failed to deserialize response");
    assert_eq!(&actual, expected);
}

/// Assert that a `ContractError`'s message contains the given substring.
pub fn assert_err_contains(err: &ContractError, substring: &str) {
    let msg = err.message();
    assert!(
        msg.contains(substring),
        "expected error containing '{}', got: '{}'",
        substring,
        msg
    );
}

/// Assert that a `Response` contains an event with the given type name.
///
/// Panics with a descriptive message if the event is not found.
pub fn assert_event(response: &Response, ty: &str) {
    for event in response.events() {
        if event.ty == ty {
            return;
        }
    }
    panic!(
        "expected event '{}', found: [{}]",
        ty,
        response
            .events()
            .iter()
            .map(|e| e.ty.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Assert that a `Response` contains an event with the given type and attribute.
///
/// Panics with a descriptive message if the event/attribute is not found.
pub fn assert_event_attribute(response: &Response, ty: &str, key: &str, value: &str) {
    for event in response.events() {
        if event.ty == ty {
            for attr in &event.attributes {
                if attr.key == key && attr.value == value {
                    return;
                }
            }
        }
    }
    panic!(
        "expected event '{}' with attribute {}={}, found events: [{}]",
        ty,
        key,
        value,
        response
            .events()
            .iter()
            .map(|e| alloc::format!(
                "{}({})",
                e.ty,
                e.attributes
                    .iter()
                    .map(|a| alloc::format!("{}={}", a.key, a.value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
}
