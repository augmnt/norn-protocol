use async_trait::async_trait;
use borsh::BorshDeserialize;
use futures::prelude::*;
use libp2p::swarm::StreamProtocol;
use norn_types::constants::MAX_MESSAGE_SIZE;
use norn_types::network::{MessageEnvelope, NornMessage};
use std::io;

use crate::error::RelayError;
use crate::protocol::{ENVELOPE_VERSION, LEGACY_PROTOCOL_VERSION, PROTOCOL_VERSION};

/// Result of decoding a gossipsub or request-response message.
/// `Known` means we successfully deserialized a `NornMessage`.
/// `Unknown` means the envelope contained a message type we don't recognize
/// (forward-compatible: the sender is running a newer protocol).
#[derive(Debug, Clone)]
pub enum DecodedMessage {
    /// A successfully decoded message.
    Known(Box<NornMessage>),
    /// An unknown message type from a newer protocol version.
    Unknown {
        /// The protocol version of the sender.
        protocol_version: u8,
        /// The message type discriminant we don't recognize.
        message_type: u8,
    },
}

/// Borsh-based length-prefixed codec for NornMessage over libp2p request-response.
///
/// Wire format (envelope): `[4-byte length][1-byte ENVELOPE_VERSION][borsh MessageEnvelope]`
/// Legacy format:          `[4-byte length][1-byte LEGACY_PROTOCOL_VERSION(3)][borsh NornMessage]`
///
/// The codec auto-detects which format is in use by inspecting byte[4].
#[derive(Debug, Clone)]
pub struct NornCodec;

impl Default for NornCodec {
    fn default() -> Self {
        NornCodec
    }
}

#[async_trait]
impl libp2p::request_response::Codec for NornCodec {
    type Protocol = StreamProtocol;
    type Request = NornMessage;
    type Response = NornMessage;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_length_prefixed_message(io).await
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_length_prefixed_message(io).await
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed_message(io, &req).await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed_message(io, &res).await
    }
}

/// Read a versioned, length-prefixed message from an async reader.
///
/// Supports dual-decode: envelope format (byte[0] == ENVELOPE_VERSION) and
/// legacy format (byte[0] == LEGACY_PROTOCOL_VERSION).
async fn read_length_prefixed_message<T>(io: &mut T) -> io::Result<NornMessage>
where
    T: AsyncRead + Unpin + Send,
{
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            ),
        ));
    }

    if len < 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too short: missing version byte",
        ));
    }

    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;

    let version_byte = buf[0];

    if version_byte == ENVELOPE_VERSION {
        // New envelope format: deserialize MessageEnvelope from buf[1..].
        let envelope = MessageEnvelope::try_from_slice(&buf[1..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        envelope.unwrap_message().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unknown message type {} from protocol v{}",
                    envelope.message_type, envelope.protocol_version
                ),
            )
        })
    } else if version_byte == LEGACY_PROTOCOL_VERSION {
        // Legacy format: raw borsh NornMessage after version byte.
        NornMessage::try_from_slice(&buf[1..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "protocol version mismatch: peer sent v{}, expected envelope v{} or legacy v{}",
                version_byte, ENVELOPE_VERSION, LEGACY_PROTOCOL_VERSION
            ),
        ))
    }
}

/// Write a versioned, length-prefixed message to an async writer using the
/// envelope format.
///
/// Wire format: `[4-byte BE length][1-byte ENVELOPE_VERSION][borsh MessageEnvelope]`
async fn write_length_prefixed_message<T>(io: &mut T, msg: &NornMessage) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let envelope = MessageEnvelope::wrap(msg, PROTOCOL_VERSION)?;
    let data =
        borsh::to_vec(&envelope).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if data.len() > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "message too large: {} bytes (max {})",
                data.len(),
                MAX_MESSAGE_SIZE
            ),
        ));
    }

    // Length = 1 (envelope version byte) + envelope payload length.
    let len = ((1 + data.len()) as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(&[ENVELOPE_VERSION]).await?;
    io.write_all(&data).await?;
    Ok(())
}

// ─── Gossipsub helpers ──────────────────────────────────────────────────────

/// Encode a NornMessage into an envelope-format byte vector for gossipsub.
///
/// Wire format: `[4-byte BE length][1-byte ENVELOPE_VERSION][borsh MessageEnvelope]`
pub fn encode_message(msg: &NornMessage) -> Result<Vec<u8>, RelayError> {
    let envelope =
        MessageEnvelope::wrap(msg, PROTOCOL_VERSION).map_err(|e| RelayError::CodecError {
            reason: e.to_string(),
        })?;
    let data = borsh::to_vec(&envelope).map_err(|e| RelayError::CodecError {
        reason: e.to_string(),
    })?;

    if data.len() > MAX_MESSAGE_SIZE {
        return Err(RelayError::MessageTooLarge {
            size: data.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }

    // Length = 1 (envelope version byte) + envelope payload length.
    let len = ((1 + data.len()) as u32).to_be_bytes();
    let mut out = Vec::with_capacity(4 + 1 + data.len());
    out.extend_from_slice(&len);
    out.push(ENVELOPE_VERSION);
    out.extend_from_slice(&data);
    Ok(out)
}

/// Encode a NornMessage in the legacy v3 wire format (raw NornMessage, no envelope).
///
/// Rejects messages with discriminant > 13 (those didn't exist in v3).
///
/// Wire format: `[4-byte BE length][1-byte LEGACY_PROTOCOL_VERSION(3)][borsh NornMessage]`
pub fn encode_message_legacy(msg: &NornMessage) -> Result<Vec<u8>, RelayError> {
    if msg.discriminant() > 13 {
        return Err(RelayError::CodecError {
            reason: format!(
                "message type {} not supported in legacy protocol v{}",
                msg.discriminant(),
                LEGACY_PROTOCOL_VERSION
            ),
        });
    }

    let data = borsh::to_vec(msg).map_err(|e| RelayError::CodecError {
        reason: e.to_string(),
    })?;

    if data.len() > MAX_MESSAGE_SIZE {
        return Err(RelayError::MessageTooLarge {
            size: data.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }

    let len = ((1 + data.len()) as u32).to_be_bytes();
    let mut out = Vec::with_capacity(4 + 1 + data.len());
    out.extend_from_slice(&len);
    out.push(LEGACY_PROTOCOL_VERSION);
    out.extend_from_slice(&data);
    Ok(out)
}

/// Decode a gossipsub message, returning `DecodedMessage` to handle both
/// known and unknown message types gracefully.
///
/// Supports dual-decode: envelope (byte[4] == ENVELOPE_VERSION) and
/// legacy (byte[4] == LEGACY_PROTOCOL_VERSION).
pub fn decode_message(data: &[u8]) -> Result<DecodedMessage, RelayError> {
    if data.len() < 4 {
        return Err(RelayError::CodecError {
            reason: "data too short for length prefix".to_string(),
        });
    }

    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(RelayError::MessageTooLarge {
            size: len,
            max: MAX_MESSAGE_SIZE,
        });
    }

    if len < 1 {
        return Err(RelayError::CodecError {
            reason: "message too short: missing version byte".to_string(),
        });
    }

    if data.len() < 4 + len {
        return Err(RelayError::CodecError {
            reason: format!(
                "data too short: expected {} bytes, got {}",
                4 + len,
                data.len()
            ),
        });
    }

    let version_byte = data[4];

    if version_byte == ENVELOPE_VERSION {
        // New envelope format.
        let envelope = MessageEnvelope::try_from_slice(&data[5..4 + len]).map_err(|e| {
            RelayError::CodecError {
                reason: format!("envelope decode error: {}", e),
            }
        })?;

        match envelope.unwrap_message() {
            Some(msg) => Ok(DecodedMessage::Known(Box::new(msg))),
            None => Ok(DecodedMessage::Unknown {
                protocol_version: envelope.protocol_version,
                message_type: envelope.message_type,
            }),
        }
    } else if version_byte == LEGACY_PROTOCOL_VERSION {
        // Legacy format: raw borsh NornMessage.
        let msg =
            NornMessage::try_from_slice(&data[5..4 + len]).map_err(|e| RelayError::CodecError {
                reason: format!("legacy decode error: {}", e),
            })?;
        Ok(DecodedMessage::Known(Box::new(msg)))
    } else {
        Err(RelayError::VersionMismatch {
            peer: version_byte,
            ours: ENVELOPE_VERSION,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::network::NornMessage;
    use norn_types::weave::Registration;

    fn sample_message() -> NornMessage {
        NornMessage::Registration(Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        })
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let msg = sample_message();
        let encoded = encode_message(&msg).expect("encode failed");
        let decoded = decode_message(&encoded).expect("decode failed");
        match decoded {
            DecodedMessage::Known(m) => assert_eq!(msg, *m),
            DecodedMessage::Unknown { .. } => panic!("expected Known"),
        }
    }

    #[test]
    fn test_decode_too_short() {
        let result = decode_message(&[0u8; 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_too_large_decode() {
        let len = (MAX_MESSAGE_SIZE + 1) as u32;
        let mut data = Vec::new();
        data.extend_from_slice(&len.to_be_bytes());
        data.extend_from_slice(&vec![0u8; MAX_MESSAGE_SIZE + 1]);

        let result = decode_message(&data);
        assert!(matches!(result, Err(RelayError::MessageTooLarge { .. })));
    }

    #[test]
    fn test_decode_truncated_body() {
        let msg = sample_message();
        let encoded = encode_message(&msg).expect("encode failed");
        let truncated = &encoded[..encoded.len() - 5];
        let result = decode_message(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_version_mismatch_detected() {
        let msg = sample_message();
        let mut encoded = encode_message(&msg).expect("encode failed");
        // Corrupt the version byte to something neither envelope nor legacy.
        encoded[4] = 99;
        let result = decode_message(&encoded);
        assert!(matches!(result, Err(RelayError::VersionMismatch { .. })));
    }

    #[test]
    fn test_encode_includes_envelope_version_byte() {
        let msg = sample_message();
        let encoded = encode_message(&msg).expect("encode failed");
        // Byte at index 4 should be the envelope version.
        assert_eq!(encoded[4], ENVELOPE_VERSION);
        // Length prefix should account for version byte.
        let len = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;
        assert_eq!(len, encoded.len() - 4);
    }

    #[test]
    fn test_legacy_encode_decode_roundtrip() {
        let msg = sample_message();
        let encoded = encode_message_legacy(&msg).expect("legacy encode failed");
        // byte[4] should be LEGACY_PROTOCOL_VERSION.
        assert_eq!(encoded[4], LEGACY_PROTOCOL_VERSION);
        // Should still decode via dual-decode.
        let decoded = decode_message(&encoded).expect("decode failed");
        match decoded {
            DecodedMessage::Known(m) => assert_eq!(msg, *m),
            DecodedMessage::Unknown { .. } => panic!("expected Known"),
        }
    }

    #[test]
    fn test_legacy_encode_rejects_new_variants() {
        let msg = NornMessage::UpgradeNotice(norn_types::network::UpgradeNotice {
            protocol_version: 5,
            message: "test".to_string(),
            timestamp: 1000,
        });
        let result = encode_message_legacy(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_unknown_message_type() {
        // Manually construct an envelope with an unknown message type.
        let envelope = MessageEnvelope {
            version: 1,
            protocol_version: 99,
            message_type: 255,
            payload: vec![0xFF, 0xFF, 0xFF],
        };
        let envelope_bytes = borsh::to_vec(&envelope).unwrap();
        let len = ((1 + envelope_bytes.len()) as u32).to_be_bytes();
        let mut data = Vec::new();
        data.extend_from_slice(&len);
        data.push(ENVELOPE_VERSION);
        data.extend_from_slice(&envelope_bytes);

        let decoded = decode_message(&data).expect("should not error for unknown type");
        match decoded {
            DecodedMessage::Unknown {
                protocol_version,
                message_type,
            } => {
                assert_eq!(protocol_version, 99);
                assert_eq!(message_type, 255);
            }
            DecodedMessage::Known(_) => panic!("expected Unknown"),
        }
    }
}
