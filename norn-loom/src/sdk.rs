use crate::error::LoomError;

/// Trait that loom smart contracts implement.
///
/// Each loom contract must provide `init`, `execute`, and `query` entry points.
/// The runtime calls these methods and mediates access to host functions (state,
/// transfers, logging) through the `LoomHostState`.
pub trait LoomContract {
    /// Called once when the loom contract is first deployed.
    fn init(&mut self) -> Result<(), LoomError>;

    /// Called to perform a state-mutating operation.
    ///
    /// The `input` bytes are contract-specific (e.g., borsh-encoded action
    /// enum). Returns output bytes on success.
    fn execute(&mut self, input: &[u8]) -> Result<Vec<u8>, LoomError>;

    /// Called for read-only queries against the current state.
    ///
    /// Must not modify state or emit transfers.
    fn query(&self, input: &[u8]) -> Result<Vec<u8>, LoomError>;
}

/// Helper to encode a u64 as little-endian bytes (useful for contract I/O).
pub fn encode_u64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Helper to decode a u64 from little-endian bytes.
pub fn decode_u64(bytes: &[u8]) -> Result<u64, LoomError> {
    if bytes.len() < 8 {
        return Err(LoomError::SerializationError {
            reason: format!("expected at least 8 bytes for u64, got {}", bytes.len()),
        });
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    Ok(u64::from_le_bytes(buf))
}

/// Helper to encode a u128 as little-endian bytes.
pub fn encode_u128(value: u128) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Helper to decode a u128 from little-endian bytes.
pub fn decode_u128(bytes: &[u8]) -> Result<u128, LoomError> {
    if bytes.len() < 16 {
        return Err(LoomError::SerializationError {
            reason: format!("expected at least 16 bytes for u128, got {}", bytes.len()),
        });
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&bytes[..16]);
    Ok(u128::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple counter contract for testing.
    struct CounterContract {
        counter: u64,
    }

    impl CounterContract {
        fn new() -> Self {
            Self { counter: 0 }
        }
    }

    impl LoomContract for CounterContract {
        fn init(&mut self) -> Result<(), LoomError> {
            self.counter = 0;
            Ok(())
        }

        fn execute(&mut self, input: &[u8]) -> Result<Vec<u8>, LoomError> {
            if input.is_empty() {
                // Default: increment by 1.
                self.counter += 1;
            } else {
                let amount = decode_u64(input)?;
                self.counter += amount;
            }
            Ok(encode_u64(self.counter))
        }

        fn query(&self, _input: &[u8]) -> Result<Vec<u8>, LoomError> {
            Ok(encode_u64(self.counter))
        }
    }

    #[test]
    fn test_mock_contract_init() {
        let mut contract = CounterContract::new();
        contract.init().unwrap();
        let val = contract.query(&[]).unwrap();
        assert_eq!(decode_u64(&val).unwrap(), 0);
    }

    #[test]
    fn test_mock_contract_execute() {
        let mut contract = CounterContract::new();
        contract.init().unwrap();

        // Increment by default (1).
        let result = contract.execute(&[]).unwrap();
        assert_eq!(decode_u64(&result).unwrap(), 1);

        // Increment by 5.
        let result = contract.execute(&encode_u64(5)).unwrap();
        assert_eq!(decode_u64(&result).unwrap(), 6);
    }

    #[test]
    fn test_mock_contract_query() {
        let mut contract = CounterContract::new();
        contract.init().unwrap();
        contract.execute(&[]).unwrap();
        contract.execute(&[]).unwrap();

        let val = contract.query(&[]).unwrap();
        assert_eq!(decode_u64(&val).unwrap(), 2);
    }

    #[test]
    fn test_encode_decode_u64() {
        let original = 123456789u64;
        let encoded = encode_u64(original);
        let decoded = decode_u64(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_u128() {
        let original = 987654321012345678u128;
        let encoded = encode_u128(original);
        let decoded = decode_u128(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_decode_u64_too_short() {
        let result = decode_u64(&[1, 2, 3]);
        assert!(result.is_err());
    }
}
