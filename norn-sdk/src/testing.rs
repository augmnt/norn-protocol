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
//!     let env = TestEnv::new().with_sender([1u8; 20]);
//!     let ctx = env.ctx();
//!     // ... use ctx with your contract ...
//!     assert!(env.logs().iter().any(|l| l.contains("action")));
//! }
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use borsh::BorshDeserialize;

use crate::contract::Context;
use crate::error::ContractError;
use crate::host;
use crate::response::Response;
use crate::types::Address;

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
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

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
