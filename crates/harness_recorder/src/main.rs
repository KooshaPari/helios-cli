// Forward-declared public API surface (L5-200 integration pending).
#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;

mod cli;
mod i18n;
mod media;
mod pty;
mod script;

use cli::Commands;
use i18n::I18n;

#[derive(Parser)]
#[command(name = "kla")]
#[command(about = "KLA - Kommand Line Automation")]
#[command(version = "0.1.0")]
#[command(author = "KLA Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Set the locale for the CLI (e.g. en, fr)
    #[arg(long, env = "HELIOS_LOCALE", default_value = "en")]
    locale: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Bootstrap structured tracing (RUST_LOG controls level; defaults to "warn").
    // tracing-subscriber installs the global logger bridge for log:: macros too.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let i18n = I18n::new(&cli.locale);

    match cli::execute_command(cli.command, &i18n).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", i18n.t_with("error.general", &[("error", &e.to_string())]));
            std::process::exit(1);
        }
    }
}
