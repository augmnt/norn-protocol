use wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store};

use crate::error::LoomError;
use crate::host::LoomHostState;

/// The Wasm runtime engine for loom contracts.
///
/// Wraps a wasmtime `Engine` configured with fuel metering for deterministic
/// gas accounting.
pub struct LoomRuntime {
    engine: Engine,
}

/// A live instance of a loom contract.
///
/// Owns the wasmtime `Store` (which holds the `LoomHostState`) and the
/// instantiated Wasm module.
pub struct LoomInstance {
    store: Store<LoomHostState>,
    instance: Instance,
}

impl LoomRuntime {
    /// Create a new runtime with fuel metering enabled.
    pub fn new() -> Result<Self, LoomError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).map_err(|e| LoomError::RuntimeError {
            reason: format!("failed to create wasmtime engine: {e}"),
        })?;
        Ok(Self { engine })
    }

    /// Compile and instantiate a Wasm module with the given host state.
    ///
    /// Host functions are registered in the `norn` namespace and delegate to
    /// methods on `LoomHostState`. The store is seeded with fuel equal to the
    /// gas limit configured in the host state.
    pub fn instantiate(
        &self,
        bytecode: &[u8],
        host_state: LoomHostState,
    ) -> Result<LoomInstance, LoomError> {
        let module =
            Module::new(&self.engine, bytecode).map_err(|e| LoomError::InvalidBytecode {
                reason: format!("failed to compile wasm module: {e}"),
            })?;

        let gas_limit = host_state.gas_meter.limit;
        let mut store = Store::new(&self.engine, host_state);
        store.limiter(|state| &mut state.store_limits);
        store
            .set_fuel(gas_limit)
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to set fuel: {e}"),
            })?;

        let mut linker: Linker<LoomHostState> = Linker::new(&self.engine);

        // ── Host function: norn_log ──────────────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_log",
                |mut caller: wasmtime::Caller<'_, LoomHostState>, msg_ptr: i32, msg_len: i32| {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or(wasmtime::Error::msg("missing memory export"))?;
                    let start = msg_ptr as usize;
                    let end = start + msg_len as usize;
                    // Copy bytes out of wasm memory before mutably borrowing caller.
                    let msg_bytes = {
                        let data = memory.data(&caller);
                        if end > data.len() {
                            return Err(wasmtime::Error::msg("out of bounds memory access"));
                        }
                        data[start..end].to_vec()
                    };
                    let msg = std::str::from_utf8(&msg_bytes).unwrap_or("<invalid utf8>");
                    caller
                        .data_mut()
                        .log(msg)
                        .map_err(|e| wasmtime::Error::msg(format!("host log error: {e}")))?;
                    Ok(())
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_log: {e}"),
            })?;

        // ── Host function: norn_state_get ────────────────────────────────
        // Signature: (key_ptr, key_len, out_ptr, out_max_len) -> i32
        // If out_ptr == 0: query mode — returns value length (or -1 if not found)
        // If out_ptr != 0: write mode — writes value to out_ptr, returns length
        // Returns -1 for not found, -2 for buffer too small
        linker
            .func_wrap(
                "norn",
                "norn_state_get",
                |mut caller: wasmtime::Caller<'_, LoomHostState>,
                 key_ptr: i32,
                 key_len: i32,
                 out_ptr: i32,
                 out_max_len: i32|
                 -> Result<i32, wasmtime::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or(wasmtime::Error::msg("missing memory export"))?;
                    let data = memory.data(&caller);
                    let start = key_ptr as usize;
                    let end = start + key_len as usize;
                    if end > data.len() {
                        return Err(wasmtime::Error::msg("out of bounds memory access"));
                    }
                    let key = data[start..end].to_vec();
                    let value = caller
                        .data_mut()
                        .state_get(&key)
                        .map_err(|e| wasmtime::Error::msg(format!("host state_get error: {e}")))?;
                    match value {
                        Some(v) => {
                            let val_len = v.len() as i32;
                            if out_ptr == 0 {
                                // Query mode: just return length
                                Ok(val_len)
                            } else if (out_max_len as usize) < v.len() {
                                // Buffer too small
                                Ok(-2)
                            } else {
                                // Write value to WASM memory
                                let out_start = out_ptr as usize;
                                let out_end = out_start + v.len();
                                let mem_data = memory.data_mut(&mut caller);
                                if out_end > mem_data.len() {
                                    return Err(wasmtime::Error::msg(
                                        "out of bounds memory access",
                                    ));
                                }
                                mem_data[out_start..out_end].copy_from_slice(&v);
                                Ok(val_len)
                            }
                        }
                        None => Ok(-1),
                    }
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_state_get: {e}"),
            })?;

        // ── Host function: norn_state_set ────────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_state_set",
                |mut caller: wasmtime::Caller<'_, LoomHostState>,
                 key_ptr: i32,
                 key_len: i32,
                 val_ptr: i32,
                 val_len: i32|
                 -> Result<(), wasmtime::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or(wasmtime::Error::msg("missing memory export"))?;
                    let data = memory.data(&caller);
                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    let val_start = val_ptr as usize;
                    let val_end = val_start + val_len as usize;
                    if key_end > data.len() || val_end > data.len() {
                        return Err(wasmtime::Error::msg("out of bounds memory access"));
                    }
                    let key = data[key_start..key_end].to_vec();
                    let val = data[val_start..val_end].to_vec();
                    caller
                        .data_mut()
                        .state_set(&key, &val)
                        .map_err(|e| wasmtime::Error::msg(format!("host state_set error: {e}")))?;
                    Ok(())
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_state_set: {e}"),
            })?;

        // ── Host function: norn_transfer ─────────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_transfer",
                |mut caller: wasmtime::Caller<'_, LoomHostState>,
                 from_ptr: i32,
                 to_ptr: i32,
                 token_ptr: i32,
                 amount: i64|
                 -> Result<(), wasmtime::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or(wasmtime::Error::msg("missing memory export"))?;
                    let data = memory.data(&caller);
                    let from_start = from_ptr as usize;
                    let to_start = to_ptr as usize;
                    let token_start = token_ptr as usize;

                    if from_start + 20 > data.len()
                        || to_start + 20 > data.len()
                        || token_start + 32 > data.len()
                    {
                        return Err(wasmtime::Error::msg("out of bounds memory access"));
                    }

                    let mut from = [0u8; 20];
                    from.copy_from_slice(&data[from_start..from_start + 20]);
                    let mut to = [0u8; 20];
                    to.copy_from_slice(&data[to_start..to_start + 20]);
                    let mut token_id = [0u8; 32];
                    token_id.copy_from_slice(&data[token_start..token_start + 32]);

                    // Validate amount is positive (i64 could be negative or zero).
                    if amount <= 0 {
                        return Err(wasmtime::Error::msg(
                            "norn_transfer: amount must be positive",
                        ));
                    }

                    // Verify the `from` address matches the contract caller.
                    // Contracts can only transfer from their own address (the sender).
                    let sender = caller.data().sender;
                    if from != sender {
                        return Err(wasmtime::Error::msg(
                            "norn_transfer: from address must match the contract caller",
                        ));
                    }

                    caller
                        .data_mut()
                        .transfer(from, to, token_id, amount as u128)
                        .map_err(|e| wasmtime::Error::msg(format!("host transfer error: {e}")))?;
                    Ok(())
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_transfer: {e}"),
            })?;

        // ── Host function: norn_sender ───────────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_sender",
                |mut caller: wasmtime::Caller<'_, LoomHostState>,
                 out_ptr: i32|
                 -> Result<(), wasmtime::Error> {
                    let sender = caller.data().sender;
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or(wasmtime::Error::msg("missing memory export"))?;
                    let start = out_ptr as usize;
                    if start + 20 > memory.data(&caller).len() {
                        return Err(wasmtime::Error::msg("out of bounds memory access"));
                    }
                    memory.data_mut(&mut caller)[start..start + 20].copy_from_slice(&sender);
                    Ok(())
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_sender: {e}"),
            })?;

        // ── Host function: norn_block_height ─────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_block_height",
                |caller: wasmtime::Caller<'_, LoomHostState>| -> i64 {
                    caller.data().block_height as i64
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_block_height: {e}"),
            })?;

        // ── Host function: norn_timestamp ────────────────────────────────
        linker
            .func_wrap(
                "norn",
                "norn_timestamp",
                |caller: wasmtime::Caller<'_, LoomHostState>| -> i64 {
                    caller.data().timestamp as i64
                },
            )
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("failed to register norn_timestamp: {e}"),
            })?;

        let instance =
            linker
                .instantiate(&mut store, &module)
                .map_err(|e| LoomError::RuntimeError {
                    reason: format!("failed to instantiate module: {e}"),
                })?;

        Ok(LoomInstance { store, instance })
    }
}

impl LoomInstance {
    /// Try to read the output buffer from an SDK-based contract.
    ///
    /// Calls `__norn_output_ptr()` and `__norn_output_len()` exports.
    /// Returns an empty vec if these exports are not present.
    fn read_output_buffer(&mut self) -> Vec<u8> {
        let output_ptr = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "__norn_output_ptr")
            .ok()
            .and_then(|f| f.call(&mut self.store, ()).ok());
        let output_len = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "__norn_output_len")
            .ok()
            .and_then(|f| f.call(&mut self.store, ()).ok());

        match (output_ptr, output_len) {
            (Some(ptr), Some(len)) if len > 0 => {
                let ptr = ptr as usize;
                let len = len as usize;
                if let Some(memory) = self.instance.get_memory(&mut self.store, "memory") {
                    let data = memory.data(&self.store);
                    if ptr + len <= data.len() {
                        return data[ptr..ptr + len].to_vec();
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// Write input into Wasm memory using `__norn_alloc` if available,
    /// falling back to offset 1024 for legacy WAT modules.
    fn write_input(&mut self, input: &[u8]) -> (i32, i32) {
        if input.is_empty() {
            return (0, 0);
        }

        let memory = match self.instance.get_memory(&mut self.store, "memory") {
            Some(m) => m,
            None => return (0, 0),
        };

        // Try __norn_alloc first (SDK-based contracts).
        if let Ok(alloc_fn) = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "__norn_alloc")
        {
            if let Ok(ptr) = alloc_fn.call(&mut self.store, input.len() as i32) {
                if ptr > 0 {
                    let offset = ptr as usize;
                    let mem_size = memory.data_size(&self.store);
                    if offset + input.len() <= mem_size {
                        memory.data_mut(&mut self.store)[offset..offset + input.len()]
                            .copy_from_slice(input);
                        return (ptr, input.len() as i32);
                    }
                }
            }
        }

        // Fallback: write at offset 1024 for legacy WAT modules.
        let mem_size = memory.data_size(&self.store);
        let offset = 1024.min(mem_size.saturating_sub(input.len()));
        if offset + input.len() <= mem_size {
            memory.data_mut(&mut self.store)[offset..offset + input.len()].copy_from_slice(input);
            (offset as i32, input.len() as i32)
        } else {
            (0, 0)
        }
    }

    /// Call the exported `init` function (no arguments, no return).
    pub fn call_init(&mut self) -> Result<(), LoomError> {
        let init = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, "init")
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("init function not found or wrong signature: {e}"),
            })?;
        init.call(&mut self.store, ())
            .map_err(|e| LoomError::RuntimeError {
                reason: format!("init execution failed: {e}"),
            })?;
        Ok(())
    }

    /// Call the exported `execute` function.
    ///
    /// The function receives `(input_ptr, input_len)` and returns an `i32`
    /// result code. Output is read from the SDK output buffer if available,
    /// falling back to the i32 return value as little-endian bytes.
    pub fn call_execute(&mut self, input: &[u8]) -> Result<Vec<u8>, LoomError> {
        // Try the simple (i32, i32) -> i32 signature first.
        if let Ok(execute) = self
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "execute")
        {
            let (ptr, len) = self.write_input(input);

            let result =
                execute
                    .call(&mut self.store, (ptr, len))
                    .map_err(|e| LoomError::RuntimeError {
                        reason: format!("execute failed: {e}"),
                    })?;

            // Try SDK output buffer first; fall back to i32-as-bytes.
            let output = self.read_output_buffer();
            if !output.is_empty() {
                return Ok(output);
            }
            return Ok(result.to_le_bytes().to_vec());
        }

        // Fallback: try () -> i32 signature (very simple test modules).
        if let Ok(execute) = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "execute")
        {
            let result =
                execute
                    .call(&mut self.store, ())
                    .map_err(|e| LoomError::RuntimeError {
                        reason: format!("execute failed: {e}"),
                    })?;

            let output = self.read_output_buffer();
            if !output.is_empty() {
                return Ok(output);
            }
            return Ok(result.to_le_bytes().to_vec());
        }

        Err(LoomError::RuntimeError {
            reason: "execute function not found or has unsupported signature".to_string(),
        })
    }

    /// Call the exported `query` function (read-only).
    pub fn call_query(&mut self, input: &[u8]) -> Result<Vec<u8>, LoomError> {
        if let Ok(query) = self
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "query")
        {
            let (ptr, len) = self.write_input(input);

            let result =
                query
                    .call(&mut self.store, (ptr, len))
                    .map_err(|e| LoomError::RuntimeError {
                        reason: format!("query failed: {e}"),
                    })?;

            // Try SDK output buffer first; fall back to i32-as-bytes.
            let output = self.read_output_buffer();
            if !output.is_empty() {
                return Ok(output);
            }
            return Ok(result.to_le_bytes().to_vec());
        }

        Err(LoomError::RuntimeError {
            reason: "query function not found or has unsupported signature".to_string(),
        })
    }

    /// Return the amount of gas (fuel) consumed so far.
    pub fn gas_used(&self) -> u64 {
        let remaining = self.store.get_fuel().unwrap_or(0);
        let limit = self.store.data().gas_meter.limit;
        limit.saturating_sub(remaining)
    }

    /// Consume this instance and return the host state (with all accumulated
    /// state changes, transfers, and logs).
    pub fn into_host_state(self) -> LoomHostState {
        self.store.into_data()
    }

    /// Get a reference to the underlying memory export (if any).
    pub fn memory(&mut self) -> Option<Memory> {
        self.instance.get_memory(&mut self.store, "memory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gas::DEFAULT_GAS_LIMIT;

    /// Minimal WAT module that exports an `execute` function returning 42.
    const SIMPLE_WAT: &str = r#"
        (module
            (func (export "execute") (param i32 i32) (result i32)
                i32.const 42
            )
        )
    "#;

    /// WAT module with an init function.
    const INIT_WAT: &str = r#"
        (module
            (func (export "init"))
            (func (export "execute") (param i32 i32) (result i32)
                i32.const 99
            )
        )
    "#;

    fn compile_wat(wat: &str) -> Vec<u8> {
        wat::parse_str(wat).expect("failed to compile WAT")
    }

    #[test]
    fn test_instantiate_and_execute() {
        let runtime = LoomRuntime::new().unwrap();
        let bytecode = compile_wat(SIMPLE_WAT);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();

        let result = instance.call_execute(&[]).unwrap();
        // 42 as little-endian i32 bytes.
        assert_eq!(result, 42i32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_gas_consumption() {
        let runtime = LoomRuntime::new().unwrap();
        let bytecode = compile_wat(SIMPLE_WAT);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();

        instance.call_execute(&[]).unwrap();
        // Some fuel should have been consumed.
        assert!(instance.gas_used() > 0);
    }

    #[test]
    fn test_init_then_execute() {
        let runtime = LoomRuntime::new().unwrap();
        let bytecode = compile_wat(INIT_WAT);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();

        instance.call_init().unwrap();
        let result = instance.call_execute(&[]).unwrap();
        assert_eq!(result, 99i32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_invalid_bytecode() {
        let runtime = LoomRuntime::new().unwrap();
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let result = runtime.instantiate(&[0xFF, 0xFF, 0xFF], host_state);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_host_state() {
        let runtime = LoomRuntime::new().unwrap();
        let bytecode = compile_wat(SIMPLE_WAT);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();

        instance.call_execute(&[]).unwrap();
        let recovered = instance.into_host_state();
        assert_eq!(recovered.sender, [1u8; 20]);
        assert_eq!(recovered.block_height, 100);
    }

    #[test]
    fn test_gas_exhaustion() {
        let runtime = LoomRuntime::new().unwrap();
        // A module with an infinite loop.
        let loop_wat = r#"
            (module
                (func (export "execute") (param i32 i32) (result i32)
                    (loop $inf
                        (br $inf)
                    )
                    i32.const 0
                )
            )
        "#;
        let bytecode = compile_wat(loop_wat);
        // Give very little fuel so it runs out quickly.
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, 100);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();

        let result = instance.call_execute(&[]);
        // Should fail due to fuel exhaustion.
        assert!(result.is_err());
    }

    /// WAT module that calls norn_state_get with out_ptr to read value back.
    #[test]
    fn test_state_get_writes_value_to_memory() {
        let runtime = LoomRuntime::new().unwrap();

        // Module that: sets state "key" = "val" via norn_state_set,
        // then calls norn_state_get to read it back, returns the length.
        let wat = r#"
            (module
                (import "norn" "norn_state_set" (func $set (param i32 i32 i32 i32)))
                (import "norn" "norn_state_get" (func $get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                ;; At offset 0: key "key" (3 bytes)
                (data (i32.const 0) "key")
                ;; At offset 3: value "val" (3 bytes)
                (data (i32.const 3) "val")
                ;; At offset 100: output buffer (64 bytes available)
                (func (export "execute") (param i32 i32) (result i32)
                    ;; Set state: key_ptr=0, key_len=3, val_ptr=3, val_len=3
                    (call $set (i32.const 0) (i32.const 3) (i32.const 3) (i32.const 3))
                    ;; Get state: key_ptr=0, key_len=3, out_ptr=100, out_max_len=64
                    (call $get (i32.const 0) (i32.const 3) (i32.const 100) (i32.const 64))
                    ;; Returns value length (3)
                )
            )
        "#;
        let bytecode = compile_wat(wat);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();
        let result = instance.call_execute(&[]).unwrap();
        // Should return 3 (length of "val")
        assert_eq!(result, 3i32.to_le_bytes().to_vec());

        // Verify the value was written to WASM memory at offset 100
        let memory = instance.memory().unwrap();
        let data = memory.data(&instance.store);
        assert_eq!(&data[100..103], b"val");
    }

    #[test]
    fn test_state_get_returns_minus1_for_missing_key() {
        let runtime = LoomRuntime::new().unwrap();
        let wat = r#"
            (module
                (import "norn" "norn_state_get" (func $get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "missing")
                (func (export "execute") (param i32 i32) (result i32)
                    ;; Query for non-existent key, query mode (out_ptr=0)
                    (call $get (i32.const 0) (i32.const 7) (i32.const 0) (i32.const 0))
                )
            )
        "#;
        let bytecode = compile_wat(wat);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();
        let result = instance.call_execute(&[]).unwrap();
        // -1 for not found
        assert_eq!(result, (-1i32).to_le_bytes().to_vec());
    }

    #[test]
    fn test_transfer_with_negative_amount_fails() {
        let runtime = LoomRuntime::new().unwrap();
        // Module that calls norn_transfer with a negative amount (-1 as i64).
        let wat = r#"
            (module
                (import "norn" "norn_transfer" (func $transfer (param i32 i32 i32 i64)))
                (memory (export "memory") 1)
                ;; from address at offset 0 (20 bytes of 0x01)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")
                ;; to address at offset 20 (20 bytes of 0x02)
                (data (i32.const 20) "\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02")
                ;; token_id at offset 40 (32 bytes of zeros)
                (func (export "execute") (param i32 i32) (result i32)
                    ;; Transfer with amount = -1 (invalid)
                    (call $transfer (i32.const 0) (i32.const 20) (i32.const 40) (i64.const -1))
                    i32.const 0
                )
            )
        "#;
        let bytecode = compile_wat(wat);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();
        // Should fail because amount is negative
        assert!(instance.call_execute(&[]).is_err());
    }

    #[test]
    fn test_transfer_with_zero_amount_fails() {
        let runtime = LoomRuntime::new().unwrap();
        let wat = r#"
            (module
                (import "norn" "norn_transfer" (func $transfer (param i32 i32 i32 i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")
                (data (i32.const 20) "\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02")
                (func (export "execute") (param i32 i32) (result i32)
                    (call $transfer (i32.const 0) (i32.const 20) (i32.const 40) (i64.const 0))
                    i32.const 0
                )
            )
        "#;
        let bytecode = compile_wat(wat);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();
        assert!(instance.call_execute(&[]).is_err());
    }

    #[test]
    fn test_memory_limit_enforced() {
        let runtime = LoomRuntime::new().unwrap();
        // Module that tries to grow memory beyond the 16MB limit.
        // Each page = 64KB, so 16MB = 256 pages. Request 512 pages.
        let wat = r#"
            (module
                (memory (export "memory") 1)
                (func (export "execute") (param i32 i32) (result i32)
                    ;; Try to grow memory by 512 pages (32MB) - should be rejected
                    (memory.grow (i32.const 512))
                    ;; memory.grow returns -1 on failure
                )
            )
        "#;
        let bytecode = compile_wat(wat);
        let host_state = LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT);
        let mut instance = runtime.instantiate(&bytecode, host_state).unwrap();
        let result = instance.call_execute(&[]).unwrap();
        // memory.grow returns -1 (as i32) when growth fails
        assert_eq!(result, (-1i32).to_le_bytes().to_vec());
    }
}
