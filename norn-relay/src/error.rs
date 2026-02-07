use thiserror::Error;

/// Errors that can occur in the relay layer.
#[derive(Debug, Error)]
pub enum RelayError {
    /// A general network-level error.
    #[error("network error: {reason}")]
    NetworkError { reason: String },

    /// Failed to encode or decode a message.
    #[error("codec error: {reason}")]
    CodecError { reason: String },

    /// Failed to establish or maintain a connection.
    #[error("connection error: {reason}")]
    ConnectionError { reason: String },

    /// The requested peer was not found.
    #[error("peer not found: {peer}")]
    PeerNotFound { peer: String },

    /// Message exceeds maximum allowed size.
    #[error("message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    /// Protocol-level error.
    #[error("protocol error: {reason}")]
    ProtocolError { reason: String },

    /// Channel send/receive error.
    #[error("channel error: {reason}")]
    ChannelError { reason: String },

    /// Peer is running an incompatible protocol version.
    #[error("protocol version mismatch: peer sent v{peer}, we run v{ours}")]
    VersionMismatch { peer: u8, ours: u8 },
}
