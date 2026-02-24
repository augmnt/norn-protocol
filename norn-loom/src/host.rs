use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use norn_types::primitives::{Address, Amount, LoomId, TokenId};
use wasmtime::StoreLimits;

use crate::call_stack::CallStack;
use crate::error::LoomError;
use crate::gas::*;

/// Shared cross-call loom state map: LoomId -> key-value state.
pub type SharedLoomStates = Arc<Mutex<HashMap<LoomId, HashMap<Vec<u8>, Vec<u8>>>>>;
/// Shared cross-call bytecode map: LoomId -> wasm bytecode.
pub type SharedLoomBytecodes = Arc<Mutex<HashMap<LoomId, Vec<u8>>>>;

/// Maximum WASM memory: 16 MB.
pub const MAX_WASM_MEMORY_BYTES: usize = 16 * 1024 * 1024;

/// Maximum pending transfers per execution (including cross-call merges).
pub const MAX_PENDING_TRANSFERS: usize = 256;
/// Maximum log messages per execution (including cross-call merges).
pub const MAX_LOGS: usize = 1_000;
/// Maximum events per execution (including cross-call merges).
pub const MAX_EVENTS: usize = 1_000;

/// A pending token transfer produced during loom execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTransfer {
    /// Sender address.
    pub from: Address,
    /// Recipient address.
    pub to: Address,
    /// Token being transferred.
    pub token_id: TokenId,
    /// Amount to transfer.
    pub amount: Amount,
}

/// A structured event emitted by a loom contract via the `norn_emit_event` host function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEvent {
    /// Event type name (e.g., "Transfer", "Approval").
    pub ty: String,
    /// Key-value attributes.
    pub attributes: Vec<(String, String)>,
}

/// Host-side state accessible to Wasm loom contracts via host functions.
///
/// This struct is owned by the wasmtime `Store` and provides the backing
/// storage, gas metering, and context that host functions operate on.
pub struct LoomHostState {
    /// Gas meter for tracking execution cost.
    pub gas_meter: GasMeter,
    /// Key-value state of the loom contract.
    pub state: HashMap<Vec<u8>, Vec<u8>>,
    /// Transfers emitted during execution (applied on success).
    pub pending_transfers: Vec<PendingTransfer>,
    /// Log messages emitted during execution.
    pub logs: Vec<String>,
    /// Structured events emitted during execution.
    pub events: Vec<HostEvent>,
    /// The address that initiated the current execution.
    pub sender: Address,
    /// Current block height.
    pub block_height: u64,
    /// Current block timestamp (unix seconds).
    pub timestamp: u64,
    /// Store limits for memory capping.
    pub store_limits: StoreLimits,

    // ── Cross-contract call fields (set only during cross-call execution) ──
    /// Shared call stack for tracking nested cross-contract calls.
    pub call_stack: Option<Arc<Mutex<CallStack>>>,
    /// Shared mutable access to all loom states (for cross-call reads/writes).
    pub loom_states: Option<SharedLoomStates>,
    /// Shared access to all loom bytecodes (for instantiating target contracts).
    pub loom_bytecodes: Option<SharedLoomBytecodes>,
    /// The loom ID of the currently executing contract (for cross-call context).
    pub current_loom_id: Option<LoomId>,
}

impl LoomHostState {
    /// Create a new host state with the given execution context.
    pub fn new(sender: Address, block_height: u64, timestamp: u64, gas_limit: u64) -> Self {
        use wasmtime::StoreLimitsBuilder;
        Self {
            gas_meter: GasMeter::new(gas_limit),
            state: HashMap::new(),
            pending_transfers: Vec::new(),
            logs: Vec::new(),
            events: Vec::new(),
            sender,
            block_height,
            timestamp,
            store_limits: StoreLimitsBuilder::new()
                .memory_size(MAX_WASM_MEMORY_BYTES)
                .build(),
            call_stack: None,
            loom_states: None,
            loom_bytecodes: None,
            current_loom_id: None,
        }
    }

    /// Read a value from the loom state.
    /// Charges GAS_STATE_READ plus GAS_BYTE_READ per byte of the value.
    pub fn state_get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, LoomError> {
        self.gas_meter.charge(GAS_STATE_READ)?;
        let value = self.state.get(key).cloned();
        if let Some(ref v) = value {
            self.gas_meter
                .charge(GAS_BYTE_READ.saturating_mul(v.len() as u64))?;
        }
        Ok(value)
    }

    /// Write a value to the loom state.
    /// Charges GAS_STATE_WRITE plus GAS_BYTE_WRITE per byte of the value.
    /// Bounded to prevent unbounded state growth.
    pub fn state_set(&mut self, key: &[u8], value: &[u8]) -> Result<(), LoomError> {
        const MAX_KEY_SIZE: usize = 1024;
        const MAX_VALUE_SIZE: usize = 65_536;
        const MAX_STATE_ENTRIES: usize = 10_000;

        if key.len() > MAX_KEY_SIZE {
            return Err(LoomError::RuntimeError {
                reason: "state key too large".to_string(),
            });
        }
        if value.len() > MAX_VALUE_SIZE {
            return Err(LoomError::RuntimeError {
                reason: "state value too large".to_string(),
            });
        }
        if !self.state.contains_key(key) && self.state.len() >= MAX_STATE_ENTRIES {
            return Err(LoomError::RuntimeError {
                reason: "state entry limit reached".to_string(),
            });
        }
        self.gas_meter.charge(GAS_STATE_WRITE)?;
        self.gas_meter
            .charge(GAS_BYTE_WRITE.saturating_mul(value.len() as u64))?;
        self.state.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    /// Queue a token transfer.
    /// Charges GAS_TRANSFER. Bounded to prevent memory exhaustion.
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: TokenId,
        amount: Amount,
    ) -> Result<(), LoomError> {
        self.gas_meter.charge(GAS_TRANSFER)?;
        if self.pending_transfers.len() >= MAX_PENDING_TRANSFERS {
            return Err(LoomError::RuntimeError {
                reason: "too many pending transfers".to_string(),
            });
        }
        self.pending_transfers.push(PendingTransfer {
            from,
            to,
            token_id,
            amount,
        });
        Ok(())
    }

    /// Emit a log message.
    /// Charges GAS_LOG. Bounded to prevent memory exhaustion.
    pub fn log(&mut self, message: &str) -> Result<(), LoomError> {
        self.gas_meter.charge(GAS_LOG)?;
        if self.logs.len() >= MAX_LOGS {
            return Err(LoomError::RuntimeError {
                reason: "too many log messages".to_string(),
            });
        }
        self.logs.push(message.to_string());
        Ok(())
    }

    /// Emit a structured event.
    /// Charges GAS_EMIT_EVENT. Bounded to prevent memory exhaustion.
    pub fn emit_event(
        &mut self,
        ty: String,
        attributes: Vec<(String, String)>,
    ) -> Result<(), LoomError> {
        self.gas_meter.charge(GAS_EMIT_EVENT)?;
        if self.events.len() >= MAX_EVENTS {
            return Err(LoomError::RuntimeError {
                reason: "too many events".to_string(),
            });
        }
        self.events.push(HostEvent { ty, attributes });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::primitives::NATIVE_TOKEN_ID;

    fn test_host_state() -> LoomHostState {
        LoomHostState::new([1u8; 20], 100, 1_000_000, DEFAULT_GAS_LIMIT)
    }

    #[test]
    fn test_state_get_set() {
        let mut host = test_host_state();

        // Set a key.
        host.state_set(b"counter", b"42").unwrap();

        // Get it back.
        let val = host.state_get(b"counter").unwrap();
        assert_eq!(val, Some(b"42".to_vec()));

        // Non-existent key returns None.
        let val = host.state_get(b"missing").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn test_state_gas_charging() {
        let mut host = LoomHostState::new([1u8; 20], 100, 1_000_000, 300);

        // A write costs GAS_STATE_WRITE (200) + GAS_BYTE_WRITE * 3 (6) = 206
        host.state_set(b"key", b"val").unwrap();
        assert_eq!(host.gas_meter.used(), 206);

        // A read costs GAS_STATE_READ (100) -- that would exceed the limit of 300.
        // 206 + 100 = 306 > 300
        let result = host.state_get(b"key");
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut host = test_host_state();
        let from = [1u8; 20];
        let to = [2u8; 20];
        host.transfer(from, to, NATIVE_TOKEN_ID, 1000).unwrap();

        assert_eq!(host.pending_transfers.len(), 1);
        assert_eq!(host.pending_transfers[0].amount, 1000);
        assert_eq!(host.pending_transfers[0].from, from);
        assert_eq!(host.pending_transfers[0].to, to);
        assert_eq!(host.gas_meter.used(), GAS_TRANSFER);
    }

    #[test]
    fn test_log() {
        let mut host = test_host_state();
        host.log("hello from loom").unwrap();

        assert_eq!(host.logs.len(), 1);
        assert_eq!(host.logs[0], "hello from loom");
        assert_eq!(host.gas_meter.used(), GAS_LOG);
    }

    #[test]
    fn test_transfer_gas_exhaustion() {
        let mut host = LoomHostState::new([1u8; 20], 100, 1_000_000, 400);
        let from = [1u8; 20];
        let to = [2u8; 20];

        // First transfer: 500 > 400 -- should fail.
        let result = host.transfer(from, to, NATIVE_TOKEN_ID, 1000);
        assert!(result.is_err());
    }
}
