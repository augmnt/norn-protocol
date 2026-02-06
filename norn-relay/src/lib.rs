//! P2P networking and message relay for the Norn Protocol.
//!
//! Built on libp2p with gossipsub for message propagation, request-response for
//! direct communication, peer discovery, and a Spindle registry for watchtower
//! service coordination.

pub mod behaviour;
pub mod codec;
pub mod config;
pub mod discovery;
pub mod error;
pub mod peer_manager;
pub mod protocol;
pub mod relay;
pub mod spindle_registry;
