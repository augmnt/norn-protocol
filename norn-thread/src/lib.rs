//! Thread chain management and Knot validation for the Norn Protocol.
//!
//! Implements personal state chains (Threads), atomic state transitions (Knots),
//! state tracking, and version management.

pub mod chain;
pub mod knot;
pub mod state;
pub mod thread;
pub mod validation;
