use std::io;
use std::net::SocketAddr;

use bitcoin::p2p::address::Address;
use bitcoin::p2p::message::NetworkMessage;
use bitcoin::p2p::message_network::VersionMessage;
use bitcoin::p2p::{Magic, ServiceFlags};
use tokio::net::TcpStream;

use super::codec::{recv_message, send_message};

fn build_version_msg(peer_addr: SocketAddr) -> VersionMessage {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    VersionMessage {
        version: 70015,
        services: ServiceFlags::NONE,
        timestamp,
        receiver: Address::new(&peer_addr, ServiceFlags::NONE),
        sender: Address::new(&"0.0.0.0:0".parse().unwrap(), ServiceFlags::NONE),
        nonce: nonce(),
        user_agent: "/btc-client:0.1/".to_string(),
        start_height: 0,
        relay: false,
    }
}

fn nonce() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    h.finish()
}

pub async fn perform_handshake(
    stream: &mut TcpStream,
    magic: Magic,
    peer_addr: SocketAddr,
) -> io::Result<VersionMessage> {
    send_message(stream, magic, NetworkMessage::Version(build_version_msg(peer_addr))).await?;
    println!("[SEND] version");

    let mut peer_version: Option<VersionMessage> = None;
    let mut got_verack = false;

    while peer_version.is_none() || !got_verack {
        let raw = recv_message(stream).await?;
        match raw.payload().clone() {
            NetworkMessage::Version(v) => {
                println!("[RECV] version | agent='{}' height={}", v.user_agent, v.start_height);
                send_message(stream, magic, NetworkMessage::Verack).await?;
                println!("[SEND] verack");
                peer_version = Some(v);
            }
            NetworkMessage::Verack => {
                println!("[RECV] verack");
                got_verack = true;
            }
            other => {
                println!("[RECV] {} (durante handshake, ignored)", other.cmd());
            }
        }
    }

    println!("[INFO] Handshake complete ✓\n");
    Ok(peer_version.unwrap())
}
