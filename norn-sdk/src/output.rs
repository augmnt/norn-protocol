//! Output buffer for returning data from contract calls.
//!
//! The runtime reads the output by calling the exported `__norn_output_ptr()`
//! and `__norn_output_len()` functions after `execute` or `query` returns.
//!
//! The `__norn_alloc` export allows the runtime to allocate Wasm memory for
//! writing input data before calling `execute`/`query`.

use alloc::vec::Vec;
use core::cell::UnsafeCell;

/// Maximum output buffer size (4 KiB).
const OUTPUT_BUF_SIZE: usize = 4096;

struct OutputBuffer {
    buf: [u8; OUTPUT_BUF_SIZE],
    len: usize,
}

/// Wrapper to make `UnsafeCell<OutputBuffer>` usable in a static.
/// Safety: Wasm is single-threaded, so concurrent access is not possible.
struct SyncOutputBuffer(UnsafeCell<OutputBuffer>);
unsafe impl Sync for SyncOutputBuffer {}

static OUTPUT: SyncOutputBuffer = SyncOutputBuffer(UnsafeCell::new(OutputBuffer {
    buf: [0u8; OUTPUT_BUF_SIZE],
    len: 0,
}));

/// Set the output data that the runtime will read after the call.
///
/// If `data` exceeds 4096 bytes, it is truncated.
pub fn set_output(data: &[u8]) {
    unsafe {
        let out = &mut *OUTPUT.0.get();
        let copy_len = data.len().min(OUTPUT_BUF_SIZE);
        out.buf[..copy_len].copy_from_slice(&data[..copy_len]);
        out.len = copy_len;
    }
}

/// Exported function: returns a pointer to the output buffer.
#[no_mangle]
pub extern "C" fn __norn_output_ptr() -> i32 {
    unsafe { (*OUTPUT.0.get()).buf.as_ptr() as i32 }
}

/// Exported function: returns the length of valid output data.
#[no_mangle]
pub extern "C" fn __norn_output_len() -> i32 {
    unsafe { (*OUTPUT.0.get()).len as i32 }
}

/// Exported function: allocate `len` bytes of Wasm memory for the runtime to
/// write input data into. Returns a pointer to the allocated region.
#[no_mangle]
pub extern "C" fn __norn_alloc(len: i32) -> i32 {
    let len = len as usize;
    let layout = core::alloc::Layout::from_size_align(len, 1).unwrap();
    unsafe {
        let ptr = alloc::alloc::alloc(layout);
        if ptr.is_null() {
            return 0;
        }
        ptr as i32
    }
}

/// Read input bytes written by the runtime at the given pointer and length.
///
/// Call this at the start of `execute(ptr, len)` or `query(ptr, len)` to
/// get the input data as a `Vec<u8>`.
pub fn read_input(ptr: i32, len: i32) -> Vec<u8> {
    if len <= 0 || ptr <= 0 {
        return Vec::new();
    }
    let len = len as usize;
    let ptr = ptr as *const u8;
    let mut buf = Vec::with_capacity(len);
    unsafe {
        for i in 0..len {
            buf.push(*ptr.add(i));
        }
    }
    buf
}
