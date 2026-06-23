use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod script;
mod pty;
mod media;

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