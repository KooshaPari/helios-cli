// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Unified HeliosCLI binary — wires harness_queue, harness_runner,
//! harness_rollback, harness_checkpoint, and helios_config together.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

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

/// Approval policy for the agent exec loop.
#[derive(Debug, Clone, clap::ValueEnum, Default, PartialEq, Eq)]
enum ApprovalPolicy {
    /// Show the plan without executing (safest).
    #[default]
    Suggest,
    /// Auto-apply file edits, but ask before shell commands.
    AutoEdit,
    /// Execute everything without asking.
    FullAuto,
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

        /// Enable sandbox restrictions (restrict working directory, validate command safety)
        #[arg(long)]
        sandbox: bool,
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

    /// Ask an AI question using an OpenAI-compatible API
    Ask {
        /// The question/prompt to send
        prompt: String,

        /// Provider URL (overrides HELIOS_AI_BASE_URL env)
        #[arg(short, long)]
        url: Option<String>,

        /// API key (overrides HELIOS_AI_API_KEY env)
        #[arg(short = 'k', long)]
        api_key: Option<String>,

        /// Model name (overrides HELIOS_AI_MODEL env)
        #[arg(short, long)]
        model: Option<String>,

        /// System prompt
        #[arg(short, long)]
        system: Option<String>,

        /// Enable interactive multi-turn chat mode
        #[arg(long)]
        chat: bool,

        /// Enable SSE streaming (tokens print as they arrive)
        #[arg(long)]
        stream: bool,
    },

    /// Execute an agent loop: send a prompt to the AI and display the response.
    ///
    /// This is the entry point for turning helios from an infrastructure CLI
    /// into a working agent. The response is parsed and printed (file-editing
    /// integration will come in a later phase).
    Exec {
        /// The task prompt to send to the AI agent
        prompt: String,

        /// Provider URL (overrides HELIOS_AI_BASE_URL env)
        #[arg(short, long)]
        url: Option<String>,

        /// API key (overrides HELIOS_AI_API_KEY env)
        #[arg(short = 'k', long)]
        api_key: Option<String>,

        /// Model name (overrides HELIOS_AI_MODEL env)
        #[arg(short, long)]
        model: Option<String>,

        /// Approval policy: suggest (plan only), auto-edit (apply file edits), full-auto (execute all)
        #[arg(long, value_enum, default_value_t)]
        approval: ApprovalPolicy,

        /// Maximum number of agent iterations (default: 10)
        #[arg(long, default_value = "10")]
        max_iterations: u32,

        /// Budget ceiling in USD (default: 1.00)
        #[arg(long, default_value = "1.0")]
        budget: f64,

        /// Cost per input token in USD (default: $30/M = 0.000030)
        #[arg(long, default_value = "0.000030")]
        cost_per_input_tokens: f64,

        /// Cost per output token in USD (default: $60/M = 0.000060)
        #[arg(long, default_value = "0.000060")]
        cost_per_output_tokens: f64,
    },

    /// Resume a previous session.
    Resume {
        /// Resume the most recent session
        #[arg(long)]
        last: bool,

        /// Resume a specific session by UUID
        #[arg(short, long, conflicts_with = "last")]
        session_id: Option<String>,
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
        Commands::Run { command, dir, timeout, shell, sandbox } => {
            cmd_run(command, dir, timeout, shell, sandbox).await
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
        Commands::Ask { prompt, url, api_key, model, system, chat, stream } => {
            cmd_ask(prompt, url, api_key, model, system, chat, stream).await
        }
        Commands::Exec { prompt, url, api_key, model, approval, max_iterations, budget, cost_per_input_tokens, cost_per_output_tokens } => {
            cmd_exec(prompt, url, api_key, model, approval, max_iterations, budget, cost_per_input_tokens, cost_per_output_tokens).await
        }
        Commands::Resume { last, session_id } => {
            cmd_resume(last, session_id)
        }
    }
}

/// Run a command through the harness runner
async fn cmd_run(command: String, dir: Option<PathBuf>, timeout: u64, shell: bool, sandbox: bool) -> Result<()> {
    use harness_runner::{RunnerConfig, Runner};

    // Sandbox mode: enable OS-level sandboxing and validate command safety
    if sandbox {
        println!("[helios] Sandbox mode enabled — enabling OS-level sandboxing…");

        // Enable real OS-level sandboxing (Landlock on Linux, guidance on macOS/Windows)
        helios_sandbox::enable_sandbox();

        // Validate command doesn't contain dangerous operations
        let dangerous = ["rm -rf /", "mkfs", "dd if=", "> /dev/", "chmod 777 /", "wget", "curl | sh", "eval ", "exec "];
        let cmd_lower = command.to_lowercase();
        for pattern in &dangerous {
            if cmd_lower.contains(pattern) {
                anyhow::bail!("[helios] Sandbox: command rejected — contains dangerous pattern: '{}'", pattern);
            }
        }

        // Restrict working directory to current dir or specified dir
        if dir.is_none() {
            println!("[helios] Sandbox: restricting working directory to current dir");
        }

        if helios_sandbox::is_sandboxed() {
            println!("[helios] Sandbox: filesystem access is now restricted (Landlock active)");
        }
    }

    println!("[helios] Running: {}", command);
    if let Some(ref d) = dir {
        println!("[helios] Working dir: {}", d.display());
    }
    println!("[helios] Timeout: {}s, Shell: {}, Sandbox: {}", timeout, shell, sandbox);

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
    println!("  helios exec <prompt>          Execute an agent task via AI");
    println!("  helios resume --last           Resume the most recent session");

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

/// Ask an AI question using an OpenAI-compatible API
async fn cmd_ask(
    prompt: String,
    url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    system: Option<String>,
    chat: bool,
    stream: bool,
) -> Result<()> {
    use helios_ai::{AiClient, ChatSession, ProviderConfig};

    // Resolve config from args > env > defaults
    let base_url = url
        .or_else(|| std::env::var("HELIOS_AI_BASE_URL").ok())
        .unwrap_or_else(|| "http://localhost:11434/v1".into());

    let api_key_val = api_key
        .or_else(|| std::env::var("HELIOS_AI_API_KEY").ok())
        .unwrap_or_default();

    let model_val = model
        .or_else(|| std::env::var("HELIOS_AI_MODEL").ok())
        .unwrap_or_else(|| "gpt-4o".into());

    let config = ProviderConfig {
        base_url,
        api_key: api_key_val,
        model: model_val,
        timeout_secs: 120,
    };

    if chat {
        // Interactive multi-turn chat mode
        let mut session = ChatSession::new(config, system.as_deref())
            .context("Failed to create chat session")?;

        println!("[helios] Chat mode (model: {}). Type 'exit' to quit, 'clear' to reset history.", session.client().config().model);

        // Send the initial prompt
        let response = session.send(&prompt).await?;
        println!("\n{response}");

        // Interactive loop
        let stdin = std::io::stdin();
        loop {
            print!("\n> ");
            std::io::Write::flush(&mut std::io::stdout()).ok();

            let mut input = String::new();
            if stdin.read_line(&mut input).is_err() || input.trim().is_empty() {
                break;
            }

            let input = input.trim().to_string();
            if input == "exit" || input == "quit" {
                println!("[helios] Chat ended. {} messages in history.", session.history().len());
                break;
            }
            if input == "clear" {
                session.clear();
                println!("[helios] History cleared.");
                continue;
            }

            match session.send(&input).await {
                Ok(response) => println!("\n{response}"),
                Err(e) => eprintln!("[helios] Error: {e}"),
            }
        }
        Ok(())
    } else if stream {
        // SSE streaming mode
        let client = AiClient::new(config)
            .context("Failed to create AI client")?;

        println!("[helios] Streaming from {}...", client.config().model);

        let messages = if let Some(sys) = system {
            vec![helios_ai::Message::system(sys), helios_ai::Message::user(&prompt)]
        } else {
            vec![helios_ai::Message::user(&prompt)]
        };

        let mut rx = client.stream_chat(&messages, None, None).await
            .context("Failed to start streaming")?;

        let mut full_response = String::new();
        while let Some(token) = rx.recv().await {
            print!("{token}");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            full_response.push_str(&token);
        }
        println!();

        if !full_response.is_empty() {
            info!(tokens = full_response.len(), "Streaming complete");
        }
        Ok(())
    } else {
        // Single-turn mode (existing behavior)
        let client = AiClient::new(config)
            .context("Failed to create AI client")?;

        println!("[helios] Asking AI (model: {})...", client.config().model);

        let response = if let Some(sys) = system {
            client.complete_with_system(&sys, &prompt).await
        } else {
            client.complete(&prompt).await
        };

        match response {
            Ok(text) => {
                println!("\n{text}");
                Ok(())
            }
            Err(e) => {
                eprintln!("[helios] AI request failed: {e}");
                anyhow::bail!("AI request failed: {e}")
            }
        }
    }
}

/// Default system prompt for the agent loop.
/// This frames helios as an autonomous agent that can plan and execute tasks.
const AGENT_SYSTEM_PROMPT: &str = concat!(
    "You are Helios, an AI-powered software engineering assistant.\n",
    "You help users with programming tasks, file operations, and\n",
    "software development processes. Be concise and actionable.\n",
    "\n",
    "Available tools:\n",
    "- read_file(path): Read a file's contents.\n",
    "- write_file(path, content): Write content to a file.\n",
    "- edit_file(path, search, replace, replace_all): Search-and-replace in a file.\n",
    "\n",
    "When you need to edit files, describe the exact tool calls you would make.\n",
    "Future versions will execute them automatically.",
);

/// Execute an agent loop: send a prompt to the AI, parse and print the response.
///
/// This is the working-agent entry point. Currently a stub that sends the
/// prompt to the configured AI provider and prints the result. Future phases
/// will add tool-calling, file editing, and multi-step planning.
///
/// The `approval` flag controls what the agent is allowed to do:
/// - `suggest`: Show the plan without executing anything.
/// - `auto-edit`: Auto-apply file edits, but ask before shell commands.
/// - `full-auto`: Execute everything without asking.
#[allow(clippy::too_many_arguments)] // CLI handler: fixed positional args from clap
async fn cmd_exec(
    prompt: String,
    url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    approval: ApprovalPolicy,
    max_iterations: u32,
    budget: f64,
    cost_per_input_tokens: f64,
    cost_per_output_tokens: f64,
) -> Result<()> {
    use helios_ai::{AiClient, CostTracker, ProviderConfig};

    // Resolve config from args > env > defaults (same pattern as cmd_ask)
    let base_url = url
        .or_else(|| std::env::var("HELIOS_AI_BASE_URL").ok())
        .unwrap_or_else(|| "http://localhost:11434/v1".into());

    let api_key_val = api_key
        .or_else(|| std::env::var("HELIOS_AI_API_KEY").ok())
        .unwrap_or_default();

    let model_val = model
        .or_else(|| std::env::var("HELIOS_AI_MODEL").ok())
        .unwrap_or_else(|| "gpt-4o".into());

    let config = ProviderConfig {
        base_url,
        api_key: api_key_val,
        model: model_val,
        timeout_secs: 120,
    };

    let client = AiClient::new(config)
        .context("Failed to create AI client for exec")?;

    let mut cost_tracker = CostTracker::new(cost_per_input_tokens, cost_per_output_tokens, budget);

    // Build an augmented system prompt based on approval policy
    let approval_desc = match approval {
        ApprovalPolicy::Suggest => {
            "You are in SUGGEST mode. Describe what you would do, but DO NOT \
             execute any commands or write any files. Just present the plan."
        }
        ApprovalPolicy::AutoEdit => {
            "You are in AUTO-EDIT mode. You may propose file edits. File edits \
             will be applied automatically. Do NOT propose shell commands."
        }
        ApprovalPolicy::FullAuto => {
            "You are in FULL-AUTO mode. Execute all actions including file edits \
             and shell commands without hesitation."
        }
    };

    let system_prompt = format!("{AGENT_SYSTEM_PROMPT}\n\nApproval policy: {approval_desc}");

    println!("[helios:exec] Sending task to {}...", client.config().model);
    println!("[helios:exec] Approval: {:?}, Max iterations: {}", approval, max_iterations);
    println!("[helios:exec] Budget: ${:.2}, Input: ${}/tok, Output: ${}/tok", budget, cost_per_input_tokens, cost_per_output_tokens);
    println!("[helios:exec] Prompt: {}", prompt);

    // Agent loop: iterate up to max_iterations, sending the prompt and printing responses.
    // Future phases will parse tool calls from the response and dispatch them.
    let mut current_prompt = prompt;

    for iteration in 1..=max_iterations {
        println!("\n--- Iteration {iteration}/{max_iterations} ---\n");

        let response = client
            .complete_with_system(&system_prompt, &current_prompt)
            .await
            .context("AI request failed during exec")?;

        println!("{response}");

        // Record token usage from the response.
        // The API doesn't return usage in complete(), so we estimate from response length.
        // When the API returns usage via chat(), it will be more accurate.
        let input_estimate = system_prompt.len() as u64 / 4 + current_prompt.len() as u64 / 4;
        let output_estimate = response.len() as u64 / 4;
        cost_tracker.record_usage(input_estimate, output_estimate);
        println!("[helios:exec] {}", cost_tracker.usage_summary());

        // Check budget
        if cost_tracker.is_over_budget() {
            println!("\n[helios:exec] Budget exceeded (${:.2}). Stopping agent loop.", budget);
            break;
        }

        // In suggest mode, stop after the first response (plan only)
        if approval == ApprovalPolicy::Suggest {
            println!("\n[suggest mode] Plan displayed. No actions taken.");
            break;
        }

        // In future phases, we would parse tool calls here and dispatch them.
        // For now, if the response doesn't contain any tool call markers, we're done.
        if !response.contains("read_file")
            && !response.contains("write_file")
            && !response.contains("edit_file")
        {
            println!("\n--- No tool calls detected. Agent loop complete. ---");
            break;
        }

        // Feed the response back as context for the next iteration
        current_prompt =
            "The previous response contained tool call descriptions. \n\
             In future versions these will be executed automatically. \n\
             For now, respond with the final result."
                .to_string();
    }

    println!("\n[helios:exec] Session complete. {}", cost_tracker.usage_summary());

    Ok(())
}

/// Resume a previous chat session.
///
/// With `--last`, loads the most recently saved session. With `--session-id <id>`,
/// loads a specific session. Prints the session metadata and conversation history.
fn cmd_resume(last: bool, session_id: Option<String>) -> Result<()> {
    use helios_ai::{load_last_session, load_session, session_from_record, session_path};

    if !last && session_id.is_none() {
        anyhow::bail!("Specify --last to resume the most recent session, or --session-id <uuid>");
    }

    let record = if last {
        println!("[helios:resume] Loading most recent session...");
        match load_last_session()? {
            Some(r) => r,
            None => {
                anyhow::bail!("No saved sessions found in ~/.helios/sessions/");
            }
        }
    } else {
        let id_str = session_id.unwrap();
        let id: uuid::Uuid = id_str.parse()
            .context("Invalid session UUID")?;
        let path = session_path(&id)?;
        println!("[helios:resume] Loading session {}...", id);
        load_session(&path)?
    };

    println!("[helios:resume] Session ID:    {}", record.id);
    println!("[helios:resume] Created:       {}", record.created_at);
    println!("[helios:resume] Last saved:    {}", record.saved_at);
    println!("[helios:resume] Model:         {}", record.config.model);
    println!("[helios:resume] Messages:      {}", record.messages.len());
    println!();

    let _session = session_from_record(&record)
        .context("Failed to reconstruct session")?;

    println!("[helios:resume] Session restored successfully.");
    println!("[helios:resume] Use 'helios ask --chat' or 'helios exec' to continue the conversation.");

    Ok(())
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

    /// Test that the CLI parser can handle the ask subcommand.
    #[test]
    fn test_cli_parser_ask_subcommand() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "ask", "What is Rust?",
            "--url", "http://localhost:11434/v1",
            "--model", "llama3",
            "--system", "You are a helpful assistant",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios ask' should succeed: {cli:?}");
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Ask { .. }));
    }

    /// Test that the CLI parser handles the run --sandbox flag.
    #[test]
    fn test_cli_parser_run_sandbox_flag() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "run", "--sandbox", "--shell", "echo hello"]);
        assert!(cli.is_ok(), "CLI parsing 'helios run --sandbox' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Run { command, shell, sandbox, .. } => {
                assert_eq!(command, "echo hello");
                assert!(shell);
                assert!(sandbox);
            }
            _ => panic!("Expected Run command"),
        }
    }

    /// Test that --sandbox defaults to false when not specified.
    #[test]
    fn test_cli_parser_run_no_sandbox() {
        let cli = Cli::try_parse_from(["helios", "run", "echo hello"]).unwrap();
        match cli.command {
            Commands::Run { sandbox, .. } => assert!(!sandbox),
            _ => panic!("Expected Run command"),
        }
    }

    /// Test that the exec subcommand parses correctly.
    #[test]
    fn test_cli_parser_exec_subcommand() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "fix the failing tests",
            "--url", "http://localhost:11434/v1",
            "--model", "llama3",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios exec' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Exec { prompt, url, model, approval, max_iterations, .. } => {
                assert_eq!(prompt, "fix the failing tests");
                assert_eq!(url.as_deref(), Some("http://localhost:11434/v1"));
                assert_eq!(model.as_deref(), Some("llama3"));
                assert_eq!(approval, ApprovalPolicy::Suggest, "default should be suggest");
                assert_eq!(max_iterations, 10, "default max_iterations should be 10");
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that the exec subcommand requires a prompt.
    #[test]
    fn test_cli_parser_exec_requires_prompt() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "exec"]);
        assert!(cli.is_err(), "exec without prompt should fail");
    }

    /// Test that the agent system prompt is non-empty and contains key markers.
    #[test]
    fn test_agent_system_prompt_is_well_formed() {
        assert!(!AGENT_SYSTEM_PROMPT.is_empty());
        assert!(AGENT_SYSTEM_PROMPT.contains("Helios"));
        assert!(AGENT_SYSTEM_PROMPT.contains("agent") || AGENT_SYSTEM_PROMPT.contains("assistant"));
    }

    /// Test that the resume subcommand parses with --last.
    #[test]
    fn test_cli_parser_resume_last() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "resume", "--last"]);
        assert!(cli.is_ok(), "CLI parsing 'helios resume --last' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Resume { last, session_id } => {
                assert!(last);
                assert!(session_id.is_none());
            }
            _ => panic!("Expected Resume command"),
        }
    }

    /// Test that the resume subcommand parses with --session-id.
    #[test]
    fn test_cli_parser_resume_session_id() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "resume", "--session-id", "550e8400-e29b-41d4-a716-446655440000",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios resume --session-id' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Resume { last, session_id } => {
                assert!(!last);
                assert_eq!(session_id.as_deref(), Some("550e8400-e29b-41d4-a716-446655440000"));
            }
            _ => panic!("Expected Resume command"),
        }
    }

    /// Test that resume without flags parses successfully (validation is at runtime).
    #[test]
    fn test_cli_parser_resume_requires_flag() {
        use clap::Parser;
        // clap allows bare 'resume'; runtime validation in cmd_resume rejects it
        let cli = Cli::try_parse_from(["helios", "resume"]);
        assert!(cli.is_ok(), "resume without flags should parse (validated at runtime): {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Resume { last, session_id } => {
                assert!(!last);
                assert!(session_id.is_none());
            }
            _ => panic!("Expected Resume command"),
        }
    }

    /// Test that --last and --session-id conflict.
    #[test]
    fn test_cli_parser_resume_conflicting_flags() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "resume", "--last", "--session-id", "some-uuid",
        ]);
        assert!(cli.is_err(), "--last and --session-id should conflict");
    }

    /// Test that the exec subcommand parses --approval auto-edit.
    #[test]
    fn test_cli_parser_exec_approval_auto_edit() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "refactor this",
            "--approval", "auto-edit",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios exec --approval auto-edit' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Exec { approval, .. } => {
                assert_eq!(approval, ApprovalPolicy::AutoEdit);
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that the exec subcommand parses --approval full-auto.
    #[test]
    fn test_cli_parser_exec_approval_full_auto() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "deploy everything",
            "--approval", "full-auto",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios exec --approval full-auto' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Exec { approval, .. } => {
                assert_eq!(approval, ApprovalPolicy::FullAuto);
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that --max-iterations is parsed correctly.
    #[test]
    fn test_cli_parser_exec_max_iterations() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "build the project",
            "--max-iterations", "5",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios exec --max-iterations 5' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Exec { max_iterations, .. } => {
                assert_eq!(max_iterations, 5);
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that --approval defaults to suggest.
    #[test]
    fn test_cli_parser_exec_approval_default() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "exec", "hello"]).unwrap();
        match cli.command {
            Commands::Exec { approval, .. } => {
                assert_eq!(approval, ApprovalPolicy::Suggest, "default approval should be suggest");
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that budget flags are parsed and have correct defaults.
    #[test]
    fn test_cli_parser_exec_budget_flags() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "do something",
            "--budget", "5.0",
            "--cost-per-input-tokens", "0.000010",
            "--cost-per-output-tokens", "0.000020",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios exec' with budget flags should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Exec { budget, cost_per_input_tokens, cost_per_output_tokens, .. } => {
                assert!((budget - 5.0).abs() < f64::EPSILON);
                assert!((cost_per_input_tokens - 0.000010).abs() < f64::EPSILON);
                assert!((cost_per_output_tokens - 0.000020).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that budget flags have sensible defaults.
    #[test]
    fn test_cli_parser_exec_budget_defaults() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["helios", "exec", "hello"]).unwrap();
        match cli.command {
            Commands::Exec { budget, cost_per_input_tokens, cost_per_output_tokens, .. } => {
                assert!((budget - 1.0).abs() < f64::EPSILON, "default budget should be 1.0");
                assert!((cost_per_input_tokens - 0.000030).abs() < f64::EPSILON, "default input cost should be $30/M");
                assert!((cost_per_output_tokens - 0.000060).abs() < f64::EPSILON, "default output cost should be $60/M");
            }
            _ => panic!("Expected Exec command"),
        }
    }

    /// Test that --approval rejects invalid values.
    #[test]
    fn test_cli_parser_exec_approval_invalid() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "exec", "hello",
            "--approval", "invalid-mode",
        ]);
        assert!(cli.is_err(), "invalid approval value should be rejected");
    }

    /// Test that the ask subcommand parses --stream.
    #[test]
    fn test_cli_parser_ask_stream_flag() {
        use clap::Parser;
        let cli = Cli::try_parse_from([
            "helios", "ask", "What is Rust?",
            "--stream",
        ]);
        assert!(cli.is_ok(), "CLI parsing 'helios ask --stream' should succeed: {cli:?}");
        let cli = cli.unwrap();
        match cli.command {
            Commands::Ask { stream, .. } => {
                assert!(stream);
            }
            _ => panic!("Expected Ask command"),
        }
    }

    /// Test that --stream defaults to false.
    #[test]
    fn test_cli_parser_ask_no_stream() {
        let cli = Cli::try_parse_from(["helios", "ask", "hello"]).unwrap();
        match cli.command {
            Commands::Ask { stream, .. } => {
                assert!(!stream, "--stream should default to false");
            }
            _ => panic!("Expected Ask command"),
        }
    }
}
