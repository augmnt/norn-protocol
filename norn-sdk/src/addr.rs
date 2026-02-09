//! Address utility functions for loom contracts.

use alloc::string::String;

use crate::error::ContractError;
use crate::types::Address;

/// The zero address `[0u8; 20]`.
pub const ZERO_ADDRESS: Address = [0u8; 20];

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Convert an address to a hex string with `0x` prefix.
///
/// ```ignore
/// let s = addr_to_hex(&[0xab, 0xcd, /* ... */]);
/// assert!(s.starts_with("0x"));
/// ```
pub fn addr_to_hex(addr: &Address) -> String {
    let mut s = String::with_capacity(42); // "0x" + 40 hex chars
    s.push_str("0x");
    for &byte in addr {
        s.push(HEX_CHARS[(byte >> 4) as usize] as char);
        s.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    s
}

/// Parse a hex string (with or without `0x` prefix) into an address.
///
/// Returns `ContractError::InvalidInput` if the string is not valid hex
/// or not exactly 20 bytes.
pub fn hex_to_addr(hex: &str) -> Result<Address, ContractError> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 40 {
        return Err(ContractError::invalid_input(
            "address must be 20 bytes (40 hex chars)",
        ));
    }
    let mut addr = [0u8; 20];
    let bytes = hex.as_bytes();
    for i in 0..20 {
        let hi = hex_nibble(bytes[i * 2])?;
        let lo = hex_nibble(bytes[i * 2 + 1])?;
        addr[i] = (hi << 4) | lo;
    }
    Ok(addr)
}

fn hex_nibble(c: u8) -> Result<u8, ContractError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(ContractError::invalid_input("invalid hex character")),
    }
}
