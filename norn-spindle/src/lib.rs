//! Watchtower service for the Norn Protocol.
//!
//! Monitors the Weave on behalf of offline users, detects fraudulent activity
//! (double-knots, stale commits), constructs fraud proofs, and manages rate
//! limiting for proof submission.

pub mod error;
pub mod monitor;
pub mod rate_limit;
pub mod service;
