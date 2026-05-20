#![allow(dead_code)]
use std::net::SocketAddr;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use bitcoin::p2p::ServiceFlags;

use super::model::{PeerInfo, PeerSource};
use super::store::PeerStore;

/// Commands that any async task can send to the DB actor.
#[derive(Debug)]
pub enum DbCommand {
    Upsert(PeerInfo),
    MarkReachable(SocketAddr, ServiceFlags),
    MarkUnreachable(SocketAddr),
    Shutdown,
}

/// A cheap, cloneable handle to the DB actor.
#[derive(Clone)]
pub struct DbHandle {
    tx: mpsc::Sender<DbCommand>,
}

impl DbHandle {
    pub fn upsert(&self, peer: PeerInfo) {
        let _ = self.tx.send(DbCommand::Upsert(peer));
    }

    pub fn mark_reachable(&self, addr: SocketAddr, services: ServiceFlags) {
        let _ = self.tx.send(DbCommand::MarkReachable(addr, services));
    }

    pub fn mark_unreachable(&self, addr: SocketAddr) {
        let _ = self.tx.send(DbCommand::MarkUnreachable(addr));
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(DbCommand::Shutdown);
    }
}

/// Spawns the DB actor from a file path.
pub fn spawn(db_path: &Path) -> (DbHandle, thread::JoinHandle<()>) {
    let store = PeerStore::open(db_path).expect("DB actor: failed to open database");
    println!("[DB] Actor started on {:?}", db_path);
    spawn_with_store(store)
}

/// Spawns the DB actor from an already-open store (useful for tests).
pub fn spawn_with_store(store: PeerStore) -> (DbHandle, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<DbCommand>();

    let handle = thread::spawn(move || {
        for cmd in rx {
            match cmd {
                DbCommand::Upsert(peer) => {
                    if let Err(e) = store.upsert(&peer) {
                        eprintln!("[DB] upsert error: {}", e);
                    }
                }
                DbCommand::MarkReachable(addr, services) => {
                    let mut peer = load_or_new(&store, addr, PeerSource::DnsSeed);
                    peer.mark_success(services);
                    if let Err(e) = store.upsert(&peer) {
                        eprintln!("[DB] mark_reachable error: {}", e);
                    }
                }
                DbCommand::MarkUnreachable(addr) => {
                    let mut peer = load_or_new(&store, addr, PeerSource::DnsSeed);
                    peer.mark_attempt();
                    if let Err(e) = store.upsert(&peer) {
                        eprintln!("[DB] mark_unreachable error: {}", e);
                    }
                }
                DbCommand::Shutdown => break,
            }
        }
    });

    (DbHandle { tx }, handle)
}

fn load_or_new(store: &PeerStore, addr: SocketAddr, source: PeerSource) -> PeerInfo {
    store
        .load_all()
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.addr == addr)
        .unwrap_or_else(|| PeerInfo::new(addr, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use super::super::model::PeerStatus;

    fn addr(ip: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
    }

    fn tmp_path() -> String {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        format!("/tmp/btc_actor_test_{}.db", n)
    }

    /// Runs a closure against a fresh actor, shuts it down, returns the store for verification.
    fn run_and_read<F: FnOnce(&DbHandle)>(f: F) -> PeerStore {
        let path = tmp_path();
        let _ = std::fs::remove_file(&path);
        let store = PeerStore::open(Path::new(&path)).unwrap();
        let (db, thread) = spawn_with_store(store);
        f(&db);
        db.shutdown();
        thread.join().unwrap();
        PeerStore::open(Path::new(&path)).unwrap()
    }

    // ── Upsert ────────────────────────────────────────────────────────────────

    #[test]
    fn test_upsert_inserts_peer() {
        let store = run_and_read(|db| {
            db.upsert(PeerInfo::new(addr([1, 2, 3, 4], 8333), PeerSource::DnsSeed));
        });
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn test_upsert_multiple_peers() {
        let store = run_and_read(|db| {
            for i in 1u8..=5 {
                db.upsert(PeerInfo::new(addr([i, i, i, i], 8333), PeerSource::DnsSeed));
            }
        });
        assert_eq!(store.count().unwrap(), 5);
    }

    // ── MarkReachable ─────────────────────────────────────────────────────────

    #[test]
    fn test_mark_reachable_sets_status() {
        let a = addr([10, 0, 0, 1], 8333);
        let store = run_and_read(|db| {
            db.mark_reachable(a, ServiceFlags::NONE);
        });
        let peers = store.load_reachable().unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].addr, a);
        assert!(peers[0].last_success.is_some());
    }

    #[test]
    fn test_mark_reachable_upgrades_unreachable_peer() {
        let a = addr([10, 0, 0, 2], 8333);
        let store = run_and_read(|db| {
            let mut peer = PeerInfo::new(a, PeerSource::DnsSeed);
            peer.mark_attempt();
            db.upsert(peer);
            db.mark_reachable(a, ServiceFlags::NONE);
        });
        let peers = store.load_reachable().unwrap();
        assert_eq!(peers.len(), 1);
    }

    // ── MarkUnreachable ───────────────────────────────────────────────────────

    #[test]
    fn test_mark_unreachable_sets_status() {
        let a = addr([10, 0, 0, 3], 8333);
        let store = run_and_read(|db| {
            db.mark_unreachable(a);
        });
        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert!(matches!(all[0].status, PeerStatus::Unreachable));
        assert_eq!(all[0].attempt_count, 1);
    }

    #[test]
    fn test_mark_unreachable_increments_attempts() {
        let a = addr([10, 0, 0, 4], 8333);
        let store = run_and_read(|db| {
            db.mark_unreachable(a);
            db.mark_unreachable(a);
            db.mark_unreachable(a);
        });
        let all = store.load_all().unwrap();
        assert_eq!(all[0].attempt_count, 3);
    }

    // ── Concorrência ──────────────────────────────────────────────────────────

    #[test]
    fn test_multiple_handles_write_concurrently() {
        let path = tmp_path();
        let _ = std::fs::remove_file(&path);
        let store = PeerStore::open(Path::new(&path)).unwrap();
        let (db, actor_thread) = spawn_with_store(store);

        // 4 threads simultâneas, cada uma com seu clone do handle
        let threads: Vec<_> = (1u8..=4)
            .map(|i| {
                let db = db.clone();
                thread::spawn(move || {
                    db.upsert(PeerInfo::new(addr([i, i, i, i], 8333), PeerSource::AddrMsg));
                })
            })
            .collect();

        for t in threads {
            t.join().unwrap();
        }

        db.shutdown();
        actor_thread.join().unwrap();

        let result = PeerStore::open(Path::new(&path)).unwrap();
        assert_eq!(result.count().unwrap(), 4);
    }

    // ── Shutdown ──────────────────────────────────────────────────────────────

    #[test]
    fn test_shutdown_joins_cleanly() {
        let store = PeerStore::open(Path::new(":memory:")).unwrap();
        let (db, thread) = spawn_with_store(store);
        db.shutdown();
        thread.join().unwrap(); // não deve travar
    }
}
