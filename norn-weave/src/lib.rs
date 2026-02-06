//! Anchor chain engine for the Norn Protocol.
//!
//! Implements block production, commitment processing, HotStuff BFT consensus,
//! EIP-1559-style dynamic fees, fraud proof verification, and validator staking.

pub mod block;
pub mod commitment;
pub mod consensus;
pub mod engine;
pub mod error;
pub mod fees;
pub mod fraud;
pub mod leader;
pub mod mempool;
pub mod registration;
pub mod staking;
