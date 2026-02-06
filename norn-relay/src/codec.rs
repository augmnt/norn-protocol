use async_trait::async_trait;
use borsh::BorshDeserialize;
use futures::prelude::*;
use libp2p::swarm::StreamProtocol;
use norn_types::constants::MAX_MESSAGE_SIZE;
use norn_types::network::NornMessage;
use std::io;

use crate::error::RelayError;

/// Borsh-based length-prefixed codec for NornMessage over libp2p request-response.
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

/// Read a 4-byte big-endian length prefix, then the body, and borsh-decode it.
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

    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;

    NornMessage::try_from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write a 4-byte big-endian length prefix followed by the borsh-encoded body.
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

    let len = (data.len() as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(&data).await?;
    Ok(())
}

// ─── Gossipsub helpers ──────────────────────────────────────────────────────

/// Encode a NornMessage into a length-prefixed borsh byte vector for gossipsub.
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

    let len = (data.len() as u32).to_be_bytes();
    let mut out = Vec::with_capacity(4 + data.len());
    out.extend_from_slice(&len);
    out.extend_from_slice(&data);
    Ok(out)
}

/// Decode a length-prefixed borsh byte slice into a NornMessage (gossipsub helper).
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

    if data.len() < 4 + len {
        return Err(RelayError::CodecError {
            reason: format!(
                "data too short: expected {} bytes, got {}",
                4 + len,
                data.len()
            ),
        });
    }

    NornMessage::try_from_slice(&data[4..4 + len]).map_err(|e| RelayError::CodecError {
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
        // Create a relay message with a huge payload to exceed MAX_MESSAGE_SIZE.
        // We can't easily create a NornMessage that serializes to > 2MB in a unit test,
        // so instead test the check with the decode path using a crafted header.
        let len = (MAX_MESSAGE_SIZE + 1) as u32;
        let mut data = Vec::new();
        data.extend_from_slice(&len.to_be_bytes());
        // Append enough dummy bytes.
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
}
