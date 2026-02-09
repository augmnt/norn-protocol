//! Safe Rust wrappers around the Norn host functions.
//!
//! These functions are linked at runtime by the norn-loom Wasm engine. They are
//! only callable when running inside the Norn runtime (target wasm32-unknown-unknown).

use alloc::vec::Vec;

// ── Raw extern declarations ────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn norn_log(msg_ptr: i32, msg_len: i32);
    fn norn_state_get(key_ptr: i32, key_len: i32, out_ptr: i32, out_max_len: i32) -> i32;
    fn norn_state_set(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32);
    fn norn_transfer(from_ptr: i32, to_ptr: i32, token_ptr: i32, amount: i64);
    fn norn_sender(out_ptr: i32);
    fn norn_block_height() -> i64;
    fn norn_timestamp() -> i64;
}

// ── Safe wrappers ──────────────────────────────────────────────────────────

/// Emit a log message visible in execution results.
#[cfg(target_arch = "wasm32")]
pub fn log(msg: &str) {
    unsafe {
        norn_log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(_msg: &str) {}

/// Read a value from contract state.
///
/// Uses a two-phase protocol: first queries the length (out_ptr=0), then
/// allocates a buffer and reads the value.
#[cfg(target_arch = "wasm32")]
pub fn state_get(key: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        // Phase 1: query length.
        let len = norn_state_get(key.as_ptr() as i32, key.len() as i32, 0, 0);
        if len < 0 {
            return None; // -1 = not found
        }
        let len = len as usize;
        if len == 0 {
            return Some(vec![]);
        }
        // Phase 2: read value into buffer.
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

#[cfg(not(target_arch = "wasm32"))]
pub fn state_get(_key: &[u8]) -> Option<Vec<u8>> {
    None
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

#[cfg(not(target_arch = "wasm32"))]
pub fn state_set(_key: &[u8], _value: &[u8]) {}

/// Transfer tokens. The `from` address must match the contract caller (sender).
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

#[cfg(not(target_arch = "wasm32"))]
pub fn transfer(_from: &[u8; 20], _to: &[u8; 20], _token_id: &[u8; 32], _amount: u128) {}

/// Get the address of the transaction sender.
#[cfg(target_arch = "wasm32")]
pub fn sender() -> [u8; 20] {
    let mut addr = [0u8; 20];
    unsafe {
        norn_sender(addr.as_mut_ptr() as i32);
    }
    addr
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sender() -> [u8; 20] {
    [0u8; 20]
}

/// Get the current block height.
#[cfg(target_arch = "wasm32")]
pub fn block_height() -> u64 {
    unsafe { norn_block_height() as u64 }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn block_height() -> u64 {
    0
}

/// Get the current block timestamp (unix seconds).
#[cfg(target_arch = "wasm32")]
pub fn timestamp() -> u64 {
    unsafe { norn_timestamp() as u64 }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp() -> u64 {
    0
}
