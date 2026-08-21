use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command as StdCommand};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Current status of an agent process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Running,
    Idle,
    Error,
    Stopped,
}

/// Information about a managed agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub pid: Option<u32>,
    pub repo: Option<String>,
    pub started_at: Option<String>,
    pub last_heartbeat: Option<String>,
    pub log_path: Option<String>,
}

/// A single log entry from an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Global agent state (in-process)
// ---------------------------------------------------------------------------

/// Holds the child process handle alongside metadata.
struct ManagedAgent {
    info: AgentInfo,
    child: Option<Child>,
    _started_instant: Option<Instant>,
}

static AGENTS: LazyLock<Mutex<Vec<ManagedAgent>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Ensure the agents state dir exists; returns its path.
fn agents_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("helios-command-center")
        .join("agents");
    fs::create_dir_all(&dir).ok();
    dir
}

fn log_file_path(id: &str) -> PathBuf {
    agents_dir().join(format!("{id}.log"))
}

fn iso_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all known agents.
pub fn list_agents() -> Vec<AgentInfo> {
    let agents = AGENTS.lock().unwrap();
    agents.iter().map(|a| a.info.clone()).collect()
}

/// Spawn a new agent process. `repo` is an optional working directory.
pub fn spawn_agent(
    name: String,
    repo: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
) -> Result<AgentInfo, String> {
    let id = format!("agent-{}", chrono::Utc::now().timestamp_millis());
    let log_path = log_file_path(&id);

    let cmd_str = command.unwrap_or_else(|| "helios".to_string());
    let cmd_args = args.unwrap_or_default();

    let mut builder = StdCommand::new(&cmd_str);
    builder.args(&cmd_args);
    if let Some(ref r) = repo {
        builder.current_dir(r);
    }

    // Redirect stdout/stderr to the log file.
    let log_file = fs::File::create(&log_path)
        .map_err(|e| format!("Failed to create log file: {e}"))?;
    let log_file_err = log_file
        .try_clone()
        .map_err(|e| format!("Failed to clone log file handle: {e}"))?;
    builder.stdout(std::process::Stdio::from(log_file));
    builder.stderr(std::process::Stdio::from(log_file_err));

    let child = builder
        .spawn()
        .map_err(|e| format!("Failed to spawn agent: {e}"))?;

    let info = AgentInfo {
        id: id.clone(),
        name,
        status: AgentStatus::Running,
        pid: child.id(),
        repo,
        started_at: Some(iso_now()),
        last_heartbeat: Some(iso_now()),
        log_path: Some(log_path.to_string_lossy().to_string()),
    };

    let managed = ManagedAgent {
        info: info.clone(),
        child: Some(child),
        _started_instant: Some(Instant::now()),
    };

    AGENTS.lock().unwrap().push(managed);

    Ok(info)
}

/// Stop a running agent by id.
pub fn stop_agent(id: &str) -> Result<AgentInfo, String> {
    let mut agents = AGENTS.lock().unwrap();
    let agent = agents
        .iter_mut()
        .find(|a| a.info.id == id)
        .ok_or_else(|| format!("Agent {id} not found"))?;

    if let Some(ref mut child) = agent.child {
        let _ = child.kill();
        let _ = child.wait();
    }

    agent.child = None;
    agent.info.status = AgentStatus::Stopped;

    Ok(agent.info.clone())
}

/// Read the last N log entries for an agent.
pub fn get_agent_logs(id: &str, tail: Option<usize>) -> Result<Vec<AgentLogEntry>, String> {
    let agents = AGENTS.lock().unwrap();
    let agent = agents
        .iter()
        .find(|a| a.info.id == id)
        .ok_or_else(|| format!("Agent {id} not found"))?;

    let log_path = agent
        .info
        .log_path
        .as_ref()
        .ok_or_else(|| "No log file associated with this agent".to_string())?;

    let path = PathBuf::from(log_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path).map_err(|e| format!("Failed to open log: {e}"))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let tail_count = tail.unwrap_or(200);
    let start = lines.len().saturating_sub(tail_count);

    Ok(lines[start..]
        .iter()
        .enumerate()
        .map(|(i, line)| parse_log_line(i, line))
        .collect())
}

/// Get the current status of a specific agent (re-checks process).
pub fn get_agent_status(id: &str) -> Result<AgentInfo, String> {
    let mut agents = AGENTS.lock().unwrap();
    let agent = agents
        .iter_mut()
        .find(|a| a.info.id == id)
        .ok_or_else(|| format!("Agent {id} not found"))?;

    // Reap finished processes.
    if let Some(ref mut child) = agent.child {
        match child.try_wait() {
            Ok(Some(_exit)) => {
                agent.info.status = AgentStatus::Stopped;
                agent.child = None;
            }
            Ok(None) => {
                // Still running, update heartbeat.
                agent.info.last_heartbeat = Some(iso_now());
            }
            Err(_) => {
                agent.info.status = AgentStatus::Error;
                agent.child = None;
            }
        }
    }

    Ok(agent.info.clone())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a single log line into a structured entry.
/// Lines are expected in the format: `TIMESTAMP LEVEL message` or just `message`.
fn parse_log_line(_idx: usize, line: &str) -> AgentLogEntry {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    if parts.len() >= 3 {
        let possible_ts = parts[0];
        let possible_level = parts[1].to_uppercase();

        if (possible_level == "INFO"
            || possible_level == "WARN"
            || possible_level == "ERROR"
            || possible_level == "DEBUG")
            && possible_ts.contains('T')
        {
            return AgentLogEntry {
                timestamp: possible_ts.to_string(),
                level: possible_level,
                message: parts[2].to_string(),
            };
        }
    }

    AgentLogEntry {
        timestamp: String::new(),
        level: "INFO".to_string(),
        message: line.to_string(),
    }
}
