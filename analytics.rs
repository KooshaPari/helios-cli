//! Analytics integration for helios-cli
//!
//! Traces to: FR-HELIOS-ANALYTICS-001
//!
//! Product analytics for Helios CLI operations

use phenotype_analytics::{AnalyticsClient, AnalyticsConfig, EventType, Result};
use std::sync::OnceLock;

static ANALYTICS: OnceLock<AnalyticsClient> = OnceLock::new();

/// Initialize analytics for the CLI
pub fn init() -> Result<()> {
    let api_key = std::env::var("PHENOTYPE_ANALYTICS_KEY").ok();
    
    if api_key.is_none() {
        // Analytics disabled
        return Ok(());
    }
    
    let config = AnalyticsConfig {
        api_key: api_key.unwrap(),
        environment: std::env::var("PHENOTYPE_ENV").unwrap_or_else(|_| "development".to_string()),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ..Default::default()
    };
    
    let client = AnalyticsClient::new(config)?;
    let _ = ANALYTICS.set(client);
    
    Ok(())
}

/// Track a CLI command execution
pub async fn track_command(command: &str, args: &[String], duration_ms: u64, success: bool) {
    if let Some(client) = ANALYTICS.get() {
        let event_type = if success {
            EventType::OperationCompleted
        } else {
            EventType::ErrorOccurred
        };
        
        let _ = client.track(
            event_type,
            serde_json::json!({
                "command": command,
                "args_count": args.len(),
                "duration_ms": duration_ms,
                "success": success,
            }),
        ).await;
    }
}

/// Track workflow execution
pub async fn track_workflow(workflow: &str, status: &str, project_id: Option<&str>) {
    if let Some(client) = ANALYTICS.get() {
        let event_type = match status {
            "started" => EventType::WorkflowStarted,
            "completed" => EventType::WorkflowCompleted,
            "failed" => EventType::WorkflowFailed,
            _ => EventType::Custom("workflow.status"),
        };
        
        let _ = client.track(
            event_type,
            serde_json::json!({
                "workflow": workflow,
                "status": status,
                "project_id": project_id,
            }),
        ).await;
    }
}

/// Track feature usage
pub async fn track_feature(feature: &str, action: &str) {
    if let Some(client) = ANALYTICS.get() {
        let _ = client.track(
            EventType::FeatureUsed,
            serde_json::json!({
                "feature": feature,
                "action": action,
            }),
        ).await;
    }
}

/// Identify CLI user
pub async fn identify_user(user_id: &str, email: Option<&str>) {
    if let Some(client) = ANALYTICS.get() {
        let _ = client.identify(
            user_id,
            serde_json::json!({
                "email": email,
                "cli_version": env!("CARGO_PKG_VERSION"),
            }),
        ).await;
    }
}

/// Flush pending events before exit
pub async fn flush() {
    if let Some(client) = ANALYTICS.get() {
        let _ = client.flush().await;
    }
}
