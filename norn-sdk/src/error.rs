//! Contract error type for loom smart contracts.

use alloc::string::String;

/// Errors that a contract can return from `execute` or `query`.
#[derive(Debug, PartialEq)]
pub enum ContractError {
    /// A custom error with a free-form message.
    Custom(String),
    /// The caller is not authorized to perform the action.
    Unauthorized,
    /// The input data could not be parsed or is invalid.
    InvalidInput(String),
    /// A requested resource was not found.
    NotFound(String),
    /// An arithmetic overflow occurred.
    Overflow,
    /// The account has insufficient funds for the operation.
    InsufficientFunds,
}

impl ContractError {
    /// Human-readable error message for this variant.
    pub fn message(&self) -> &str {
        match self {
            ContractError::Custom(msg) => msg,
            ContractError::Unauthorized => "unauthorized",
            ContractError::InvalidInput(msg) => msg,
            ContractError::NotFound(msg) => msg,
            ContractError::Overflow => "arithmetic overflow",
            ContractError::InsufficientFunds => "insufficient funds",
        }
    }

    /// Create a custom error with a message.
    pub fn custom(msg: impl Into<String>) -> Self {
        ContractError::Custom(msg.into())
    }

    /// Create a not-found error describing what was missing.
    pub fn not_found(what: impl Into<String>) -> Self {
        ContractError::NotFound(what.into())
    }

    /// Create an invalid-input error describing the problem.
    pub fn invalid_input(what: impl Into<String>) -> Self {
        ContractError::InvalidInput(what.into())
    }
}

impl core::fmt::Display for ContractError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.message())
    }
}

impl From<&str> for ContractError {
    fn from(msg: &str) -> Self {
        ContractError::Custom(String::from(msg))
    }
}
