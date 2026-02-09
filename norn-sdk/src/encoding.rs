//! Encoding helpers for contract I/O.

use alloc::vec::Vec;

/// Encode a u64 as 8 little-endian bytes.
pub fn encode_u64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Decode a u64 from little-endian bytes.
pub fn decode_u64(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    Some(u64::from_le_bytes(buf))
}

/// Encode a u128 as 16 little-endian bytes.
pub fn encode_u128(value: u128) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Decode a u128 from little-endian bytes.
pub fn decode_u128(bytes: &[u8]) -> Option<u128> {
    if bytes.len() < 16 {
        return None;
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&bytes[..16]);
    Some(u128::from_le_bytes(buf))
}
