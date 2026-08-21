use crate::github::MonitoredRepo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub repos: Vec<MonitoredRepo>,
    pub github_token: Option<String>,
    pub refresh_interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            github_token: None,
            refresh_interval_secs: 60,
        }
    }
}

/// Returns the path to the Helios config directory.
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("helios-command-center")
}

/// Returns the full path to the config JSON file.
fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

impl AppConfig {
    /// Load config from disk, returning defaults if not found.
    pub fn load() -> Self {
        let path = config_file();
        if !path.exists() {
            return Self::default();
        }
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
        let path = config_file();
        let data =
            serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("Failed to write config: {e}"))?;
        Ok(())
    }

    /// Add a repo to the monitored list (deduplicates).
    pub fn add_repo(&mut self, owner: String, name: String) -> bool {
        let full = format!("{owner}/{name}");
        if self.repos.iter().any(|r| r.full_name() == full) {
            return false;
        }
        self.repos.push(MonitoredRepo { owner, name });
        true
    }

    /// Remove a repo from the monitored list.
    pub fn remove_repo(&mut self, full_name: &str) -> bool {
        let len_before = self.repos.len();
        self.repos.retain(|r| r.full_name() != full_name);
        self.repos.len() < len_before
    }
}
