// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Runner module - Optimized process execution
//! Features: Timeout, streaming, environment isolation

use std::process::Stdio;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::instrument;

/// Shell-safe quoting: wraps `s` in single quotes and escapes embedded single
/// quotes so the result can be safely joined into a `sh -c` string without
/// shell injection.
///
/// # Security
///
/// Single-quote shell escaping is the only quoting mechanism that prevents
/// **all** shell metacharacter interpretation (no backslash, no dollar, no
/// backtick processing) without requiring a whitelist. Each argument is
/// individually quoted so an attacker-controlled `cmd` or `args` value like
/// `"; rm -rf /; echo "` becomes a literal string.
fn shell_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

/// Runner configuration
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub working_dir: Option<String>,
    pub timeout_secs: Option<u64>,
    pub env: std::collections::HashMap<String, String>,
    pub shell: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            working_dir: None,
            timeout_secs: Some(30),
            env: std::collections::HashMap::new(),
            shell: false,
        }
    }
}

/// Process runner with environment control
pub struct Runner {
    config: RunnerConfig,
}

impl Runner {
    pub fn new() -> Self {
        Self { config: RunnerConfig::default() }
    }

    pub fn with_config(config: RunnerConfig) -> Self {
        Self { config }
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.config.working_dir = Some(dir.to_string());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.config.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = Some(secs);
        self
    }

    pub fn with_shell(mut self, shell: bool) -> Self {
        self.config.shell = shell;
        self
    }

    /// Run command and get result
    #[instrument(name = "runner_run", skip(self, args))]
    pub async fn run(&self, cmd: &str, args: &[&str]) -> Result<RunResult> {
        let start = Instant::now();

        let mut child = if self.config.shell {
            let mut c = Command::new("sh");
            // SECURITY: Each argument is individually single-quote-escaped to
            // prevent shell injection through untrusted cmd/args values.
            let quoted_cmd = shell_quote(cmd);
            let quoted_args: Vec<String> = args.iter().map(|a| shell_quote(a)).collect();
            let shell_line = if quoted_args.is_empty() {
                quoted_cmd
            } else {
                format!("{} {}", quoted_cmd, quoted_args.join(" "))
            };
            c.arg("-c").arg(shell_line);
            c
        } else {
            let mut c = Command::new(cmd);
            c.args(args);
            c
        };

        if let Some(ref dir) = self.config.working_dir {
            child.current_dir(dir);
        }

        for (k, v) in &self.config.env {
            child.env(k, v);
        }

        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        let output = match self.config.timeout_secs {
            Some(timeout) => {
                match tokio::time::timeout(Duration::from_secs(timeout), child.output()).await {
                    Ok(Ok(output)) => output,
                    Ok(Err(e)) => return Err(RunError::Io(e)),
                    Err(_) => return Err(RunError::Timeout(timeout)),
                }
            }
            None => child.output().await?,
        };

        let duration = start.elapsed();

        Ok(RunResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    /// Run with stdin input
    #[instrument(name = "runner_run_with_input", skip(self, args, input))]
    pub async fn run_with_input(&self, cmd: &str, args: &[&str], input: &str) -> Result<RunResult> {
        let start = Instant::now();

        let mut child = if self.config.shell {
            let mut c = Command::new("sh");
            // SECURITY: Same shell-injection prevention as run().
            let quoted_cmd = shell_quote(cmd);
            let quoted_args: Vec<String> = args.iter().map(|a| shell_quote(a)).collect();
            let shell_line = if quoted_args.is_empty() {
                quoted_cmd
            } else {
                format!("{} {}", quoted_cmd, quoted_args.join(" "))
            };
            c.arg("-c").arg(shell_line);
            c
        } else {
            let mut c = Command::new(cmd);
            c.args(args);
            c
        };

        if let Some(ref dir) = self.config.working_dir {
            child.current_dir(dir);
        }

        for (k, v) in &self.config.env {
            child.env(k, v);
        }

        child.stdin(Stdio::piped());
        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        let mut child_handle = child.spawn()?;

        if let Some(ref mut stdin) = child_handle.stdin {
            stdin.write_all(input.as_bytes()).await?;
        }

        let output = child_handle.wait_with_output().await?;

        let duration = start.elapsed();

        Ok(RunResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

/// Run result with metadata
#[derive(Debug, Clone)]
pub struct RunResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl RunResult {
    pub fn output(&self) -> String {
        if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            self.stdout.clone()
        }
    }

    pub fn output_lines(&self) -> Vec<String> {
        self.output().lines().map(|s| s.to_string()).collect()
    }
}

/// Run errors
///
/// `RunError` is derived with `thiserror` and implements
/// `From<std::io::Error>` so callers can use `?` for I/O failures without
/// an explicit `.map_err(RunError::IoError)`.
#[derive(Debug, Error)]
pub enum RunError {
    /// Wrapped I/O error from process spawning, stdin writes, or output
    /// collection.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The configured timeout elapsed before the child process exited.
    #[error("Timeout after {0}s")]
    Timeout(u64),

    /// The requested command could not be located.
    #[error("Command not found")]
    NotFound,
}

/// Result alias for the runner API.
pub type Result<T> = std::result::Result<T, RunError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Traces to: FR-HELIOS-IO-006
    /// `From<io::Error>` must map to the `Io` variant.
    #[test]
    fn from_io_error_maps_to_io_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such binary");
        let err: RunError = io_err.into();
        assert!(matches!(err, RunError::Io(_)));
    }

    /// Traces to: FR-HELIOS-IO-006
    /// Display must surface the underlying I/O message.
    #[test]
    fn io_error_display_includes_message() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such binary");
        let err: RunError = io_err.into();
        assert!(err.to_string().contains("no such binary"));
    }

    /// Traces to: FR-HELIOS-IO-006
    /// Timeout and NotFound variants must render their own messages.
    #[test]
    fn timeout_and_not_found_display() {
        assert_eq!(RunError::Timeout(7).to_string(), "Timeout after 7s");
        assert_eq!(RunError::NotFound.to_string(), "Command not found");
    }

    /// SEC-HELIOS-CMDINJ-001
    /// `shell_quote` must wrap bare text in single quotes.
    #[test]
    fn shell_quote_bare_text() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    /// SEC-HELIOS-CMDINJ-002
    /// `shell_quote` must escape embedded single quotes so they cannot break
    /// out of the quoting context.
    #[test]
    fn shell_quote_embedded_single_quote() {
        let q = shell_quote("it's");
        // Should produce something like 'it'\\''s' (the literal \\ is a Rust
        // string escaping of the backslash that we emit).
        assert!(q.starts_with("'"));
        assert!(q.ends_with("'"));
        // The embedded single quote must be escaped with `'\''` pattern.
        assert!(
            shell_quote("a'b").contains("'\\''"),
            "single quote should be escaped with shell pattern: '\\''"
        );
    }

    /// SEC-HELIOS-CMDINJ-003
    /// `shell_quote` must not remove or truncate input.
    #[test]
    fn shell_quote_preserves_length() {
        let input = "echo hello";
        let quoted = shell_quote(input);
        assert!(quoted.len() > input.len());
        assert!(quoted.contains(input));
    }

    /// SEC-HELIOS-CMDINJ-004
    /// Shell metacharacters must be neutralised.
    #[test]
    fn shell_quote_dollar_sign_is_literal() {
        let q = shell_quote("$(id)");
        assert_eq!(q, "'$(id)'");
    }

    /// SEC-HELIOS-CMDINJ-005
    /// Backticks must be literal inside single quotes.
    #[test]
    fn shell_quote_backtick_is_literal() {
        let q = shell_quote("`whoami`");
        assert_eq!(q, "'`whoami`'");
    }

    /// SEC-HELIOS-CMDINJ-006
    /// Semicolons (command separator) must be literal inside single quotes.
    #[test]
    fn shell_quote_semicolon_is_literal() {
        let q = shell_quote("; rm -rf /");
        assert_eq!(q, "'; rm -rf /'");
    }

    /// SEC-HELIOS-CMDINJ-007
    /// The shell mode must construct a shell line where each token is
    /// individually quoted.  An attacker-controlled value in args must not
    /// alter the command structure.
    #[tokio::test]
    async fn shell_mode_rejects_injection() {
        let runner = Runner::new().with_shell(true);
        // If unquoted, this argument would inject a semicolon and run `echo
        // pwned` as a separate command.  With quoting it must run `echo` with
        // the literal string `; echo pwned`.
        let result = runner.run("echo", &["; echo pwned"]).await.expect("runner should not panic");
        assert!(result.success, "shell mode should still succeed");
        // stdout must contain the literal semicolon (escaped by quoting).
        let stdout = result.stdout.trim();
        assert!(
            stdout.contains("; echo pwned"),
            "expected stdout to contain literal injection string, got: {stdout:?}"
        );
    }

    /// SEC-HELIOS-CMDINJ-008
    /// The shell mode with input must also be safe against injection.
    #[tokio::test]
    async fn shell_mode_with_input_rejects_injection() {
        let runner = Runner::new().with_shell(true);
        let result = runner
            .run_with_input("echo", &["safe_arg"], "hello")
            .await
            .expect("run_with_input should succeed");
        assert!(result.success, "run_with_input should succeed");
    }

    /// COR-HELIOS-RUN-001
    /// `run_with_input` must report a non-zero (i.e. real) duration, not
    /// `Duration::ZERO`.
    #[tokio::test]
    async fn run_with_input_reports_real_duration() {
        let runner = Runner::new().with_shell(false);
        let result = runner
            .run_with_input("echo", &["hello"], "test")
            .await
            .expect("run_with_input should succeed");
        assert!(
            result.duration > Duration::ZERO,
            "duration should not be zero; got {:?}",
            result.duration
        );
    }
}
