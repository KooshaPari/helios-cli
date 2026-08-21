mod config;
mod github;

use config::AppConfig;
use github::{CIStatus, Issue, PullRequest, RepoStatus, WorkflowRun};
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

// ---------------------------------------------------------------------------
// IPC command handlers
// ---------------------------------------------------------------------------

/// Get the status of all monitored repos.
#[tauri::command]
async fn get_repo_status() -> Result<Vec<RepoStatus>, String> {
    let app_config = AppConfig::load();
    let token = github::resolve_token();
    let token_ref = token.as_deref();

    let mut statuses = Vec::new();
    for repo in &app_config.repos {
        match github::fetch_repo(&repo.owner, &repo.name, token_ref).await {
            Ok(mut status) => {
                // Fetch latest CI status
                match github::fetch_workflow_runs(&repo.owner, &repo.name, token_ref, 1).await {
                    Ok(runs) if !runs.is_empty() => {
                        status.ci_status = match runs[0].conclusion.as_deref() {
                            Some("success") => CIStatus::Passing,
                            Some("failure") => CIStatus::Failing,
                            Some("cancelled") => CIStatus::Failing,
                            None => CIStatus::Pending,
                            _ => CIStatus::Unknown,
                        };
                    }
                    _ => {}
                }
                // Fetch open PR count
                match github::fetch_prs(&repo.owner, &repo.name, token_ref).await {
                    Ok(prs) => {
                        status.open_prs = prs.len() as u32;
                    }
                    _ => {}
                }
                statuses.push(status);
            }
            Err(e) => {
                eprintln!("Failed to fetch repo {}/{}: {e}", repo.owner, repo.name);
            }
        }
    }
    Ok(statuses)
}

/// Get recent CI workflow runs for a repo.
#[tauri::command]
async fn get_ci_runs(owner: String, name: String) -> Result<Vec<WorkflowRun>, String> {
    let token = github::resolve_token();
    github::fetch_workflow_runs(&owner, &name, token.as_deref(), 10).await
}

/// Get open issues for a repo.
#[tauri::command]
async fn get_open_issues(owner: String, name: String) -> Result<Vec<Issue>, String> {
    let token = github::resolve_token();
    github::fetch_issues(&owner, &name, token.as_deref()).await
}

/// Get open pull requests for a repo.
#[tauri::command]
async fn get_open_prs(owner: String, name: String) -> Result<Vec<PullRequest>, String> {
    let token = github::resolve_token();
    github::fetch_prs(&owner, &name, token.as_deref()).await
}

/// Add a repository to the monitored list.
#[tauri::command]
async fn add_repo(owner: String, name: String) -> Result<AppConfig, String> {
    let mut app_config = AppConfig::load();
    app_config.add_repo(owner, name);
    app_config.save()?;
    Ok(app_config)
}

/// Remove a repository from the monitored list.
#[tauri::command]
async fn remove_repo(full_name: String) -> Result<AppConfig, String> {
    let mut app_config = AppConfig::load();
    app_config.remove_repo(&full_name);
    app_config.save()?;
    Ok(app_config)
}

/// List all monitored repos (returns full config).
#[tauri::command]
async fn list_repos() -> Result<AppConfig, String> {
    Ok(AppConfig::load())
}

/// Update app configuration.
#[tauri::command]
async fn update_config(config: AppConfig) -> Result<AppConfig, String> {
    config.save()?;
    Ok(config)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let system_tray = SystemTray::new().with_menu(
        SystemTrayMenu::new().add_item(SystemTrayMenuItem::new("Show Window")),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                if id.0 == "Show Window" {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_repo_status,
            get_ci_runs,
            get_open_issues,
            get_open_prs,
            add_repo,
            remove_repo,
            list_repos,
            update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Helios Command Center");
}
