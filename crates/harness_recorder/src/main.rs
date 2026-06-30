// Forward-declared public API surface (L5-200 integration pending).
#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;

mod cli;
mod media;
mod pty;
mod script;

use cli::Commands;

#[derive(Parser)]
#[command(name = "kla")]
#[command(about = "KLA - Kommand Line Automation")]
#[command(version = "0.1.0")]
#[command(author = "KLA Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Bootstrap structured tracing (RUST_LOG controls level; defaults to "warn").
    // env_logger is retained for log:: macros emitted by transitive deps.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();
    env_logger::init();

    let cli = Cli::parse();

    match cli::execute_command(cli.command).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
