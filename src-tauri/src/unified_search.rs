use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which tool / data source a search result originates from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    Tracera,
    AgilePlus,
    GitHub,
    Helios,
}

/// The kind of entity a search result represents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Issue,
    Task,
    Spec,
    PullRequest,
    Workflow,
    Notification,
    Agent,
}

/// Query sent from the frontend for a unified search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    #[serde(default)]
    pub source_filter: Option<ToolSource>,
    #[serde(default)]
    pub type_filter: Option<ResultType>,
}

/// A single search result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source: ToolSource,
    #[serde(rename = "type")]
    pub result_type: ResultType,
    pub title: String,
    pub snippet: String,
    pub url: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    /// Relevance score (0-100, higher = more relevant).
    pub score: u32,
}

// ---------------------------------------------------------------------------
// Tracera search  (SQLite)
// ---------------------------------------------------------------------------

/// Path to a Tracera SQLite database (if present on disk).
fn tracera_db_path() -> Option<PathBuf> {
    dirs::data_local_dir()
        .map(|d| d.join("tracera").join("tracera.db"))
        .filter(|p| p.exists())
}

/// Search Tracera issues stored in a local SQLite database.
fn search_tracera(query: &SearchQuery) -> Vec<SearchResult> {
    let db_path = match tracera_db_path() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Attempt to query an `issues` table if it exists.
    let sql = "SELECT id, title, description, status, priority, created_at \
               FROM issues \
               WHERE title LIKE ?1 OR description LIKE ?1 \
               ORDER BY created_at DESC \
               LIMIT 20";

    let pattern = format!("%{}%", query.text);
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt
        .query_map(rusqlite::params![pattern], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2).unwrap_or_default();
            let status: String = row.get(3).unwrap_or_default();
            Ok((id, title, description, status))
        })
        .unwrap_or_default();

    let mut results = Vec::new();
    for row in rows.flatten() {
        let (id, title, description, status) = row;
        let snippet = truncate(&description, 200);
        results.push(SearchResult {
            source: ToolSource::Tracera,
            result_type: ResultType::Issue,
            title,
            snippet: format!("[{status}] {snippet}"),
            url: Some(format!("tracera://issue/{id}")),
            timestamp: None,
            score: compute_relevance(&query.text, &title),
        });
    }
    results
}

// ---------------------------------------------------------------------------
// AgilePlus search  (filesystem-based specs)
// ---------------------------------------------------------------------------

/// Look for AgilePlus spec directories.
fn agileplus_spec_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        let candidate = home.join("AgilePlus").join("specs");
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }
    // Also check the current working directory for an AgilePlus folder.
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("AgilePlus").join("specs");
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }
    roots
}

/// Search AgilePlus specs on the filesystem.
fn search_agileplus(query: &SearchQuery) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let pattern = query.text.to_lowercase();

    for root in agileplus_spec_roots() {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if file_name.to_lowercase().contains(&pattern) {
                    let content_preview = std::fs::read_to_string(&path)
                        .map(|c| truncate(&c, 200))
                        .unwrap_or_else(|_| "(unreadable)".to_string());

                    results.push(SearchResult {
                        source: ToolSource::AgilePlus,
                        result_type: ResultType::Spec,
                        title: file_name.clone(),
                        snippet: content_preview,
                        url: Some(path.to_string_lossy().to_string()),
                        timestamp: path
                            .metadata()
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| {
                                let dt: DateTime<Utc> = t.into();
                                Some(dt)
                            }),
                        score: compute_relevance(&query.text, &file_name),
                    });
                }
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// GitHub search (via config-known repos – local task + PR data)
// ---------------------------------------------------------------------------

/// Search GitHub-related data (PRs, workflows) across configured repos.
/// This is a lightweight local search – we look at cached data.
fn search_github(query: &SearchQuery) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let pattern = query.text.to_lowercase();

    // Search cached PR info from the tasks database.
    let db_path = dirs::data_local_dir()
        .map(|d| d.join("helios-command-center").join("tasks.db"));

    if let Some(path) = db_path {
        if let Ok(conn) =
            rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        {
            // Try querying cached GitHub items if a table exists.
            let sql = "SELECT id, title, description FROM github_cache \
                       WHERE title LIKE ?1 OR description LIKE ?1 \
                       LIMIT 10";
            let mut stmt = match conn.prepare(sql) {
                Ok(s) => s,
                Err(_) => return results,
            };

            let pat = format!("%{}%", query.text);
            if let Ok(rows) = stmt.query_map(rusqlite::params![pat], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let desc: String = row.get(2).unwrap_or_default();
                Ok((id, title, desc))
            }) {
                for row in rows.flatten() {
                    let (id, title, desc) = row;
                    results.push(SearchResult {
                        source: ToolSource::GitHub,
                        result_type: ResultType::PullRequest,
                        title,
                        snippet: truncate(&desc, 200),
                        url: Some(format!("github://pr/{id}")),
                        timestamp: None,
                        score: compute_relevance(&query.text, &title),
                    });
                }
            }
        }
    }

    // Also search Helios task titles that reference PRs or GitHub.
    let task_results = crate::tasks::list_tasks().unwrap_or_default();
    for task in &task_results {
        if task.title.to_lowercase().contains(&pattern) {
            results.push(SearchResult {
                source: ToolSource::GitHub,
                result_type: ResultType::Task,
                title: task.title.clone(),
                snippet: format!("[helios task] status: {}", task.status_str()),
                url: Some(format!("helios://task/{}", task.id)),
                timestamp: None,
                score: compute_relevance(&query.text, &task.title),
            });
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Helios task search
// ---------------------------------------------------------------------------

/// Search Helios internal tasks.
fn search_helios_tasks(query: &SearchQuery) -> Vec<SearchResult> {
    let tasks = match crate::tasks::list_tasks() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let pattern = query.text.to_lowercase();
    let mut results = Vec::new();

    for task in &tasks {
        if task.title.to_lowercase().contains(&pattern) {
            results.push(SearchResult {
                source: ToolSource::Helios,
                result_type: ResultType::Task,
                title: task.title.clone(),
                snippet: format!(
                    "Status: {} | Agent: {}",
                    task.status_str(),
                    task.assignee_agent.as_deref().unwrap_or("none")
                ),
                url: Some(format!("helios://task/{}", task.id)),
                timestamp: chrono::DateTime::parse_from_rfc3339(&task.created_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc)),
                score: compute_relevance(&query.text, &task.title),
            });
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Unified search entry point
// ---------------------------------------------------------------------------

/// Execute a unified search across all connected tools in parallel and return
/// merged, ranked results.
pub async fn unified_search(query: SearchQuery) -> Vec<SearchResult> {
    let query_clone = query.clone();

    // Run all searchers concurrently using tokio::task::spawn_blocking
    // for the synchronous I/O-bound searches.
    let (tracera_tx, tracera_rx) = tokio::sync::oneshot::channel();
    let (agileplus_tx, agileplus_rx) = tokio::sync::oneshot::channel();
    let (github_tx, github_rx) = tokio::sync::oneshot::channel();
    let (helios_tx, helios_rx) = tokio::sync::oneshot::channel();

    let q1 = query.clone();
    tokio::task::spawn_blocking(move || {
        let _ = tracera_tx.send(search_tracera(&q1));
    });

    let q2 = query.clone();
    tokio::task::spawn_blocking(move || {
        let _ = agileplus_tx.send(search_agileplus(&q2));
    });

    let q3 = query.clone();
    tokio::task::spawn_blocking(move || {
        let _ = github_tx.send(search_github(&q3));
    });

    let q4 = query_clone;
    tokio::task::spawn_blocking(move || {
        let _ = helios_tx.send(search_helios_tasks(&q4));
    });

    // Collect results from all sources.
    let mut all_results = Vec::new();
    if let Ok(r) = tracera_rx.await {
        all_results.extend(r);
    }
    if let Ok(r) = agileplus_rx.await {
        all_results.extend(r);
    }
    if let Ok(r) = github_rx.await {
        all_results.extend(r);
    }
    if let Ok(r) = helios_rx.await {
        all_results.extend(r);
    }

    // Apply type filter if provided.
    if let Some(ref type_filter) = query.type_filter {
        all_results.retain(|r| &r.result_type == type_filter);
    }

    // Apply source filter if provided.
    if let Some(ref source_filter) = query.source_filter {
        all_results.retain(|r| &r.source == source_filter);
    }

    // Sort by score (descending), then by timestamp (newest first).
    all_results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.timestamp.cmp(&a.timestamp))
    });

    // Limit to top 50 results.
    all_results.truncate(50);
    all_results
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Simple relevance score: higher if the query appears more prominently in
/// the title.
fn compute_relevance(query: &str, title: &str) -> u32 {
    let q = query.to_lowercase();
    let t = title.to_lowercase();

    if t == q {
        100
    } else if t.starts_with(&q) {
        90
    } else if t.contains(&q) {
        70
    } else {
        // Fuzzy: count how many query words appear.
        let words: Vec<&str> = q.split_whitespace().collect();
        if words.is_empty() {
            return 0;
        }
        let matches = words.iter().filter(|w| t.contains(**w)).count();
        ((matches as f32 / words.len() as f32) * 60.0) as u32
    }
}

/// Truncate a string to `max_chars`, appending "..." if truncated.
fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", &s[..max_chars.saturating_sub(3)])
    }
}
