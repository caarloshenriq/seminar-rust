mod cli;
mod dns;
mod network;
mod peer;

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use bitcoin::p2p::Magic;
use clap::Parser;
use tokio::net::TcpStream;
use tokio::time::timeout;

use peer::db_actor::{DbHandle, self as db_actor};
use peer::model::{PeerInfo, PeerSource};
use peer::store::PeerStore;

const MAGIC: Magic = Magic::BITCOIN;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    println!("=== btc-client ===\n");
    println!("[CFG] host        = {}", args.host);
    println!("[CFG] port        = {}", args.port);
    println!("[CFG] seeder_port = {}", args.seeder_port);
    println!("[CFG] threads     = {}", args.threads);
    println!("[CFG] timeout     = {}s", args.timeout);
    println!("[CFG] logfile     = {}", args.logfile.as_deref().unwrap_or("none"));
    println!();

    // ── Spawn the DB actor (owns SQLite, runs in its own thread) ──────────────
    let db_path = Path::new("peers.db");
    let (db, _db_thread) = db_actor::spawn(db_path);

    // Quick read-only access to seed the first round of peers (before tasks start)
    let store = PeerStore::open(db_path).expect("Failed to open peer database");
    println!("[DB] Opened peers.db ({} peers known)\n", store.count().unwrap_or(0));

    let seeder_addr = SocketAddr::from(([127, 0, 0, 1], args.seeder_port));
    let parallelism = args.threads.max(1) as usize;

    loop {
        // ── Collect candidate addresses ───────────────────────────────────────
        let known = store.load_reachable().unwrap_or_default();

        let addrs: Vec<SocketAddr> = if !known.is_empty() {
            println!("[DB] Trying {} known reachable peer(s)...", known.len());
            known.iter().map(|p| p.addr).take(parallelism * 2).collect()
        } else {
            println!("[DNS] No known peers, querying seeder...");
            match dns::lookup_a(&args.host, seeder_addr) {
                Ok(ips) => {
                    let addrs: Vec<SocketAddr> = ips.iter()
                        .map(|ip| SocketAddr::from((*ip, args.port)))
                        .collect();
                    println!("[DNS] Resolved {} address(es)", addrs.len());
                    addrs
                }
                Err(e) => {
                    println!("[WARN] DNS failed: {}, retrying in 10s...", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            }
        };

        if addrs.is_empty() {
            println!("[WARN] No addresses available, retrying in 10s...");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }

        // ── Spawn parallel crawl tasks ────────────────────────────────────────
        let batch: Vec<SocketAddr> = addrs.into_iter().take(parallelism).collect();
        println!("[CRAWL] Spawning {} parallel crawl task(s)...\n", batch.len());

        let handles: Vec<_> = batch
            .into_iter()
            .map(|addr| {
                let db = db.clone();
                tokio::spawn(async move {
                    crawl_peer(addr, db).await;
                })
            })
            .collect();

        // Wait for all tasks in this batch to finish
        for h in handles {
            let _ = h.await;
        }

        println!("\n[CRAWL] Batch complete. Total peers: {}", store.count().unwrap_or(0));
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // (unreachable in normal operation, but good practice)
    #[allow(unreachable_code, unused_variables)]
    {
        db.shutdown();
        let _ = _db_thread.join();
    }
}

// ── Crawl a single peer ───────────────────────────────────────────────────────

async fn crawl_peer(addr: SocketAddr, db: DbHandle) {
    println!("[{}] Connecting...", addr);

    // ── TCP connect with timeout ──────────────────────────────────────────────
    let mut stream = match timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
        Ok(Ok(s)) => {
            println!("[{}] Connected ✓", addr);
            db.mark_reachable(addr, bitcoin::p2p::ServiceFlags::NONE);
            s
        }
        Ok(Err(e)) => {
            println!("[{}] Connection failed: {}", addr, e);
            db.mark_unreachable(addr);
            return;
        }
        Err(_) => {
            println!("[{}] Connection timed out", addr);
            db.mark_unreachable(addr);
            return;
        }
    };

    // ── Handshake ─────────────────────────────────────────────────────────────
    let peer_ver = match network::handshake::perform_handshake(&mut stream, MAGIC, addr).await {
        Ok(v) => {
            println!("[{}] agent='{}' height={}", addr, v.user_agent, v.start_height);
            db.mark_reachable(addr, v.services);
            v
        }
        Err(e) => {
            println!("[{}] Handshake failed: {}", addr, e);
            db.mark_unreachable(addr);
            return;
        }
    };

    // ── Message loop ──────────────────────────────────────────────────────────
    match network::client::run(&mut stream, MAGIC).await {
        Ok(discovered) => {
            println!("[{}] Done. Discovered {} peer(s).", addr, discovered.len());
            for peer_addr in discovered {
                let info = PeerInfo::new(peer_addr, PeerSource::AddrMsg);
                db.upsert(info);
            }
        }
        Err(e) => {
            println!("[{}] Error in message loop: {}", addr, e);
        }
    }

    let _ = peer_ver; // suppress unused warning
}
