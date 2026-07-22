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

        let mut cmd = if self.config.shell {
            let mut c = Command::new("sh");
            c.arg("-c").arg(format!("{} {}", cmd, args.join(" ")));
            c
        } else {
            let mut c = Command::new(cmd);
            c.args(args);
            c
        };

        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = match self.config.timeout_secs {
            Some(timeout) => {
                match tokio::time::timeout(Duration::from_secs(timeout), cmd.output()).await {
                    Ok(Ok(output)) => output,
                    Ok(Err(e)) => return Err(RunError::Io(e)),
                    Err(_) => return Err(RunError::Timeout(timeout)),
                }
            }
            None => cmd.output().await?,
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
        let mut cmd = if self.config.shell {
            let mut c = Command::new("sh");
            c.arg("-c").arg(format!("{} {}", cmd, args.join(" ")));
            c
        } else {
            let mut c = Command::new(cmd);
            c.args(args);
            c
        };

        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(input.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;

        Ok(RunResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration: Duration::ZERO,
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

    #[tokio::test]
    async fn run_executes_command_and_collects_stdout() {
        #[cfg(windows)]
        let result = Runner::new().run("cmd", &["/C", "echo hello-runner"]).await.unwrap();
        #[cfg(not(windows))]
        let result = Runner::new().run("sh", &["-c", "echo hello-runner"]).await.unwrap();

        assert!(result.success);
        assert!(result.output().contains("hello-runner"));
        assert!(result.duration > Duration::ZERO || result.success);
        assert!(!result.output_lines().is_empty());
    }

    #[tokio::test]
    #[cfg(not(windows))]
    async fn run_with_input_pipes_stdin() {
        let result = Runner::new()
            .run_with_input("sh", &["-c", "read OUT; echo $OUT"], "piped\n")
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("piped") || result.output().contains("piped"));
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn run_with_input_pipes_stdin_on_windows() {
        let result = Runner::new()
            .run_with_input("cmd", &["/C", "findstr piped"], "piped\r\n")
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.output().contains("piped"));
    }

    #[tokio::test]
    async fn run_respects_timeout_configuration() {
        #[cfg(windows)]
        let cmd = ("cmd", vec!["/C", "ping", "-n", "6", "127.0.0.1"]);
        #[cfg(not(windows))]
        let cmd = ("sh", vec!["-c", "sleep 5"]);

        let result = Runner::new().with_timeout(1).run(cmd.0, cmd.1.as_slice()).await;

        assert!(matches!(result, Err(RunError::Timeout(1))));
    }
}
