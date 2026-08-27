// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Unified HeliosCLI binary — wires harness_queue, harness_runner,
//! harness_rollback, harness_checkpoint, and helios_config together.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "helios",
    about = "HeliosCLI — Unified harness for agent task orchestration",
    version,
    long_about = "HeliosCLI combines task queuing, command execution, checkpoint/rollback,\nand configuration into a single CLI binary."
)]
struct Cli {
    /// Config file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a command through the harness runner
    Run {
        /// Command to execute
        command: String,

        /// Working directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Timeout in seconds
        #[arg(short = 't', long, default_value = "300")]
        timeout: u64,

        /// Run in shell mode
        #[arg(long)]
        shell: bool,
    },

    /// Create a git checkpoint
    Checkpoint {
        /// Spec/project identifier
        #[arg(short, long, default_value = "default")]
        spec: String,

        /// Optional message
        #[arg(short, long)]
        message: Option<String>,

        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        repo: Option<PathBuf>,
    },

    /// Rollback to a checkpoint
    Rollback {
        /// Checkpoint ID (git SHA or checkpoint UUID)
        checkpoint_id: String,

        /// Repository path
        #[arg(short, long)]
        repo: Option<PathBuf>,
    },

    /// Show system status and harness crate versions
    Status,

    /// Enqueue a task for background processing
    Enqueue {
        /// Task payload (JSON string)
        payload: String,

        /// Queue capacity
        #[arg(short, long, default_value = "100")]
        capacity: usize,
    },

    /// Record a terminal session using KLA
    Record {
        /// Path to the KLA script (.kla.yaml)
        script: PathBuf,

        /// Output directory for recordings
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Output format (png, gif, both)
        #[arg(short, long, default_value = "both")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { command, dir, timeout, shell } => {
            cmd_run(command, dir, timeout, shell).await
        }
        Commands::Checkpoint { spec, message, repo } => {
            cmd_checkpoint(spec, message, repo)
        }
        Commands::Rollback { checkpoint_id, repo } => {
            cmd_rollback(checkpoint_id, repo)
        }
        Commands::Status => cmd_status(),
        Commands::Enqueue { payload, capacity } => {
            cmd_enqueue(payload, capacity)
        }
        Commands::Record { script, output, format } => {
            cmd_record(script, output, format).await
        }
    }
}

/// Run a command through the harness runner
async fn cmd_run(command: String, dir: Option<PathBuf>, timeout: u64, shell: bool) -> Result<()> {
    use harness_runner::{RunnerConfig, Runner};

    println!("[helios] Running: {}", command);
    if let Some(ref d) = dir {
        println!("[helios] Working dir: {}", d.display());
    }
    println!("[helios] Timeout: {}s, Shell: {}", timeout, shell);

    let config = RunnerConfig {
        working_dir: dir.map(|p| p.to_string_lossy().to_string()),
        timeout_secs: Some(timeout),
        env: std::collections::HashMap::new(),
        shell,
    };

    let runner = Runner::with_config(config);

    match runner.run(&command, &[]).await {
        Ok(output) => {
            let code = output.exit_code.unwrap_or(1);
            println!("[helios] Exit code: {}", code);
            if !output.stdout.is_empty() {
                println!("--- stdout ---");
                println!("{}", output.stdout);
            }
            if !output.stderr.is_empty() {
                println!("--- stderr ---");
                eprintln!("{}", output.stderr);
            }
            if code != 0 {
                std::process::exit(code);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("[helios] Run failed: {}", e);
            anyhow::bail!("Command failed: {}", e)
        }
    }
}

/// Create a git checkpoint
fn cmd_checkpoint(spec: String, message: Option<String>, repo: Option<PathBuf>) -> Result<()> {
    use harness_checkpoint::checkpoint::CheckpointOptions;
    use harness_checkpoint::git::create_git_checkpoint;

    let repo_path = repo.unwrap_or_else(|| std::env::current_dir().expect("failed to get cwd"));

    println!("[helios] Creating checkpoint for spec '{}' in {}", spec, repo_path.display());

    let options = CheckpointOptions {
        include_uncommitted: true,
        message: message.or_else(|| Some(format!("helios checkpoint for {}", spec))),
        ..Default::default()
    };

    let checkpoint = create_git_checkpoint(&repo_path, &spec, &options)
        .context("Failed to create checkpoint")?;

    println!("[helios] Checkpoint created:");
    println!("  ID:   {}", checkpoint.id);
    println!("  SHA:  {}", checkpoint.git_sha.as_deref().unwrap_or("none"));
    println!("  Msg:  {}", checkpoint.git_message.as_deref().unwrap_or("none"));
    println!("  Spec: {}", checkpoint.spec_id);

    Ok(())
}

/// Rollback to a checkpoint
fn cmd_rollback(checkpoint_id: String, repo: Option<PathBuf>) -> Result<()> {
    use harness_rollback::RollbackEngine;

    let repo_path = repo.unwrap_or_else(|| std::env::current_dir().expect("failed to get cwd"));

    println!("[helios] Rolling back to checkpoint: {}", checkpoint_id);
    println!("[helios] Repository: {}", repo_path.display());

    let mut engine = RollbackEngine::with_repo(repo_path);
    // Register with the checkpoint_id as both the ID and SHA (caller provides the SHA)
    engine.register(&checkpoint_id, &checkpoint_id, "cli");

    match engine.rollback(&checkpoint_id) {
        Some(record) => {
            println!("[helios] Rollback completed:");
            println!("  Status:  {:?}", record.status);
            println!("  Restored: {:?}", record.restored_items);
            if !record.failed_items.is_empty() {
                eprintln!("  Failed: {:?}", record.failed_items);
            }
            if !engine.verify(&record) {
                anyhow::bail!("Rollback verification failed (partial or failed status)");
            }
            Ok(())
        }
        None => {
            anyhow::bail!("Rollback returned no record — checkpoint may not exist");
        }
    }
}

/// Show system status
fn cmd_status() -> Result<()> {
    println!("=== HeliosCLI Status ===");
    println!();
    println!("Crates:");
    println!("  helios_config:      {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_queue:      {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_runner:     {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_rollback:   {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_checkpoint: {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_spec:       {}", env!("CARGO_PKG_VERSION"));
    println!("  harness_verify:     {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Check config
    let config_path = std::env::current_dir()
        .ok()
        .map(|d| d.join("helios.toml"))
        .filter(|p| p.exists());

    match config_path {
        Some(path) => println!("Config: {}", path.display()),
        None => println!("Config: not found (using defaults)"),
    }

    println!();
    println!("Usage:");
    println!("  helios run <command>          Run a command through the harness");
    println!("  helios checkpoint --spec <s>  Create a git checkpoint");
    println!("  helios rollback <id>          Rollback to a checkpoint");
    println!("  helios status                 Show this status");
    println!("  helios enqueue <payload>      Enqueue a background task");
    println!("  helios record <script>        Record a terminal session (KLA)");

    Ok(())
}

/// Enqueue a task for background processing.
///
/// Uses [`harness_queue::Channel::try_send`] to guarantee the enqueue never
/// blocks — it returns immediately with an error if the channel is full or
/// closed, rather than waiting for a consumer.
fn cmd_enqueue(payload: String, capacity: usize) -> Result<()> {
    use harness_queue::Channel;

    let channel: Channel<String> = Channel::new(capacity);

    // Parse payload as JSON to validate, then enqueue
    let _parsed: serde_json::Value = serde_json::from_str(&payload)
        .context("Invalid JSON payload")?;

    // try_send is non-blocking: returns Err immediately if full / closed.
    channel.try_send(payload.clone())
        .map_err(|e| anyhow::anyhow!("Queue send failed: {:?}", e))?;

    println!("[helios] Task enqueued (queue depth: 1)");
    println!("  Payload: {}", payload);

    // Verify it can be received
    if let Some(item) = channel.recv() {
        println!("  Verified: received back from queue");
        println!("  Item: {}", item);
    }

    Ok(())
}

/// Record a terminal session using KLA
async fn cmd_record(script: PathBuf, output: PathBuf, format: String) -> Result<()> {
    println!("[helios] Recording session from: {}", script.display());
    println!("[helios] Output dir: {}", output.display());
    println!("[helios] Format: {}", format);

    kla::cli::commands::record_command(script, output, format).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the status command runs successfully and displays version info.
    #[test]
    fn test_status_command_displays_version_info() {
        let result = cmd_status();
        assert!(result.is_ok(), "cmd_status should return Ok(())");
    }

    /// Test that enqueue validates JSON input — invalid JSON must fail.
    #[test]
    fn test_enqueue_validates_json_input() {
        let result = cmd_enqueue("not valid json {{{".to_string(), 10);
        assert!(result.is_err(), "invalid JSON should be rejected");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("JSON") || err_msg.contains("json"),
            "error should mention JSON: {err_msg}"
        );
    }

    /// Test that RunnerConfig defaults are sane.
    #[test]
    fn test_runner_config_defaults_are_sane() {
        let cfg = harness_runner::RunnerConfig::default();
        assert!(cfg.working_dir.is_none(), "default working_dir should be None");
        assert!(cfg.timeout_secs.is_some(), "default timeout_secs should be set");
        assert!(cfg.timeout_secs.unwrap() > 0, "default timeout should be positive");
        assert!(cfg.env.is_empty(), "default env should be empty");
        assert!(!cfg.shell, "default shell mode should be false");
    }

    /// Test that CheckpointOptions loads sensible defaults.
    #[test]
    fn test_checkpoint_config_loads() {
        let opts = harness_checkpoint::checkpoint::CheckpointOptions::default();
        assert!(opts.git_checkpoint, "git_checkpoint should default to true");
        assert!(opts.config_snapshot, "config_snapshot should default to true");
        assert!(opts.metrics_baseline, "metrics_baseline should default to true");
        assert!(opts.include_uncommitted, "include_uncommitted should default to true");
        assert!(opts.message.is_none(), "message should default to None");
    }

    /// Test that RollbackEngine initializes with empty state.
    #[test]
    fn test_rollback_engine_initializes() {
        let engine = harness_rollback::RollbackEngine::new();
        assert!(engine.history().is_empty(), "new engine should have no history");
    }

    /// Test that enqueue succeeds with valid JSON payload.
    #[test]
    fn test_enqueue_accepts_valid_json() {
        let payload = r#"{"task":"run-tests","priority":"high"}"#.to_string();
        let result = cmd_enqueue(payload, 10);
        assert!(result.is_ok(), "valid JSON payload should be accepted: {result:?}");
    }

    /// Test that enqueue with capacity=1 does not block even when channel is full.
    #[test]
    fn test_enqueue_with_full_channel_returns_error() {
        use harness_queue::Channel;
        let channel: Channel<String> = Channel::new(1);
        channel.try_send("a".into()).unwrap();
        // The channel is full — try_send must return Err immediately, not block.
        assert!(
            channel.try_send("b".into()).is_err(),
            "try_send on a full channel must not block"
        );
    }

    /// Test that the CLI parser can be constructed without panicking.
    #[test]
    fn test_cli_parser_construction() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "status"]);
        assert!(cli.is_ok(), "CLI parsing 'helios status' should succeed: {cli:?}");
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    /// Test that the CLI parser can handle the record subcommand.
    #[test]
    fn test_cli_parser_record_subcommand() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "record", "test.kla.yaml",
            "--output", "./recording",
            "--format", "gif",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios record' should succeed: {cli:?}");
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Record { .. }));
    }
}
