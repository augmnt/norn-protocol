//! Safe Rust wrappers around the Norn host functions.
//!
//! On `wasm32` targets these call real host imports provided by the Norn runtime.
//! On native targets (for `cargo test`) they use `thread_local!` storage so that
//! [`Item`](crate::storage::Item), [`Map`](crate::storage::Map), and the
//! [`Contract`](crate::Contract) trait work in unit tests.

#[allow(unused_imports)]
use alloc::vec;
use alloc::vec::Vec;

// ── Raw extern declarations (wasm32 only) ──────────────────────────────────

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn norn_log(msg_ptr: i32, msg_len: i32);
    fn norn_state_get(key_ptr: i32, key_len: i32, out_ptr: i32, out_max_len: i32) -> i32;
    fn norn_state_set(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32);
    fn norn_transfer(from_ptr: i32, to_ptr: i32, token_ptr: i32, amount: i64);
    fn norn_sender(out_ptr: i32);
    fn norn_block_height() -> i64;
    fn norn_timestamp() -> i64;
    fn norn_emit_event(type_ptr: i32, type_len: i32, data_ptr: i32, data_len: i32);
}

// ═══════════════════════════════════════════════════════════════════════════
// wasm32 implementations — real host calls
// ═══════════════════════════════════════════════════════════════════════════

/// Emit a log message visible in execution results.
#[cfg(target_arch = "wasm32")]
pub fn log(msg: &str) {
    unsafe {
        norn_log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Read a value from contract state.
#[cfg(target_arch = "wasm32")]
pub fn state_get(key: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        let len = norn_state_get(key.as_ptr() as i32, key.len() as i32, 0, 0);
        if len < 0 {
            return None;
        }
        let len = len as usize;
        if len == 0 {
            return Some(vec![]);
        }
        let mut buf = vec![0u8; len];
        let result = norn_state_get(
            key.as_ptr() as i32,
            key.len() as i32,
            buf.as_mut_ptr() as i32,
            len as i32,
        );
        if result < 0 {
            return None;
        }
        Some(buf)
    }
}

/// Write a value to contract state.
#[cfg(target_arch = "wasm32")]
pub fn state_set(key: &[u8], value: &[u8]) {
    unsafe {
        norn_state_set(
            key.as_ptr() as i32,
            key.len() as i32,
            value.as_ptr() as i32,
            value.len() as i32,
        );
    }
}

/// Remove a key from contract state (writes empty value on wasm32).
#[cfg(target_arch = "wasm32")]
pub fn state_remove(key: &[u8]) {
    state_set(key, &[]);
}

/// Transfer tokens.
#[cfg(target_arch = "wasm32")]
pub fn transfer(from: &[u8; 20], to: &[u8; 20], token_id: &[u8; 32], amount: u128) {
    unsafe {
        norn_transfer(
            from.as_ptr() as i32,
            to.as_ptr() as i32,
            token_id.as_ptr() as i32,
            amount as i64,
        );
    }
}

/// Get the address of the transaction sender.
#[cfg(target_arch = "wasm32")]
pub fn sender() -> [u8; 20] {
    let mut addr = [0u8; 20];
    unsafe {
        norn_sender(addr.as_mut_ptr() as i32);
    }
    addr
}

/// Get the current block height.
#[cfg(target_arch = "wasm32")]
pub fn block_height() -> u64 {
    unsafe { norn_block_height() as u64 }
}

/// Get the current block timestamp (unix seconds).
#[cfg(target_arch = "wasm32")]
pub fn timestamp() -> u64 {
    unsafe { norn_timestamp() as u64 }
}

/// Emit a structured event with key-value attributes.
///
/// The type name is passed as a string, and the attributes are borsh-serialized
/// as `Vec<(String, String)>`.
#[cfg(target_arch = "wasm32")]
pub fn emit_event(ty: &str, attributes: &[crate::response::Attribute]) {
    let pairs: Vec<(alloc::string::String, alloc::string::String)> = attributes
        .iter()
        .map(|a| (a.key.clone(), a.value.clone()))
        .collect();
    let data = borsh::to_vec(&pairs).unwrap_or_default();
    unsafe {
        norn_emit_event(
            ty.as_ptr() as i32,
            ty.len() as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Native implementations — thread-local mock storage for `cargo test`
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
mod mock {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::string::String;
    use std::vec::Vec;

    type TransferRecord = (Vec<u8>, Vec<u8>, Vec<u8>, u128);

    /// A captured structured event (type + attributes).
    #[derive(Debug, Clone)]
    pub struct MockEvent {
        pub ty: String,
        pub attributes: Vec<(String, String)>,
    }

    std::thread_local! {
        static STATE: RefCell<BTreeMap<Vec<u8>, Vec<u8>>> = const { RefCell::new(BTreeMap::new()) };
        static LOGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static SENDER: RefCell<[u8; 20]> = const { RefCell::new([0u8; 20]) };
        static BLOCK_HEIGHT: RefCell<u64> = const { RefCell::new(0) };
        static TIMESTAMP: RefCell<u64> = const { RefCell::new(0) };
        static TRANSFERS: RefCell<Vec<TransferRecord>> = const { RefCell::new(Vec::new()) };
        static EVENTS: RefCell<Vec<MockEvent>> = const { RefCell::new(Vec::new()) };
    }

    // ── Host function implementations ──────────────────────────────────────

    pub fn log(msg: &str) {
        LOGS.with(|logs| logs.borrow_mut().push(String::from(msg)));
    }

    pub fn state_get(key: &[u8]) -> Option<Vec<u8>> {
        STATE.with(|state| state.borrow().get(key).cloned())
    }

    pub fn state_set(key: &[u8], value: &[u8]) {
        STATE.with(|state| {
            if value.is_empty() {
                state.borrow_mut().remove(key);
            } else {
                state.borrow_mut().insert(key.to_vec(), value.to_vec());
            }
        });
    }

    pub fn state_remove(key: &[u8]) {
        STATE.with(|state| {
            state.borrow_mut().remove(key);
        });
    }

    pub fn transfer(from: &[u8; 20], to: &[u8; 20], token_id: &[u8; 32], amount: u128) {
        TRANSFERS.with(|t| {
            t.borrow_mut()
                .push((from.to_vec(), to.to_vec(), token_id.to_vec(), amount));
        });
    }

    pub fn sender() -> [u8; 20] {
        SENDER.with(|s| *s.borrow())
    }

    pub fn block_height() -> u64 {
        BLOCK_HEIGHT.with(|h| *h.borrow())
    }

    pub fn timestamp() -> u64 {
        TIMESTAMP.with(|t| *t.borrow())
    }

    pub fn emit_event(ty: &str, attributes: &[crate::response::Attribute]) {
        let pairs: Vec<(String, String)> = attributes
            .iter()
            .map(|a| (a.key.clone(), a.value.clone()))
            .collect();
        EVENTS.with(|e| {
            e.borrow_mut().push(MockEvent {
                ty: String::from(ty),
                attributes: pairs,
            })
        });
    }

    // ── Mock control functions ─────────────────────────────────────────────

    pub fn mock_reset() {
        STATE.with(|s| s.borrow_mut().clear());
        LOGS.with(|l| l.borrow_mut().clear());
        SENDER.with(|s| *s.borrow_mut() = [0u8; 20]);
        BLOCK_HEIGHT.with(|h| *h.borrow_mut() = 0);
        TIMESTAMP.with(|t| *t.borrow_mut() = 0);
        TRANSFERS.with(|t| t.borrow_mut().clear());
        EVENTS.with(|e| e.borrow_mut().clear());
    }

    pub fn mock_set_sender(addr: [u8; 20]) {
        SENDER.with(|s| *s.borrow_mut() = addr);
    }

    pub fn mock_set_block_height(h: u64) {
        BLOCK_HEIGHT.with(|bh| *bh.borrow_mut() = h);
    }

    pub fn mock_set_timestamp(t: u64) {
        TIMESTAMP.with(|ts| *ts.borrow_mut() = t);
    }

    pub fn mock_get_logs() -> Vec<String> {
        LOGS.with(|l| l.borrow().clone())
    }

    pub fn mock_reset_logs() {
        LOGS.with(|l| l.borrow_mut().clear());
    }

    pub fn mock_get_events() -> Vec<MockEvent> {
        EVENTS.with(|e| e.borrow().clone())
    }

    pub fn mock_reset_events() {
        EVENTS.with(|e| e.borrow_mut().clear());
    }

    pub fn mock_get_transfers() -> Vec<TransferRecord> {
        TRANSFERS.with(|t| t.borrow().clone())
    }

    pub fn mock_reset_transfers() {
        TRANSFERS.with(|t| t.borrow_mut().clear());
    }
}

// ── Re-export native stubs as public module-level functions ────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn log(msg: &str) {
    mock::log(msg);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn state_get(key: &[u8]) -> Option<Vec<u8>> {
    mock::state_get(key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn state_set(key: &[u8], value: &[u8]) {
    mock::state_set(key, value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn state_remove(key: &[u8]) {
    mock::state_remove(key);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn transfer(from: &[u8; 20], to: &[u8; 20], token_id: &[u8; 32], amount: u128) {
    mock::transfer(from, to, token_id, amount);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sender() -> [u8; 20] {
    mock::sender()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn block_height() -> u64 {
    mock::block_height()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp() -> u64 {
    mock::timestamp()
}

// ── Mock control (native only, public) ─────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_reset() {
    mock::mock_reset();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_set_sender(addr: [u8; 20]) {
    mock::mock_set_sender(addr);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_set_block_height(h: u64) {
    mock::mock_set_block_height(h);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_set_timestamp(t: u64) {
    mock::mock_set_timestamp(t);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_get_logs() -> Vec<alloc::string::String> {
    mock::mock_get_logs()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_reset_logs() {
    mock::mock_reset_logs();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn emit_event(ty: &str, attributes: &[crate::response::Attribute]) {
    mock::emit_event(ty, attributes);
}

/// Event captured during mock execution.
#[cfg(not(target_arch = "wasm32"))]
pub use mock::MockEvent;

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_get_events() -> alloc::vec::Vec<MockEvent> {
    mock::mock_get_events()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_reset_events() {
    mock::mock_reset_events();
}

/// A captured transfer record: `(from, to, token_id, amount)`.
#[cfg(not(target_arch = "wasm32"))]
pub type MockTransfer = (
    alloc::vec::Vec<u8>,
    alloc::vec::Vec<u8>,
    alloc::vec::Vec<u8>,
    u128,
);

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_get_transfers() -> alloc::vec::Vec<MockTransfer> {
    mock::mock_get_transfers()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mock_reset_transfers() {
    mock::mock_reset_transfers();
}
