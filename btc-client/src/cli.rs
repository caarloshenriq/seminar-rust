use clap::Parser;

/// A minimal Bitcoin P2P crawler.
#[derive(Parser, Debug)]
#[command(name = "btc-client", about, long_about = None)]
pub struct Cli {
    /// DNS seed host to bootstrap peer discovery
    #[arg(long, default_value = "seed.bitcoin.sipa.be")]
    pub host: String,

    /// Bitcoin P2P port
    #[arg(long, default_value_t = 8333)]
    pub port: u16,

    /// DNS seeder port (for custom local seeders)
    #[arg(long, default_value_t = 53)]
    pub seeder_port: u16,

    /// Number of crawling threads (not yet implemented)
    #[arg(long, default_value_t = 1)]
    pub threads: u32,

    /// Path to log file (not yet implemented)
    #[arg(long)]
    pub logfile: Option<String>,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
}
