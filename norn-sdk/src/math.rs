//! Safe arithmetic helpers for contract math.
//!
//! These replace verbose `checked_add(x).ok_or(ContractError::Overflow)?` patterns
//! with concise `safe_add(a, b)?` calls.

use crate::error::ContractError;

/// Add two `u128` values, returning `ContractError::Overflow` on overflow.
pub fn safe_add(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_add(b).ok_or(ContractError::Overflow)
}

/// Subtract `b` from `a`, returning `ContractError::InsufficientFunds` on underflow.
pub fn safe_sub(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_sub(b).ok_or(ContractError::InsufficientFunds)
}

/// Multiply two `u128` values, returning `ContractError::Overflow` on overflow.
pub fn safe_mul(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_mul(b).ok_or(ContractError::Overflow)
}
