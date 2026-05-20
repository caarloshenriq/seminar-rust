use std::net::SocketAddr;
use crate::peer::model::PeerStatus;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO ",
            LogLevel::Warn  => "WARN ",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum Subsystem {
    Dns,
    Network,
    Database,
    Cli,
}

impl std::fmt::Display for Subsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Subsystem::Dns      => "dns     ",
            Subsystem::Network  => "network ",
            Subsystem::Database => "database",
            Subsystem::Cli      => "cli     ",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    // Network
    Connected(SocketAddr),
    FailedConnection(SocketAddr, String),
    HandshakeFailed(SocketAddr, String),
    HandshakeComplete(SocketAddr, String, i32), // addr, agent, height
    PeerDiscovered(SocketAddr),
    PingReceived(SocketAddr),
    ConnectionClosed(SocketAddr),

    // DNS
    DnsResolved(String, usize), // host, count
    DnsFailed(String, String),  // host, error

    // Database
    PeerSaved(SocketAddr),
    PeerUpdated(SocketAddr, PeerStatus),
    DatabaseOpened(usize), // peer count on open

    // Generic
    Custom(String),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Connected(addr)                    => write!(f, "connected to {}", addr),
            Event::FailedConnection(addr, err)        => write!(f, "failed to connect to {}: {}", addr, err),
            Event::HandshakeFailed(addr, err)         => write!(f, "handshake failed with {}: {}", addr, err),
            Event::HandshakeComplete(addr, agent, h)  => write!(f, "handshake complete with {} agent='{}' height={}", addr, agent, h),
            Event::PeerDiscovered(addr)               => write!(f, "discovered peer {}", addr),
            Event::PingReceived(addr)                 => write!(f, "ping from {}", addr),
            Event::ConnectionClosed(addr)             => write!(f, "connection closed by {}", addr),
            Event::DnsResolved(host, n)               => write!(f, "resolved {} address(es) for {}", n, host),
            Event::DnsFailed(host, err)               => write!(f, "DNS query failed for {}: {}", host, err),
            Event::PeerSaved(addr)                    => write!(f, "saved peer {}", addr),
            Event::PeerUpdated(addr, status)          => write!(f, "updated peer {} status={:?}", addr, status),
            Event::DatabaseOpened(n)                  => write!(f, "opened database with {} known peer(s)", n),
            Event::Custom(msg)                        => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub subsystem: Subsystem,
    pub event: Event,
    pub timestamp: u64,
}

impl LogMessage {
    pub fn new(level: LogLevel, subsystem: Subsystem, event: Event) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self { level, subsystem, event, timestamp }
    }
}
