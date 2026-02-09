//! Counter contract example for the Norn Protocol.
//!
//! Actions (first byte of input):
//! - 0x01: Increment counter by 1
//! - 0x02: Decrement counter by 1 (saturating)
//! - 0x03: Reset counter to 0
//!
//! Query: returns the current counter value as u64 LE bytes.

#![no_std]

extern crate alloc;

// Global allocator for wasm32 target.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use norn_sdk::{encoding, host, output};

const KEY_COUNTER: &[u8] = b"counter";

fn get_counter() -> u64 {
    host::state_get(KEY_COUNTER)
        .and_then(|bytes| encoding::decode_u64(&bytes))
        .unwrap_or(0)
}

fn set_counter(value: u64) {
    host::state_set(KEY_COUNTER, &encoding::encode_u64(value));
}

#[no_mangle]
pub extern "C" fn init() {
    set_counter(0);
    host::log("counter initialized");
}

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i32 {
    let input = output::read_input(ptr, len);

    if input.is_empty() {
        // Default: increment by 1.
        let value = get_counter() + 1;
        set_counter(value);
        output::set_output(&encoding::encode_u64(value));
        return 0;
    }

    let action = input[0];
    let value = match action {
        0x01 => {
            // Increment
            let v = get_counter() + 1;
            set_counter(v);
            host::log("incremented");
            v
        }
        0x02 => {
            // Decrement (saturating)
            let v = get_counter().saturating_sub(1);
            set_counter(v);
            host::log("decremented");
            v
        }
        0x03 => {
            // Reset
            set_counter(0);
            host::log("reset");
            0
        }
        _ => {
            host::log("unknown action");
            return 1; // error
        }
    };

    output::set_output(&encoding::encode_u64(value));
    0
}

#[no_mangle]
pub extern "C" fn query(ptr: i32, len: i32) -> i32 {
    let _input = output::read_input(ptr, len);
    let value = get_counter();
    output::set_output(&encoding::encode_u64(value));
    0
}
