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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn addr(ip: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info  < LogLevel::Warn);
        assert!(LogLevel::Warn  < LogLevel::Error);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(),  "INFO ");
        assert_eq!(LogLevel::Warn.to_string(),  "WARN ");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
    }

    #[test]
    fn test_event_display_connected() {
        let a = addr([1, 2, 3, 4], 8333);
        let msg = Event::Connected(a).to_string();
        assert!(msg.contains("1.2.3.4:8333"));
        assert!(msg.contains("connected"));
    }

    #[test]
    fn test_event_display_failed_connection() {
        let a = addr([5, 6, 7, 8], 8333);
        let msg = Event::FailedConnection(a, "timeout".into()).to_string();
        assert!(msg.contains("5.6.7.8:8333"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_event_display_dns_resolved() {
        let msg = Event::DnsResolved("dnsseed.local".into(), 42).to_string();
        assert!(msg.contains("42"));
        assert!(msg.contains("dnsseed.local"));
    }

    #[test]
    fn test_event_display_custom() {
        let msg = Event::Custom("hello world".into()).to_string();
        assert_eq!(msg, "hello world");
    }

    #[test]
    fn test_log_message_timestamp_is_recent() {
        let msg = LogMessage::new(
            LogLevel::Info,
            Subsystem::Network,
            Event::Custom("test".into()),
        );
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(msg.timestamp <= now);
        assert!(msg.timestamp >= now - 2);
    }
}
