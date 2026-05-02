use std::io::{self, Cursor};

use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::p2p::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::p2p::Magic;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// Bitcoin P2P message header is always 24 bytes:
//   4  magic
//   12 command
//   4  payload length
//   4  checksum
const HEADER_SIZE: usize = 24;

pub async fn send_message(
    stream: &mut TcpStream,
    magic: Magic,
    msg: NetworkMessage,
) -> io::Result<()> {
    let raw = RawNetworkMessage::new(magic, msg);
    let mut buf = Vec::new();
    raw.consensus_encode(&mut buf)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    stream.write_all(&buf).await?;
    stream.flush().await
}

pub async fn recv_message(stream: &mut TcpStream) -> io::Result<RawNetworkMessage> {
    // ── 1. Read header ────────────────────────────────────────────────────────
    let mut header = [0u8; HEADER_SIZE];
    stream.read_exact(&mut header).await?;

    // ── 2. Parse payload length from bytes [16..20] (little-endian u32) ──────
    let payload_len = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;

    // ── 3. Read payload ───────────────────────────────────────────────────────
    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        stream.read_exact(&mut payload).await?;
    }

    // ── 4. Reassemble and decode ──────────────────────────────────────────────
    let mut full = Vec::with_capacity(HEADER_SIZE + payload_len);
    full.extend_from_slice(&header);
    full.extend_from_slice(&payload);

    RawNetworkMessage::consensus_decode(&mut Cursor::new(&full))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::p2p::message::NetworkMessage;

    fn encode_message(magic: Magic, msg: NetworkMessage) -> Vec<u8> {
        let raw = RawNetworkMessage::new(magic, msg);
        let mut buf = Vec::new();
        raw.consensus_encode(&mut buf).unwrap();
        buf
    }

    #[test]
    fn test_roundtrip_ping() {
        let magic = Magic::BITCOIN;
        let nonce = 0xDEADBEEFu64;
        let bytes = encode_message(magic, NetworkMessage::Ping(nonce));
        let decoded =
            RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(*decoded.magic(), magic);
        assert!(matches!(decoded.payload(), NetworkMessage::Ping(n) if *n == nonce));
    }

    #[test]
    fn test_roundtrip_verack() {
        let magic = Magic::BITCOIN;
        let bytes = encode_message(magic, NetworkMessage::Verack);
        let decoded =
            RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();
        assert!(matches!(decoded.payload(), NetworkMessage::Verack));
    }

    #[test]
    fn test_wrong_magic_still_decodes() {
        let magic = Magic::TESTNET3;
        let bytes = encode_message(magic, NetworkMessage::Ping(1));
        let decoded =
            RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(*decoded.magic(), Magic::TESTNET3);
    }

    #[test]
    fn test_corrupted_payload_fails() {
        let magic = Magic::BITCOIN;
        let mut bytes = encode_message(magic, NetworkMessage::Ping(42));
        let len = bytes.len();
        bytes[len - 5] ^= 0xFF;
        let result = RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes));
        assert!(result.is_err());
    }
}
