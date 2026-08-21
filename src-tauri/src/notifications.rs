use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which tool generated this notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSource {
    Tracera,
    AgilePlus,
    GitHub,
    Helios,
}

/// The category of notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    TraceraIssue,
    AgilePlusGateFailure,
    AgentError,
    CIStatus,
    TaskComplete,
}

/// A single notification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub source: NotificationSource,
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

/// Summary counts grouped by source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCounts {
    pub total: u32,
    pub unread: u32,
    pub by_source: Vec<SourceCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCount {
    pub source: NotificationSource,
    pub total: u32,
    pub unread: u32,
}

// ---------------------------------------------------------------------------
// Database
// ---------------------------------------------------------------------------

fn db_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("helios-command-center");
    fs::create_dir_all(&dir).ok();
    dir.join("notifications.db")
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
        Connection::open(db_path()).map_err(|e| format!("Failed to open notifications DB: {e}"))?;
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS notifications (
                id              TEXT PRIMARY KEY,
                source          TEXT NOT NULL,
                notification_type TEXT NOT NULL,
                title           TEXT NOT NULL,
                body            TEXT NOT NULL,
                link            TEXT,
                is_read         INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_notifications_source
                ON notifications(source);
            CREATE INDEX IF NOT EXISTS idx_notifications_is_read
                ON notifications(is_read);
            CREATE INDEX IF NOT EXISTS idx_notifications_created
                ON notifications(created_at DESC);",
        )
        .map_err(|e| format!("Failed to create notifications table: {e}"))?;
    *guard = Some(connection);
    Ok(())
}

fn get_connection() -> Result<Connection, String> {
    ensure_db()?;
    let guard = conn()?;
    Ok(guard.as_ref().unwrap().unchecked_clone())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_source(s: &str) -> NotificationSource {
    match s {
        "tracera" => NotificationSource::Tracera,
        "agileplus" => NotificationSource::AgilePlus,
        "github" => NotificationSource::GitHub,
        "helios" => NotificationSource::Helios,
        _ => NotificationSource::Helios,
    }
}

fn parse_type(s: &str) -> NotificationType {
    match s {
        "tracera_issue" => NotificationType::TraceraIssue,
        "agile_plus_gate_failure" => NotificationType::AgilePlusGateFailure,
        "agent_error" => NotificationType::AgentError,
        "ci_status" => NotificationType::CIStatus,
        "task_complete" => NotificationType::TaskComplete,
        _ => NotificationType::TaskComplete,
    }
}

fn source_to_str(s: &NotificationSource) -> &'static str {
    match s {
        NotificationSource::Tracera => "tracera",
        NotificationSource::AgilePlus => "agileplus",
        NotificationSource::GitHub => "github",
        NotificationSource::Helios => "helios",
    }
}

fn type_to_str(t: &NotificationType) -> &'static str {
    match t {
        NotificationType::TraceraIssue => "tracera_issue",
        NotificationType::AgilePlusGateFailure => "agile_plus_gate_failure",
        NotificationType::AgentError => "agent_error",
        NotificationType::CIStatus => "ci_status",
        NotificationType::TaskComplete => "task_complete",
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new notification and persist it.
pub fn create_notification(
    source: NotificationSource,
    notification_type: NotificationType,
    title: String,
    body: String,
    link: Option<String>,
) -> Result<Notification, String> {
    let id = format!("notif-{}", chrono::Utc::now().timestamp_millis());
    let created_at = chrono::Utc::now().to_rfc3339();

    let notification = Notification {
        id: id.clone(),
        source: source.clone(),
        notification_type: notification_type.clone(),
        title: title.clone(),
        body: body.clone(),
        link: link.clone(),
        is_read: false,
        created_at: created_at.clone(),
    };

    let db = get_connection()?;
    db.execute(
        "INSERT INTO notifications (id, source, notification_type, title, body, link, is_read, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
        params![
            notification.id,
            source_to_str(&source),
            type_to_str(&notification_type),
            title,
            body,
            link,
            created_at,
        ],
    )
    .map_err(|e| format!("Insert notification failed: {e}"))?;

    Ok(notification)
}

/// List notifications, most recent first.
pub fn list_notifications(limit: Option<u32>) -> Result<Vec<Notification>, String> {
    let db = get_connection()?;
    let lim = limit.unwrap_or(100);
    let mut stmt = db
        .prepare(
            "SELECT id, source, notification_type, title, body, link, is_read, created_at
             FROM notifications
             ORDER BY created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Prepare failed: {e}"))?;

    let rows = stmt
        .query_map(params![lim], |row| {
            Ok(Notification {
                id: row.get(0)?,
                source: parse_source(&row.get::<_, String>(1)?),
                notification_type: parse_type(&row.get::<_, String>(2)?),
                title: row.get(3)?,
                body: row.get(4)?,
                link: row.get(5)?,
                is_read: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut notifications = Vec::new();
    for row in rows {
        notifications.push(row.map_err(|e| format!("Row read failed: {e}"))?);
    }
    Ok(notifications)
}

/// Mark a notification as read.
pub fn mark_read(notification_id: &str) -> Result<(), String> {
    let db = get_connection()?;
    db.execute(
        "UPDATE notifications SET is_read = 1 WHERE id = ?1",
        params![notification_id],
    )
    .map_err(|e| format!("Mark read failed: {e}"))?;
    Ok(())
}

/// Mark all notifications as read.
pub fn mark_all_read() -> Result<(), String> {
    let db = get_connection()?;
    db.execute("UPDATE notifications SET is_read = 1", [])
        .map_err(|e| format!("Mark all read failed: {e}"))?;
    Ok(())
}

/// Get counts by source.
pub fn get_counts_by_source() -> Result<NotificationCounts, String> {
    let db = get_connection()?;
    let mut stmt = db
        .prepare(
            "SELECT source,
                    COUNT(*) as total,
                    SUM(CASE WHEN is_read = 0 THEN 1 ELSE 0 END) as unread
             FROM notifications
             GROUP BY source",
        )
        .map_err(|e| format!("Prepare counts query failed: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let source: String = row.get(0)?;
            let total: u32 = row.get(1)?;
            let unread: u32 = row.get(2)?;
            Ok(SourceCount {
                source: parse_source(&source),
                total,
                unread,
            })
        })
        .map_err(|e| format!("Query counts failed: {e}"))?;

    let mut by_source = Vec::new();
    let mut total = 0u32;
    let mut unread = 0u32;

    for row in rows.flatten() {
        total += row.total;
        unread += row.unread;
        by_source.push(row);
    }

    Ok(NotificationCounts {
        total,
        unread,
        by_source,
    })
}
