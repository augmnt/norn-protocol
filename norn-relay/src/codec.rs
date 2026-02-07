use async_trait::async_trait;
use borsh::BorshDeserialize;
use futures::prelude::*;
use libp2p::swarm::StreamProtocol;
use norn_types::constants::MAX_MESSAGE_SIZE;
use norn_types::network::NornMessage;
use std::io;

use crate::error::RelayError;
use crate::protocol::PROTOCOL_VERSION;

/// Borsh-based length-prefixed codec for NornMessage over libp2p request-response.
///
/// Wire format: `[4-byte length][1-byte protocol version][borsh payload]`
///
/// The length prefix covers the version byte + payload (i.e. `1 + payload.len()`).
/// On receive, the version byte is checked; a mismatch produces a clear IO error.
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
/// Wire format: `[4-byte BE length][1-byte version][borsh payload]`
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
            "message too short: missing protocol version byte",
        ));
    }

    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;

    // First byte is the protocol version.
    let version = buf[0];
    if version != PROTOCOL_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "protocol version mismatch: peer sent v{}, we run v{} — disconnecting",
                version, PROTOCOL_VERSION
            ),
        ));
    }

    NornMessage::try_from_slice(&buf[1..])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write a versioned, length-prefixed message to an async writer.
///
/// Wire format: `[4-byte BE length][1-byte version][borsh payload]`
async fn write_length_prefixed_message<T>(io: &mut T, msg: &NornMessage) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let data = borsh::to_vec(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

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

    // Length = 1 (version byte) + payload length.
    let len = ((1 + data.len()) as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(&[PROTOCOL_VERSION]).await?;
    io.write_all(&data).await?;
    Ok(())
}

// ─── Gossipsub helpers ──────────────────────────────────────────────────────

/// Encode a NornMessage into a versioned, length-prefixed byte vector for gossipsub.
///
/// Wire format: `[4-byte BE length][1-byte version][borsh payload]`
pub fn encode_message(msg: &NornMessage) -> Result<Vec<u8>, RelayError> {
    let data = borsh::to_vec(msg).map_err(|e| RelayError::CodecError {
        reason: e.to_string(),
    })?;

    if data.len() > MAX_MESSAGE_SIZE {
        return Err(RelayError::MessageTooLarge {
            size: data.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }

    // Length = 1 (version byte) + payload length.
    let len = ((1 + data.len()) as u32).to_be_bytes();
    let mut out = Vec::with_capacity(4 + 1 + data.len());
    out.extend_from_slice(&len);
    out.push(PROTOCOL_VERSION);
    out.extend_from_slice(&data);
    Ok(out)
}

/// Decode a versioned, length-prefixed byte slice into a NornMessage (gossipsub helper).
pub fn decode_message(data: &[u8]) -> Result<NornMessage, RelayError> {
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
            reason: "message too short: missing protocol version byte".to_string(),
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

    // Check protocol version.
    let version = data[4];
    if version != PROTOCOL_VERSION {
        return Err(RelayError::VersionMismatch {
            peer: version,
            ours: PROTOCOL_VERSION,
        });
    }

    NornMessage::try_from_slice(&data[5..4 + len]).map_err(|e| RelayError::CodecError {
        reason: e.to_string(),
    })
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
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_decode_too_short() {
        let result = decode_message(&[0u8; 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_too_large_encode() {
        let len = (MAX_MESSAGE_SIZE + 1) as u32;
        let mut data = Vec::new();
        data.extend_from_slice(&len.to_be_bytes());
        // Append enough dummy bytes (version + payload).
        data.extend_from_slice(&vec![0u8; MAX_MESSAGE_SIZE + 1]);

        let result = decode_message(&data);
        assert!(matches!(result, Err(RelayError::MessageTooLarge { .. })));
    }

    #[test]
    fn test_decode_truncated_body() {
        let msg = sample_message();
        let encoded = encode_message(&msg).expect("encode failed");
        // Truncate the body.
        let truncated = &encoded[..encoded.len() - 5];
        let result = decode_message(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_version_mismatch_detected() {
        let msg = sample_message();
        let mut encoded = encode_message(&msg).expect("encode failed");
        // Corrupt the version byte (byte at index 4).
        encoded[4] = PROTOCOL_VERSION + 1;
        let result = decode_message(&encoded);
        assert!(matches!(result, Err(RelayError::VersionMismatch { .. })));
    }

    #[test]
    fn test_encode_includes_version_byte() {
        let msg = sample_message();
        let encoded = encode_message(&msg).expect("encode failed");
        // Byte at index 4 should be the protocol version.
        assert_eq!(encoded[4], PROTOCOL_VERSION);
        // Length prefix should account for version byte.
        let len = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;
        assert_eq!(len, encoded.len() - 4); // length = everything after the 4-byte prefix
    }
}
