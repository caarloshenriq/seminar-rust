use std::net::SocketAddr;
use bitcoin::p2p::ServiceFlags;

#[derive(Debug, Clone)]
pub enum PeerStatus {
    NeverTried,
    Reachable,
    Unreachable,
    Banned,
}

#[derive(Debug, Clone)]
pub enum PeerSource {
    DnsSeed,
    AddrMsg,
    Manual,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub addr: SocketAddr,
    pub services: ServiceFlags,
    pub status: PeerStatus,
    pub source: PeerSource,
    pub first_seen: u64,
    pub last_attempt: Option<u64>,
    pub last_success: Option<u64>,
    pub attempt_count: u32,
}

impl PeerInfo {
    pub fn new(addr: SocketAddr, source: PeerSource) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            addr,
            services: ServiceFlags::NONE,
            status: PeerStatus::NeverTried,
            source,
            first_seen: now,
            last_attempt: None,
            last_success: None,
            attempt_count: 0,
        }
    }

    pub fn mark_attempt(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_attempt = Some(now);
        self.attempt_count += 1;
        self.status = PeerStatus::Unreachable;
    }

    pub fn mark_success(&mut self, services: ServiceFlags) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_success = Some(now);
        self.services = services;
        self.status = PeerStatus::Reachable;
    }
}
