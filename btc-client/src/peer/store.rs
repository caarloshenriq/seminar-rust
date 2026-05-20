use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use rusqlite::{Connection, Result, params};
use bitcoin::p2p::ServiceFlags;

use super::model::{PeerInfo, PeerSource, PeerStatus};

pub struct PeerStore {
    conn: Connection,
}

impl PeerStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS peers (
                ip              TEXT NOT NULL,
                port            INTEGER NOT NULL,
                services        INTEGER NOT NULL DEFAULT 0,
                status          TEXT NOT NULL DEFAULT 'NeverTried',
                source          TEXT NOT NULL DEFAULT 'DnsSeed',
                first_seen      INTEGER NOT NULL,
                last_attempt    INTEGER,
                last_success    INTEGER,
                attempt_count   INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (ip, port)
            );
        ")?;
        Ok(())
    }

    pub fn upsert(&self, peer: &PeerInfo) -> Result<()> {
        self.conn.execute(
            "INSERT INTO peers (ip, port, services, status, source, first_seen, last_attempt, last_success, attempt_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(ip, port) DO UPDATE SET
                services      = excluded.services,
                status        = excluded.status,
                last_attempt  = excluded.last_attempt,
                last_success  = excluded.last_success,
                attempt_count = excluded.attempt_count",
            params![
                peer.addr.ip().to_string(),
                peer.addr.port(),
                peer.services.to_u64() as i64,
                status_to_str(&peer.status),
                source_to_str(&peer.source),
                peer.first_seen as i64,
                peer.last_attempt.map(|t| t as i64),
                peer.last_success.map(|t| t as i64),
                peer.attempt_count,
            ],
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<PeerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT ip, port, services, status, source, first_seen, last_attempt, last_success, attempt_count
             FROM peers"
        )?;

        let peers = stmt.query_map([], |row| {
            let ip: String = row.get(0)?;
            let port: u16 = row.get(1)?;
            let services: i64 = row.get(2)?;
            let status: String = row.get(3)?;
            let source: String = row.get(4)?;
            let first_seen: i64 = row.get(5)?;
            let last_attempt: Option<i64> = row.get(6)?;
            let last_success: Option<i64> = row.get(7)?;
            let attempt_count: u32 = row.get(8)?;

            let ip: IpAddr = ip.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;
            let addr = SocketAddr::new(ip, port);

            Ok(PeerInfo {
                addr,
                services: ServiceFlags::from(services as u64),
                status: str_to_status(&status),
                source: str_to_source(&source),
                first_seen: first_seen as u64,
                last_attempt: last_attempt.map(|t| t as u64),
                last_success: last_success.map(|t| t as u64),
                attempt_count,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(peers)
    }

    pub fn load_reachable(&self) -> Result<Vec<PeerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT ip, port, services, status, source, first_seen, last_attempt, last_success, attempt_count
             FROM peers
             WHERE status = 'Reachable'
             ORDER BY last_success DESC"
        )?;

        let peers = stmt.query_map([], |row| {
            let ip: String = row.get(0)?;
            let port: u16 = row.get(1)?;
            let services: i64 = row.get(2)?;
            let status: String = row.get(3)?;
            let source: String = row.get(4)?;
            let first_seen: i64 = row.get(5)?;
            let last_attempt: Option<i64> = row.get(6)?;
            let last_success: Option<i64> = row.get(7)?;
            let attempt_count: u32 = row.get(8)?;

            let ip: IpAddr = ip.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;
            let addr = SocketAddr::new(ip, port);

            Ok(PeerInfo {
                addr,
                services: ServiceFlags::from(services as u64),
                status: str_to_status(&status),
                source: str_to_source(&source),
                first_seen: first_seen as u64,
                last_attempt: last_attempt.map(|t| t as u64),
                last_success: last_success.map(|t| t as u64),
                attempt_count,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(peers)
    }

    pub fn count(&self) -> Result<u64> {
        let count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM peers", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    pub fn load_never_tried(&self) -> Result<Vec<PeerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT ip, port, services, status, source, first_seen, last_attempt, last_success, attempt_count
             FROM peers
             WHERE status = 'NeverTried'
             ORDER BY first_seen DESC
             LIMIT 50"
        )?;

        let peers = stmt.query_map([], |row| {
            let ip: String = row.get(0)?;
            let port: u16 = row.get(1)?;
            let services: i64 = row.get(2)?;
            let status: String = row.get(3)?;
            let source: String = row.get(4)?;
            let first_seen: i64 = row.get(5)?;
            let last_attempt: Option<i64> = row.get(6)?;
            let last_success: Option<i64> = row.get(7)?;
            let attempt_count: u32 = row.get(8)?;

            let ip: std::net::IpAddr = ip.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;
            let addr = SocketAddr::new(ip, port);

            Ok(PeerInfo {
                addr,
                services: bitcoin::p2p::ServiceFlags::from(services as u64),
                status: str_to_status(&status),
                source: str_to_source(&source),
                first_seen: first_seen as u64,
                last_attempt: last_attempt.map(|t| t as u64),
                last_success: last_success.map(|t| t as u64),
                attempt_count,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(peers)
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn status_to_str(s: &PeerStatus) -> &'static str {
    match s {
        PeerStatus::NeverTried  => "NeverTried",
        PeerStatus::Reachable   => "Reachable",
        PeerStatus::Unreachable => "Unreachable",
        PeerStatus::Banned      => "Banned",
    }
}

fn str_to_status(s: &str) -> PeerStatus {
    match s {
        "Reachable"   => PeerStatus::Reachable,
        "Unreachable" => PeerStatus::Unreachable,
        "Banned"      => PeerStatus::Banned,
        _             => PeerStatus::NeverTried,
    }
}

fn source_to_str(s: &PeerSource) -> &'static str {
    match s {
        PeerSource::DnsSeed => "DnsSeed",
        PeerSource::AddrMsg => "AddrMsg",
        PeerSource::Manual  => "Manual",
    }
}

fn str_to_source(s: &str) -> PeerSource {
    match s {
        "AddrMsg" => PeerSource::AddrMsg,
        "Manual"  => PeerSource::Manual,
        _         => PeerSource::DnsSeed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn in_memory_store() -> PeerStore {
        PeerStore::open(Path::new(":memory:")).expect("failed to open in-memory store")
    }

    fn make_peer(ip: [u8; 4], port: u16, source: PeerSource) -> PeerInfo {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port);
        PeerInfo::new(addr, source)
    }

    #[test]
    fn test_empty_store() {
        let store = in_memory_store();
        assert_eq!(store.count().unwrap(), 0);
        assert!(store.load_all().unwrap().is_empty());
    }

    #[test]
    fn test_upsert_and_load() {
        let store = in_memory_store();
        let peer = make_peer([1, 2, 3, 4], 8333, PeerSource::DnsSeed);

        store.upsert(&peer).unwrap();

        assert_eq!(store.count().unwrap(), 1);
        let loaded = store.load_all().unwrap();
        assert_eq!(loaded[0].addr, peer.addr);
        assert!(matches!(loaded[0].status, PeerStatus::NeverTried));
        assert!(matches!(loaded[0].source, PeerSource::DnsSeed));
    }

    #[test]
    fn test_upsert_updates_existing() {
        let store = in_memory_store();
        let mut peer = make_peer([1, 2, 3, 4], 8333, PeerSource::DnsSeed);
        store.upsert(&peer).unwrap();

        peer.mark_success(ServiceFlags::NONE);
        store.upsert(&peer).unwrap();

        assert_eq!(store.count().unwrap(), 1);
        let loaded = store.load_all().unwrap();
        assert!(matches!(loaded[0].status, PeerStatus::Reachable));
        assert!(loaded[0].last_success.is_some());
    }

    #[test]
    fn test_mark_attempt_sets_unreachable() {
        let store = in_memory_store();
        let mut peer = make_peer([5, 6, 7, 8], 8333, PeerSource::AddrMsg);

        peer.mark_attempt();
        store.upsert(&peer).unwrap();

        let loaded = store.load_all().unwrap();
        assert!(matches!(loaded[0].status, PeerStatus::Unreachable));
        assert_eq!(loaded[0].attempt_count, 1);
        assert!(loaded[0].last_attempt.is_some());
    }

    #[test]
    fn test_load_reachable_only() {
        let store = in_memory_store();

        let peer_a = make_peer([1, 1, 1, 1], 8333, PeerSource::DnsSeed);
        let mut peer_b = make_peer([2, 2, 2, 2], 8333, PeerSource::DnsSeed);
        let mut peer_c = make_peer([3, 3, 3, 3], 8333, PeerSource::DnsSeed);

        peer_b.mark_success(ServiceFlags::NONE);
        peer_c.mark_attempt();

        store.upsert(&peer_a).unwrap();
        store.upsert(&peer_b).unwrap();
        store.upsert(&peer_c).unwrap();

        let reachable = store.load_reachable().unwrap();
        assert_eq!(reachable.len(), 1);
        assert_eq!(reachable[0].addr, peer_b.addr);
    }

    #[test]
    fn test_multiple_peers() {
        let store = in_memory_store();

        for i in 1..=5 {
            let peer = make_peer([i, i, i, i], 8333, PeerSource::DnsSeed);
            store.upsert(&peer).unwrap();
        }

        assert_eq!(store.count().unwrap(), 5);
    }
}
