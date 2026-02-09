use crate::error::LoomError;

// ─── Gas Cost Constants ─────────────────────────────────────────────────────

/// Cost per Wasm instruction executed.
pub const GAS_PER_INSTRUCTION: u64 = 1;

/// Cost for a single state read operation.
pub const GAS_STATE_READ: u64 = 100;

/// Cost for a single state write operation.
pub const GAS_STATE_WRITE: u64 = 200;

/// Cost per byte read from state.
pub const GAS_BYTE_READ: u64 = 1;

/// Cost per byte written to state.
pub const GAS_BYTE_WRITE: u64 = 2;

/// Cost for a single token transfer operation.
pub const GAS_TRANSFER: u64 = 500;

/// Cost for a single log emission.
pub const GAS_LOG: u64 = 50;

/// Cost for emitting a structured event.
pub const GAS_EMIT_EVENT: u64 = 75;

/// Default gas limit when none is specified.
pub const DEFAULT_GAS_LIMIT: u64 = 10_000_000;

// ─── Gas Meter ──────────────────────────────────────────────────────────────

/// Tracks gas consumption during loom execution.
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Maximum gas allowed.
    pub limit: u64,
    /// Gas consumed so far.
    pub used: u64,
}

impl GasMeter {
    /// Create a new gas meter with the given limit.
    pub fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Charge the given amount of gas. Returns an error if the limit is exceeded.
    pub fn charge(&mut self, amount: u64) -> Result<(), LoomError> {
        let new_used = self.used.saturating_add(amount);
        if new_used > self.limit {
            // Set used to the attempted total so the error message is informative.
            self.used = new_used;
            return Err(LoomError::GasExhausted {
                used: new_used,
                limit: self.limit,
            });
        }
        self.used = new_used;
        Ok(())
    }

    /// Return the remaining gas.
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Return gas consumed so far.
    pub fn used(&self) -> u64 {
        self.used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_meter_charge() {
        let mut meter = GasMeter::new(1000);
        assert!(meter.charge(100).is_ok());
        assert_eq!(meter.used(), 100);
        assert_eq!(meter.remaining(), 900);
    }

    #[test]
    fn test_gas_meter_exhaust() {
        let mut meter = GasMeter::new(100);
        assert!(meter.charge(50).is_ok());
        assert!(meter.charge(51).is_err());
    }

    #[test]
    fn test_gas_meter_remaining() {
        let mut meter = GasMeter::new(500);
        assert_eq!(meter.remaining(), 500);
        meter.charge(200).unwrap();
        assert_eq!(meter.remaining(), 300);
    }

    #[test]
    fn test_gas_meter_exact_limit() {
        let mut meter = GasMeter::new(100);
        assert!(meter.charge(100).is_ok());
        assert_eq!(meter.remaining(), 0);
        assert!(meter.charge(1).is_err());
    }

    #[test]
    fn test_gas_meter_zero_charge() {
        let mut meter = GasMeter::new(100);
        assert!(meter.charge(0).is_ok());
        assert_eq!(meter.used(), 0);
        assert_eq!(meter.remaining(), 100);
    }
}
