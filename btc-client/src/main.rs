mod cli;
mod dns;
mod network;
mod peer;

use std::net::{SocketAddr, TcpStream};
use std::path::Path;

use bitcoin::p2p::Magic;
use clap::Parser;

use peer::model::{PeerInfo, PeerSource};
use peer::store::PeerStore;

const MAGIC: Magic = Magic::BITCOIN;

fn main() {
    let args = cli::Cli::parse();

    println!("=== btc-client ===\n");
    println!("[CFG] host        = {}", args.host);
    println!("[CFG] port        = {}", args.port);
    println!("[CFG] seeder_port = {}", args.seeder_port);
    println!("[CFG] threads     = {}", args.threads);
    println!("[CFG] timeout     = {}s", args.timeout);
    println!("[CFG] logfile     = {}", args.logfile.as_deref().unwrap_or("none"));
    println!();

    // ── Open database ─────────────────────────────────────────────────────────
    let store = PeerStore::open(Path::new("peers.db")).expect("Failed to open peer database");
    println!("[DB] Opened peers.db ({} peers known)\n", store.count().unwrap_or(0));

    let seeder_addr = SocketAddr::from(([127, 0, 0, 1], args.seeder_port));

    loop {
        // ── Try peers from database first ─────────────────────────────────────
        let known = store.load_reachable().unwrap_or_default();

        let addrs: Vec<SocketAddr> = if !known.is_empty() {
            println!("[DB] Trying {} known reachable peer(s)...", known.len());
            known.iter().map(|p| p.addr).collect()
        } else {
            // ── Fall back to DNS seeder ────────────────────────────────────────
            println!("[DNS] No known peers, querying seeder...");
            match dns::lookup_a(&args.host, seeder_addr) {
                Ok(ips) => {
                    let addrs: Vec<SocketAddr> = ips.iter()
                        .map(|ip| SocketAddr::from((*ip, args.port)))
                        .collect();
                    println!("[DNS] Resolved {} address(es) for {}:", addrs.len(), args.host);
                    for addr in &addrs {
                        println!("      {}", addr);
                    }
                    println!();
                    addrs
                }
                Err(e) => {
                    println!("[WARN] DNS query failed: {}, retrying in 10s...", e);
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    continue;
                }
            }
        };

        // ── Connect ───────────────────────────────────────────────────────────
        let connection = addrs.iter().find_map(|addr| {
            print!("[CONN] Trying {}... ", addr);
            let mut info = PeerInfo::new(*addr, PeerSource::DnsSeed);
            match TcpStream::connect(addr) {
                Ok(s) => {
                    println!("ok ✓");
                    info.mark_success(bitcoin::p2p::ServiceFlags::NONE);
                    store.upsert(&info).unwrap();
                    Some((s, *addr))
                }
                Err(e) => {
                    println!("failed ({})", e);
                    info.mark_attempt();
                    store.upsert(&info).unwrap();
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

        // ── Handshake ─────────────────────────────────────────────────────────
        let peer_ver = match network::handshake::perform_handshake(&mut stream, MAGIC, peer_addr) {
            Ok(v) => {
                println!(
                    "[INFO] Peer: agent='{}' | height={} | services={:?}\n",
                    v.user_agent, v.start_height, v.services
                );
                v
            }
            Err(e) => {
                println!("[WARN] Handshake failed: {}, reconnecting...", e);
                continue;
            }
        };

        // Update services from version message
        if let Ok(mut info) = store.load_all().map(|peers| {
            peers.into_iter().find(|p| p.addr == peer_addr)
        }).map(|opt| opt.unwrap_or_else(|| PeerInfo::new(peer_addr, PeerSource::DnsSeed))) {
            info.mark_success(peer_ver.services);
            store.upsert(&info).unwrap();
        }

        // ── Message loop ──────────────────────────────────────────────────────
        match network::client::run(&mut stream, MAGIC) {
            Ok(discovered) => {
                println!("[INFO] Connection closed. Discovered {} peer(s).", discovered.len());
                // Save discovered peers to database
                for addr in discovered {
                    let info = PeerInfo::new(addr, PeerSource::AddrMsg);
                    store.upsert(&info).unwrap();
                }
                println!("[DB] Total peers known: {}", store.count().unwrap_or(0));
            }
            Err(e) => println!("[WARN] Connection error: {}", e),
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
