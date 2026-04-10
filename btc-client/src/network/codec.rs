use std::io::{self, Write};
use std::net::TcpStream;

use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::p2p::Magic;
use bitcoin::p2p::message::{NetworkMessage, RawNetworkMessage};

pub fn send_message(stream: &mut TcpStream, magic: Magic, msg: NetworkMessage) -> io::Result<()> {
    let raw = RawNetworkMessage::new(magic, msg);
    let mut buf = Vec::new();
    raw.consensus_encode(&mut buf)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    stream.write_all(&buf)?;
    stream.flush()
}

pub fn recv_message(stream: &mut TcpStream) -> io::Result<RawNetworkMessage> {
    RawNetworkMessage::consensus_decode(stream)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use bitcoin::p2p::message::NetworkMessage;
    use bitcoin::consensus::{Decodable, Encodable};

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
        let decoded = RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();

        assert_eq!(*decoded.magic(), magic);
        assert!(matches!(decoded.payload(), NetworkMessage::Ping(n) if *n == nonce));
    }

    #[test]
    fn test_roundtrip_verack() {
        let magic = Magic::BITCOIN;
        let bytes = encode_message(magic, NetworkMessage::Verack);
        let decoded = RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();

        assert_eq!(*decoded.magic(), magic);
        assert!(matches!(decoded.payload(), NetworkMessage::Verack));
    }

    #[test]
    fn test_wrong_magic_still_decodes() {
        let magic = Magic::TESTNET3;
        let bytes = encode_message(magic, NetworkMessage::Ping(1));
        let decoded = RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes)).unwrap();

        assert_eq!(*decoded.magic(), Magic::TESTNET3);
    }

    #[test]
    fn test_corrupted_payload_fails() {
        let magic = Magic::BITCOIN;
        let mut bytes = encode_message(magic, NetworkMessage::Ping(42));

        // corrupt the checksum
        let len = bytes.len();
        bytes[len - 5] ^= 0xFF;

        let result = RawNetworkMessage::consensus_decode(&mut Cursor::new(&bytes));
        assert!(result.is_err());
    }
}
