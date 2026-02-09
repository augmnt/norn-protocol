//! Guard macros for early-return error handling in contracts.
//!
//! These macros provide concise, readable assertions that return
//! `Err(ContractError)` when a condition is not met.
//!
//! # Examples
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! fn withdraw(ctx: &Context, amount: u128, balance: u128) -> ContractResult {
//!     ensure!(amount > 0, "amount must be positive");
//!     ensure!(amount <= balance, ContractError::InsufficientFunds);
//!     ensure_eq!(ctx.sender(), owner, ContractError::Unauthorized);
//!     // ...
//! }
//! ```

/// Return early with an error if the condition is false.
///
/// The error argument can be a `ContractError`, a `&str`, or any type that
/// implements `Into<ContractError>`.
///
/// ```ignore
/// ensure!(amount > 0, "amount must be positive");
/// ensure!(ctx.sender() == owner, ContractError::Unauthorized);
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return ::core::result::Result::Err(::core::convert::Into::into($err));
        }
    };
}

/// Return early with an error if two values are not equal.
///
/// ```ignore
/// ensure_eq!(ctx.sender(), owner, ContractError::Unauthorized);
/// ```
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $err:expr) => {
        if $left != $right {
            return ::core::result::Result::Err(::core::convert::Into::into($err));
        }
    };
}

/// Return early with an error if two values are equal.
///
/// ```ignore
/// ensure_ne!(recipient, ZERO_ADDRESS, "cannot send to zero address");
/// ```
#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr, $err:expr) => {
        if $left == $right {
            return ::core::result::Result::Err(::core::convert::Into::into($err));
        }
    };
}
