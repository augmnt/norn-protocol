//! Standard library modules for common contract patterns.
//!
//! Provides reusable building blocks inspired by OpenZeppelin:
//! - [`Ownable`] — single-owner access control
//! - [`Pausable`] — emergency pause/unpause
//! - [`Norn20`] — ERC20-equivalent fungible token

pub mod norn20;
pub mod ownable;
pub mod pausable;

pub use norn20::{Norn20, Norn20Info};
pub use ownable::Ownable;
pub use pausable::Pausable;
