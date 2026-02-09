//! Type aliases used across loom contract code.

use borsh::{BorshDeserialize, BorshSerialize};

/// A 20-byte account address.
pub type Address = [u8; 20];

/// A 32-byte token identifier.
pub type TokenId = [u8; 32];

/// Unit type for contracts that don't need constructor arguments.
///
/// Use `type Init = Empty;` in your `Contract` impl when the init
/// function doesn't need any parameters.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Empty;
