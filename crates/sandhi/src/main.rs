use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use sandhi::config::{self, Config};

/// One machine on several networks at once.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// The config file [default: $XDG_CONFIG_HOME/sandhi/config.toml]
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the daemon: every backend's dutavas, the inbound forwards and the control socket.
    Up,
    /// Connect stdin and stdout to <host>:<port> through the backend that owns <host>; for
    /// ssh's ProxyCommand: `ProxyCommand sandhi connect %h %p`.
    Connect { host: String, port: u16 },
    /// The backends, their state, addresses and forwards.
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let quiet = !matches!(cli.command, Command::Up);
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("SANDHI_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(if quiet { "warn" } else { "info" })),
        )
        .init();
    let config = Config::load(&cli.config.unwrap_or_else(config::default_path))?;
    match cli.command {
        Command::Up => sandhi::daemon::run(config).await,
        Command::Connect { host, port } => {
            let s = sandhi::client::connect(&config.socket(), &host, port).await?;
            sandhi::client::splice_stdio(s).await?;
            // Tokio's stdin reader may still be blocked in a read; leave without waiting.
            std::process::exit(0)
        }
        Command::Status => {
            print!("{}", sandhi::client::status(&config.socket()).await?);
            Ok(())
        }
    }
}
