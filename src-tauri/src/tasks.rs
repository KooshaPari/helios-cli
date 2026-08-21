use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of a task in the queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

/// Result payload of a completed (or failed) task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub summary: String,
    pub artifacts: Vec<String>,
}

/// A unit of work in the task queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub assignee_agent: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result: Option<TaskResult>,
    pub error: Option<String>,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::RolledBack => "rolled_back",
        }
    }
}

impl Task {
    /// Returns the status as a human-readable string.
    pub fn status_str(&self) -> &'static str {
        self.status.as_str()
    }
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

fn db_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("helios-command-center");
    fs::create_dir_all(&dir).ok();
    dir.join("tasks.db")
}

static DB: LazyLock<Mutex<Option<Connection>>> = LazyLock::new(|| Mutex::new(None));

fn conn() -> Result<std::sync::MutexGuard<'static, Option<Connection>>, String> {
    let guard = DB.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
    Ok(guard)
}

fn ensure_db() -> Result<(), String> {
    let mut guard = conn()?;
    if guard.is_some() {
        return Ok(());
    }
    let connection =
        Connection::open(db_path()).map_err(|e| format!("Failed to open DB: {e}"))?;
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id            TEXT PRIMARY KEY,
                title         TEXT NOT NULL,
                status        TEXT NOT NULL DEFAULT 'pending',
                assignee      TEXT,
                created_at    TEXT NOT NULL,
                started_at    TEXT,
                completed_at  TEXT,
                result_json   TEXT,
                error         TEXT
            );",
        )
        .map_err(|e| format!("Failed to create table: {e}"))?;
    *guard = Some(connection);
    Ok(())
}

fn get_connection() -> Result<Connection, String> {
    ensure_db()?;
    let guard = conn()?;
    // Safety: we just ensured it is Some.
    Ok(guard.as_ref().unwrap().unchecked_clone())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new task and persist it.
pub fn create_task(title: String, assignee_agent: Option<String>) -> Result<Task, String> {
    let id = format!("task-{}", chrono::Utc::now().timestamp_millis());
    let created_at = chrono::Utc::now().to_rfc3339();

    let task = Task {
        id: id.clone(),
        title,
        status: TaskStatus::Pending,
        assignee_agent,
        created_at,
        started_at: None,
        completed_at: None,
        result: None,
        error: None,
    };

    let db = get_connection()?;
    db.execute(
        "INSERT INTO tasks (id, title, status, assignee, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![task.id, task.title, "pending", task.assignee_agent, task.created_at],
    )
    .map_err(|e| format!("Insert failed: {e}"))?;

    Ok(task)
}

/// List all tasks, most recent first.
pub fn list_tasks() -> Result<Vec<Task>, String> {
    let db = get_connection()?;
    let mut stmt = db
        .prepare(
            "SELECT id, title, status, assignee, created_at, started_at, completed_at, result_json, error
             FROM tasks ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Prepare failed: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let status_str: String = row.get(2)?;
            let status = match status_str.as_str() {
                "running" => TaskStatus::Running,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                "rolled_back" => TaskStatus::RolledBack,
                _ => TaskStatus::Pending,
            };

            let result_json: Option<String> = row.get(7)?;
            let result: Option<TaskResult> = result_json
                .and_then(|j| serde_json::from_str(&j).ok());

            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                status,
                assignee_agent: row.get(3)?,
                created_at: row.get(4)?,
                started_at: row.get(5)?,
                completed_at: row.get(6)?,
                result,
                error: row.get(8)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row.map_err(|e| format!("Row read failed: {e}"))?);
    }
    Ok(tasks)
}

/// Update the status of a task.
pub fn update_task_status(
    task_id: String,
    status: TaskStatus,
    error: Option<String>,
) -> Result<Task, String> {
    let db = get_connection()?;

    let status_str = match &status {
        TaskStatus::Pending => "pending",
        TaskStatus::Running => "running",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
        TaskStatus::RolledBack => "rolled_back",
    };

    let now = chrono::Utc::now().to_rfc3339();

    match &status {
        TaskStatus::Running => {
            db.execute(
                "UPDATE tasks SET status = ?1, started_at = ?2 WHERE id = ?3",
                params![status_str, now, task_id],
            )
            .map_err(|e| format!("Update failed: {e}"))?;
        }
        TaskStatus::Completed | TaskStatus::Failed => {
            db.execute(
                "UPDATE tasks SET status = ?1, completed_at = ?2, error = ?3 WHERE id = ?4",
                params![status_str, now, error, task_id],
            )
            .map_err(|e| format!("Update failed: {e}"))?;
        }
        _ => {
            db.execute(
                "UPDATE tasks SET status = ?1, error = ?2 WHERE id = ?3",
                params![status_str, error, task_id],
            )
            .map_err(|e| format!("Update failed: {e}"))?;
        }
    }

    // Return updated task.
    let tasks = list_tasks()?;
    tasks
        .into_iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {task_id} not found after update"))
}

/// Rollback a completed task.
pub fn rollback_task(task_id: String) -> Result<Task, String> {
    update_task_status(task_id, TaskStatus::RolledBack, Some("Rolled back by user".to_string()))
}

/// Get task history (all tasks with status changes, ordered by creation).
pub fn get_task_history() -> Result<Vec<Task>, String> {
    list_tasks()
}
