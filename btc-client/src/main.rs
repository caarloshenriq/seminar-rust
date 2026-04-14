mod cli;
mod dns;
mod network;

use bitcoin::p2p::Magic;
use clap::Parser;
use std::net::{SocketAddr, TcpStream};

const MAGIC: Magic = Magic::BITCOIN;

fn main() {
    let args = cli::Cli::parse();

    println!("=== btc-client ===\n");
    println!("[CFG] host        = {}", args.host);
    println!("[CFG] port        = {}", args.port);
    println!("[CFG] seeder_port = {}", args.seeder_port);
    println!("[CFG] threads     = {}", args.threads);
    println!("[CFG] timeout     = {}s", args.timeout);
    println!(
        "[CFG] logfile     = {}",
        args.logfile.as_deref().unwrap_or("none")
    );
    println!();

    let seeder_addr = SocketAddr::from(([127, 0, 0, 1], args.seeder_port));

    loop {
        let ips = match dns::lookup_a(&args.host, seeder_addr) {
            Ok(ips) => ips,
            Err(e) => {
                println!("[WARN] DNS query failed: {}, retrying in 10s...", e);
                std::thread::sleep(std::time::Duration::from_secs(10));
                continue;
            }
        };

        let addrs: Vec<SocketAddr> = ips
            .iter()
            .map(|ip| SocketAddr::from((*ip, args.port)))
            .collect();

        println!(
            "[DNS] Resolved {} address(es) for {}:",
            addrs.len(),
            args.host
        );
        for addr in &addrs {
            println!("      {}", addr);
        }
        println!();

        let connection = addrs.iter().find_map(|addr| {
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
        });

        let (mut stream, peer_addr) = match connection {
            Some(c) => c,
            None => {
                println!("[WARN] No address reachable, retrying in 10s...");
                std::thread::sleep(std::time::Duration::from_secs(10));
                continue;
            }
        };

        println!("[CONN] Connected to {}\n", peer_addr);

        match network::handshake::perform_handshake(&mut stream, MAGIC, peer_addr) {
            Ok(peer_ver) => {
                println!(
                    "[INFO] Peer: agent='{}' | height={} | services={:?}\n",
                    peer_ver.user_agent, peer_ver.start_height, peer_ver.services
                );
            }
            Err(e) => {
                println!("[WARN] Handshake failed: {}, reconnecting...", e);
                continue;
            }
        }

        match network::client::run(&mut stream, MAGIC) {
            Ok(_) => println!("[INFO] Connection closed, reconnecting..."),
            Err(e) => println!("[WARN] Connection error: {}, reconnecting...", e),
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
