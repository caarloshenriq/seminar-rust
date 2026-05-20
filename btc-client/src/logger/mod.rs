pub mod event;

use tokio::sync::mpsc;
use event::{Event, LogLevel, LogMessage, Subsystem};

#[derive(Clone)]
pub struct Logger {
    sender: mpsc::Sender<LogMessage>,
}

impl Logger {
    pub fn new(sender: mpsc::Sender<LogMessage>) -> Self {
        Self { sender }
    }

    pub fn log(&self, level: LogLevel, subsystem: Subsystem, event: Event) {
        let msg = LogMessage::new(level, subsystem, event);
        // fire and forget — never block the caller
        let _ = self.sender.try_send(msg);
    }

    // convenience methods
    pub fn info(&self, subsystem: Subsystem, event: Event) {
        self.log(LogLevel::Info, subsystem, event);
    }

    pub fn warn(&self, subsystem: Subsystem, event: Event) {
        self.log(LogLevel::Warn, subsystem, event);
    }

    pub fn error(&self, subsystem: Subsystem, event: Event) {
        self.log(LogLevel::Error, subsystem, event);
    }

    pub fn debug(&self, subsystem: Subsystem, event: Event) {
        self.log(LogLevel::Debug, subsystem, event);
    }

    pub fn trace(&self, subsystem: Subsystem, event: Event) {
        self.log(LogLevel::Trace, subsystem, event);
    }
}

/// Spawn the logger task. Returns a Logger handle and a JoinHandle.
pub fn spawn(min_level: LogLevel) -> (Logger, tokio::task::JoinHandle<()>) {
    let (tx, mut rx) = mpsc::channel::<LogMessage>(256);
    let logger = Logger::new(tx);

    let handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if msg.level >= min_level {
                println!(
                    "[{ts}] {level} {subsystem} — {event}",
                    ts        = msg.timestamp,
                    level     = msg.level,
                    subsystem = msg.subsystem,
                    event     = msg.event,
                );
            }
        }
    });

    (logger, handle)
}
