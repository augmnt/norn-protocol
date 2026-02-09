//! The `norn_entry!` declarative macro.
//!
//! Generates the Wasm entry points (`init`, `execute`, `query`) and wires up
//! the global allocator, state persistence, message deserialization, and output
//! buffer management so contract developers never touch FFI directly.

/// Generate all Wasm boilerplate for a [`Contract`](crate::Contract) implementation.
///
/// Expands to:
/// - `#[global_allocator]` with `dlmalloc` (wasm32 only)
/// - `#[no_mangle] pub extern "C" fn init()` — initializes state
/// - `#[no_mangle] pub extern "C" fn execute(ptr, len)` — state-changing call
/// - `#[no_mangle] pub extern "C" fn query(ptr, len)` — read-only call
///
/// # Example
///
/// ```ignore
/// use norn_sdk::prelude::*;
///
/// #[derive(BorshSerialize, BorshDeserialize)]
/// pub struct MyContract { value: u64 }
///
/// #[derive(BorshSerialize, BorshDeserialize)]
/// pub enum Exec { DoSomething }
///
/// #[derive(BorshSerialize, BorshDeserialize)]
/// pub enum Query { GetValue }
///
/// impl Contract for MyContract {
///     type Exec = Exec;
///     type Query = Query;
///     fn init(_ctx: &Context) -> Self { MyContract { value: 0 } }
///     fn execute(&mut self, _ctx: &Context, _msg: Exec) -> ContractResult { ok_empty() }
///     fn query(&self, _ctx: &Context, _msg: Query) -> ContractResult { ok(self.value) }
/// }
///
/// norn_entry!(MyContract);
/// ```
#[macro_export]
macro_rules! norn_entry {
    ($contract:ty) => {
        // Global allocator for wasm32 targets.
        #[cfg(target_arch = "wasm32")]
        #[global_allocator]
        static __NORN_ALLOC: $crate::dlmalloc::GlobalDlmalloc = $crate::dlmalloc::GlobalDlmalloc;

        // Panic handler for wasm32 targets (no_std requires one).
        #[cfg(target_arch = "wasm32")]
        #[panic_handler]
        fn __norn_panic(_info: &::core::panic::PanicInfo) -> ! {
            ::core::arch::wasm32::unreachable()
        }

        const __NORN_STATE_KEY: &[u8] = b"__norn_contract_state";

        #[no_mangle]
        pub extern "C" fn init() {
            let ctx = $crate::contract::Context::new();
            let state = <$contract as $crate::contract::Contract>::init(&ctx);
            if let Ok(bytes) = ::borsh::to_vec(&state) {
                $crate::host::state_set(__NORN_STATE_KEY, &bytes);
            }
        }

        #[no_mangle]
        pub extern "C" fn execute(ptr: i32, len: i32) -> i32 {
            // Load state
            let state_bytes = match $crate::host::state_get(__NORN_STATE_KEY) {
                Some(b) => b,
                None => {
                    $crate::output::set_output(b"contract state not initialized");
                    return 1;
                }
            };
            let mut state: $contract = match ::borsh::BorshDeserialize::try_from_slice(&state_bytes)
            {
                Ok(s) => s,
                Err(_) => {
                    $crate::output::set_output(b"failed to deserialize contract state");
                    return 1;
                }
            };

            // Deserialize input message
            let input = $crate::output::read_input(ptr, len);
            let msg: <$contract as $crate::contract::Contract>::Exec =
                match ::borsh::BorshDeserialize::try_from_slice(&input) {
                    Ok(m) => m,
                    Err(_) => {
                        $crate::output::set_output(b"failed to deserialize execute message");
                        return 1;
                    }
                };

            // Execute
            let ctx = $crate::contract::Context::new();
            match <$contract as $crate::contract::Contract>::execute(&mut state, &ctx, msg) {
                Ok(response) => {
                    // Persist updated state
                    if let Ok(bytes) = ::borsh::to_vec(&state) {
                        $crate::host::state_set(__NORN_STATE_KEY, &bytes);
                    }
                    response.__emit_to_host();
                    $crate::output::set_output(response.__data());
                    0
                }
                Err(err) => {
                    let err_bytes = $crate::contract::error_to_bytes(&err);
                    $crate::output::set_output(&err_bytes);
                    1
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn query(ptr: i32, len: i32) -> i32 {
            // Load state (read-only)
            let state_bytes = match $crate::host::state_get(__NORN_STATE_KEY) {
                Some(b) => b,
                None => {
                    $crate::output::set_output(b"contract state not initialized");
                    return 1;
                }
            };
            let state: $contract = match ::borsh::BorshDeserialize::try_from_slice(&state_bytes) {
                Ok(s) => s,
                Err(_) => {
                    $crate::output::set_output(b"failed to deserialize contract state");
                    return 1;
                }
            };

            // Deserialize query message
            let input = $crate::output::read_input(ptr, len);
            let msg: <$contract as $crate::contract::Contract>::Query =
                match ::borsh::BorshDeserialize::try_from_slice(&input) {
                    Ok(m) => m,
                    Err(_) => {
                        $crate::output::set_output(b"failed to deserialize query message");
                        return 1;
                    }
                };

            // Query (no state save)
            let ctx = $crate::contract::Context::new();
            match <$contract as $crate::contract::Contract>::query(&state, &ctx, msg) {
                Ok(response) => {
                    response.__emit_to_host();
                    $crate::output::set_output(response.__data());
                    0
                }
                Err(err) => {
                    let err_bytes = $crate::contract::error_to_bytes(&err);
                    $crate::output::set_output(&err_bytes);
                    1
                }
            }
        }
    };
}
