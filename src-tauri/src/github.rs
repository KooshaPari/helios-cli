use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;

// ===========================================================================
// Public types shared with the frontend
// ===========================================================================

/// A monitored repository's status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stars: u32,
    pub forks: u32,
    pub open_issues: u32,
    pub open_prs: u32,
    pub default_branch: String,
    pub last_push: Option<String>,
    pub ci_status: CIStatus,
    pub url: String,
}

/// Aggregated CI status for a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CIStatus {
    Passing,
    Pending,
    Failing,
    Unknown,
}

impl CIStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CIStatus::Passing => "passing",
            CIStatus::Pending => "pending",
            CIStatus::Failing => "failing",
            CIStatus::Unknown => "unknown",
        }
    }
}

/// A GitHub Actions workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub branch: String,
    pub commit_sha: String,
    pub commit_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub run_started_at: Option<String>,
    pub html_url: String,
}

impl WorkflowRun {
    /// Compute the run duration in seconds, if timing data is available.
    pub fn duration_secs(&self) -> Option<i64> {
        let start = self.run_started_at.as_deref()?;
        let end = &self.updated_at;
        let parse = |s: &str| chrono::DateTime::parse_from_rfc3339(s).ok();
        let start_dt = parse(start)?;
        let end_dt = parse(end)?;
        Some((end_dt - start_dt).num_seconds())
    }

    pub fn status_icon(&self) -> &str {
        match self.conclusion.as_deref() {
            Some("success") => "passing",
            Some("failure") => "failing",
            Some("cancelled") => "cancelled",
            Some("skipped") => "skipped",
            None if self.status == "in_progress" => "pending",
            None if self.status == "queued" => "pending",
            _ => "unknown",
        }
    }
}

/// An open GitHub issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub author: String,
}

/// An open GitHub pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub state: String,
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub head_sha: String,
    pub ci_status: CIStatus,
    pub review_status: String,
}

/// A repository the user wants to monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredRepo {
    pub owner: String,
    pub name: String,
}

impl MonitoredRepo {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

// ===========================================================================
// Internal GitHub API response types
// ===========================================================================

const API_BASE: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
struct GhRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    open_issues_count: u32,
    default_branch: String,
    pushed_at: Option<String>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GhWorkflowRun {
    id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    head_branch: String,
    head_sha: String,
    commit: Option<GhCommit>,
    created_at: String,
    updated_at: String,
    run_started_at: Option<String>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GhCommit {
    commit: Option<GhCommitInner>,
}

#[derive(Debug, Deserialize)]
struct GhCommitInner {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhIssue {
    number: u32,
    title: String,
    state: String,
    labels: Vec<GhLabel>,
    assignee: Option<GhAssignee>,
    created_at: String,
    updated_at: String,
    html_url: String,
    user: Option<GhUser>,
    #[serde(default)]
    pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhAssignee {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhPullRequest {
    number: u32,
    title: String,
    user: Option<GhUser>,
    state: String,
    draft: bool,
    mergeable: Option<bool>,
    labels: Vec<GhLabel>,
    created_at: String,
    updated_at: String,
    html_url: String,
    head: Option<GhPrHead>,
}

#[derive(Debug, Deserialize)]
struct GhPrHead {
    sha: String,
}

#[derive(Deserialize)]
struct RunsResponse {
    workflow_runs: Vec<GhWorkflowRun>,
}

// ===========================================================================
// Token resolution
// ===========================================================================

/// Resolve the GitHub token by checking env vars then the `gh` CLI.
pub fn resolve_token() -> Option<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    if let Ok(token) = std::env::var("GH_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    StdCommand::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !token.is_empty() {
                    Some(token)
                } else {
                    None
                }
            } else {
                None
            }
        })
}

// ===========================================================================
// HTTP client
// ===========================================================================

fn build_client(token: Option<&str>) -> Result<Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        "application/vnd.github+json".parse().unwrap(),
    );
    if let Some(t) = token {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {t}").parse().unwrap(),
        );
    }
    Client::builder()
        .user_agent("helios-command-center/0.1.0")
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

// ===========================================================================
// Public API functions
// ===========================================================================

/// Fetch a single repository's metadata.
pub async fn fetch_repo(
    owner: &str,
    name: &str,
    token: Option<&str>,
) -> Result<RepoStatus, String> {
    let client = build_client(token)?;
    let url = format!("{API_BASE}/repos/{owner}/{name}");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for {owner}/{name}",
            resp.status()
        ));
    }

    let repo: GhRepo = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(RepoStatus {
        name: repo.name,
        full_name: repo.full_name,
        description: repo.description,
        stars: repo.stargazers_count,
        forks: repo.forks_count,
        open_issues: repo.open_issues_count,
        open_prs: 0,
        default_branch: repo.default_branch,
        last_push: repo.pushed_at,
        ci_status: CIStatus::Unknown,
        url: repo.html_url,
    })
}

/// Fetch recent workflow runs for a repository.
pub async fn fetch_workflow_runs(
    owner: &str,
    name: &str,
    token: Option<&str>,
    per_page: u32,
) -> Result<Vec<WorkflowRun>, String> {
    let client = build_client(token)?;
    let url = format!("{API_BASE}/repos/{owner}/{name}/actions/runs?per_page={per_page}");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for workflow runs",
            resp.status()
        ));
    }

    let data: RunsResponse = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(data
        .workflow_runs
        .into_iter()
        .map(|r| WorkflowRun {
            id: r.id,
            name: r.name,
            status: r.status,
            conclusion: r.conclusion,
            branch: r.head_branch,
            commit_sha: r.head_sha,
            commit_message: r.commit.and_then(|c| c.commit).and_then(|c| c.message),
            created_at: r.created_at,
            updated_at: r.updated_at,
            run_started_at: r.run_started_at,
            html_url: r.html_url,
        })
        .collect())
}

/// Fetch open issues for a repository, filtering out pull requests.
pub async fn fetch_issues(
    owner: &str,
    name: &str,
    token: Option<&str>,
) -> Result<Vec<Issue>, String> {
    let client = build_client(token)?;
    let url = format!("{API_BASE}/repos/{owner}/{name}/issues?state=open&per_page=50");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {} for issues", resp.status()));
    }

    let items: Vec<GhIssue> = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(items
        .into_iter()
        .filter(|i| i.pull_request.is_none())
        .map(|i| Issue {
            number: i.number,
            title: i.title,
            state: i.state,
            labels: i.labels.into_iter().map(|l| l.name).collect(),
            assignee: i.assignee.map(|a| a.login),
            created_at: i.created_at,
            updated_at: i.updated_at,
            html_url: i.html_url,
            author: i.user.map(|u| u.login).unwrap_or_default(),
        })
        .collect())
}

/// Fetch open pull requests for a repository.
pub async fn fetch_prs(
    owner: &str,
    name: &str,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, String> {
    let client = build_client(token)?;
    let url = format!("{API_BASE}/repos/{owner}/{name}/pulls?state=open&per_page=50");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for pull requests",
            resp.status()
        ));
    }

    let items: Vec<GhPullRequest> = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(items
        .into_iter()
        .map(|p| PullRequest {
            number: p.number,
            title: p.title,
            author: p.user.map(|u| u.login).unwrap_or_default(),
            state: p.state,
            draft: p.draft,
            mergeable: p.mergeable,
            labels: p.labels.into_iter().map(|l| l.name).collect(),
            created_at: p.created_at,
            updated_at: p.updated_at,
            html_url: p.html_url,
            head_sha: p.head.map(|h| h.sha).unwrap_or_default(),
            ci_status: CIStatus::Unknown,
            review_status: if p.draft {
                "draft".to_string()
            } else {
                "pending".to_string()
            },
        })
        .collect())
}
