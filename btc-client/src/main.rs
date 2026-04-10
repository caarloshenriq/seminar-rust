mod dns;
mod network;

use bitcoin::p2p::Magic;
use std::net::{SocketAddr, TcpStream};

const MAGIC: Magic = Magic::BITCOIN;
const SEEDER_HOSTNAME: &str = "dnsseed.local";
const SEEDER_ADDR: &str = "127.0.0.1:15353";
const BTC_PORT: u16 = 8333;

fn main() {
    println!("=== btc-client ===\n");

    let server: SocketAddr = SEEDER_ADDR.parse().unwrap();
    let ips = dns::lookup_a(SEEDER_HOSTNAME, server).expect("DNS query failed");

    let addrs: Vec<SocketAddr> = ips
        .iter()
        .map(|ip| SocketAddr::from((*ip, BTC_PORT)))
        .collect();

    println!(
        "[DNS] Resolved {} address(es) for {}:",
        addrs.len(),
        SEEDER_HOSTNAME
    );
    for addr in &addrs {
        println!("      {}", addr);
    }
    println!();

    let (mut stream, peer_addr) = addrs
        .iter()
        .find_map(|addr| {
            print!("[CONN] Trying {}... ", addr);
            match TcpStream::connect(addr) {
                Ok(s) => {
                    println!("ok ✓");
                    Some((s, *addr))
                }
                Err(e) => {
                    println!("failed ({})", e);
                    None
                }
            }
        })
        .expect("Could not connect to any address");

    println!("[CONN] Connected to {}\n", peer_addr);

    let peer_ver = network::handshake::perform_handshake(&mut stream, MAGIC, peer_addr)
        .expect("Handshake failed");

    println!(
        "[INFO] Peer: agent='{}' | height={} | services={:?}\n",
        peer_ver.user_agent, peer_ver.start_height, peer_ver.services
    );

    network::client::run(&mut stream, MAGIC).expect("Message loop error");

    println!("\n[INFO] Done.");
}
