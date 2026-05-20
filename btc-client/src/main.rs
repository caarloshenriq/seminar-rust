mod cli;
mod dns;
mod logger;
mod network;
mod peer;

use std::net::SocketAddr;
use std::path::Path;

use bitcoin::p2p::Magic;
use clap::Parser;
use tokio::net::TcpStream;

use logger::event::{Event, LogLevel, Subsystem};
use peer::model::{PeerInfo, PeerSource};
use peer::store::PeerStore;

const MAGIC: Magic = Magic::BITCOIN;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    let min_level = match args.verbosity {
        cli::Verbosity::Trace => LogLevel::Trace,
        cli::Verbosity::Debug => LogLevel::Debug,
        cli::Verbosity::Info  => LogLevel::Info,
        cli::Verbosity::Warn  => LogLevel::Warn,
        cli::Verbosity::Error => LogLevel::Error,
    };

    let (log, _log_handle) = logger::spawn(min_level);

    log.info(Subsystem::Cli, Event::Custom(format!(
        "starting btc-client | host={} port={} seeder_port={}",
        args.host, args.port, args.seeder_port
    )));

    let store = PeerStore::open(Path::new("peers.db")).expect("Failed to open peer database");
    let count = store.count().unwrap_or(0);
    log.info(Subsystem::Database, Event::DatabaseOpened(count as usize));

    let seeder_addr = SocketAddr::from(([127, 0, 0, 1], args.seeder_port));

    loop {
        // ── Select peers to try ───────────────────────────────────────────────
        let addrs: Vec<SocketAddr> = {
            let reachable = store.load_reachable().unwrap_or_default();
            if !reachable.is_empty() {
                log.debug(Subsystem::Database, Event::Custom(
                    format!("trying {} known reachable peer(s)", reachable.len())
                ));
                reachable.iter().map(|p| p.addr).collect()
            } else {
                let never_tried = store.load_never_tried().unwrap_or_default();
                if !never_tried.is_empty() {
                    log.debug(Subsystem::Database, Event::Custom(
                        format!("trying {} never-tried peer(s) from db", never_tried.len())
                    ));
                    never_tried.iter().map(|p| p.addr).collect()
                } else {
                    log.info(Subsystem::Dns, Event::Custom("no known peers, querying seeder...".into()));
                    match dns::lookup_a(&args.host, seeder_addr) {
                        Ok(ips) => {
                            log.info(Subsystem::Dns, Event::DnsResolved(args.host.clone(), ips.len()));
                            ips.iter().map(|ip| SocketAddr::from((*ip, args.port))).collect()
                        }
                        Err(e) => {
                            log.warn(Subsystem::Dns, Event::DnsFailed(args.host.clone(), e.to_string()));
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                }
            }
        };

        // ── Connect ───────────────────────────────────────────────────────────
        let connection = {
            let mut result = None;
            for addr in &addrs {
                let mut info = PeerInfo::new(*addr, PeerSource::DnsSeed);
                match TcpStream::connect(addr).await {
                    Ok(s) => {
                        log.info(Subsystem::Network, Event::Connected(*addr));
                        info.mark_success(bitcoin::p2p::ServiceFlags::NONE);
                        store.upsert(&info).unwrap();
                        result = Some((s, *addr));
                        break;
                    }
                    Err(e) => {
                        log.warn(Subsystem::Network, Event::FailedConnection(*addr, e.to_string()));
                        info.mark_attempt();
                        store.upsert(&info).unwrap();
                    }
                }
            }
            result
        };

        let (mut stream, peer_addr) = match connection {
            Some(c) => c,
            None => {
                log.warn(Subsystem::Network, Event::Custom("no address reachable, retrying in 10s".into()));
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };

        // ── Handshake ─────────────────────────────────────────────────────────
        match network::handshake::perform_handshake(&mut stream, MAGIC, peer_addr).await {
            Ok(v) => {
                log.info(Subsystem::Network, Event::HandshakeComplete(
                    peer_addr,
                    v.user_agent.clone(),
                    v.start_height,
                ));

                let mut info = store.load_all()
                    .unwrap_or_default()
                    .into_iter()
                    .find(|p| p.addr == peer_addr)
                    .unwrap_or_else(|| PeerInfo::new(peer_addr, PeerSource::DnsSeed));
                info.mark_success(v.services);
                store.upsert(&info).unwrap();
                log.debug(Subsystem::Database, Event::PeerUpdated(peer_addr, info.status));
            }
            Err(e) => {
                log.warn(Subsystem::Network, Event::HandshakeFailed(peer_addr, e.to_string()));
                // mark as unreachable so we don't retry immediately
                let mut info = PeerInfo::new(peer_addr, PeerSource::DnsSeed);
                info.mark_attempt();
                store.upsert(&info).unwrap();
                continue;
            }
        }

        // ── Message loop ──────────────────────────────────────────────────────
        match network::client::run(&mut stream, MAGIC).await {
            Ok(discovered) => {
                log.info(Subsystem::Network, Event::ConnectionClosed(peer_addr));
                for addr in &discovered {
                    let info = PeerInfo::new(*addr, PeerSource::AddrMsg);
                    store.upsert(&info).unwrap();
                    log.trace(Subsystem::Database, Event::PeerSaved(*addr));
                }
                log.info(Subsystem::Database, Event::Custom(
                    format!("saved {} discovered peer(s), total={}", discovered.len(), store.count().unwrap_or(0))
                ));
            }
            Err(e) => {
                log.error(Subsystem::Network, Event::Custom(format!("connection error: {}", e)));
                // mark as unreachable so we don't retry immediately
                let mut info = PeerInfo::new(peer_addr, PeerSource::AddrMsg);
                info.mark_attempt();
                store.upsert(&info).unwrap();
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
