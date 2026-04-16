use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use bitcoin::p2p::message::NetworkMessage;
use bitcoin::p2p::Magic;

use super::codec::{recv_message, send_message};

pub fn run(stream: &mut TcpStream, magic: Magic) -> io::Result<Vec<SocketAddr>> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;

    send_message(stream, magic, NetworkMessage::GetAddr)?;
    println!("[SEND] getaddr\n");

    let mut discovered: Vec<SocketAddr> = Vec::new();

    loop {
        let mut buf = [0u8; 1];
        match stream.peek(&mut buf) {
            Ok(0) => {
                println!("[INFO] Peer closed the connection.");
                break;
            }
            Err(ref e) if is_timeout(e) => {
                println!("[INFO] Timeout — peer idle, disconnecting.");
                break;
            }
            Err(e) => return Err(e),
            Ok(_) => {}
        }

        let raw = match recv_message(stream) {
            Ok(m) => m,
            Err(ref e) if is_timeout(e) => {
                println!("[INFO] Timeout receiving message.");
                break;
            }
            Err(e) => return Err(e),
        };

        match raw.payload().clone() {
            NetworkMessage::Ping(nonce) => {
                println!("[RECV] ping  nonce={}", nonce);
                send_message(stream, magic, NetworkMessage::Pong(nonce))?;
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

fn is_timeout(e: &io::Error) -> bool {
    matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
}
