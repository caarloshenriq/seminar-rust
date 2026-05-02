use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use bitcoin::p2p::message::NetworkMessage;
use bitcoin::p2p::Magic;
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::codec::{recv_message, send_message};

const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Post-handshake message loop.
/// Returns the list of peer addresses discovered via addr/addrv2 messages.
pub async fn run(stream: &mut TcpStream, magic: Magic) -> io::Result<Vec<SocketAddr>> {
    send_message(stream, magic, NetworkMessage::GetAddr).await?;
    println!("[SEND] getaddr\n");

    let mut discovered: Vec<SocketAddr> = Vec::new();

    loop {
        let raw = match timeout(READ_TIMEOUT, recv_message(stream)).await {
            Ok(Ok(msg))  => msg,
            Ok(Err(e))   => return Err(e),
            Err(_elapsed) => {
                println!("[INFO] Timeout — peer idle, disconnecting.");
                break;
            }
        };

        match raw.payload().clone() {
            NetworkMessage::Ping(nonce) => {
                println!("[RECV] ping  nonce={}", nonce);
                send_message(stream, magic, NetworkMessage::Pong(nonce)).await?;
                println!("[SEND] pong  nonce={}", nonce);
            }
            NetworkMessage::Pong(nonce) => {
                println!("[RECV] pong  nonce={}", nonce);
            }
            NetworkMessage::Addr(addrs) => {
                println!("[RECV] addr  {} address(es)", addrs.len());
                for (_, addr) in &addrs {
                    if let Ok(sa) = addr.socket_addr() {
                        discovered.push(sa);
                    }
                }
            }
            NetworkMessage::AddrV2(addrs) => {
                println!("[RECV] addrv2  {} address(es)", addrs.len());
                for addr in &addrs {
                    if let Ok(sa) = addr.socket_addr() {
                        discovered.push(sa);
                    }
                }
            }
            other => {
                println!("[RECV] {} (ignored)", other.cmd());
            }
        }
    }

    Ok(discovered)
}
