// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Rollback Engine - Simple version

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RollbackStatus {
    Pending,
    Started,
    Completed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    pub id: Uuid,
    pub checkpoint_id: String,
    pub spec_id: String,
    pub status: RollbackStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub restored_items: Vec<String>,
    pub failed_items: Vec<String>,
    pub error: Option<String>,
}

impl RollbackRecord {
    pub fn new(checkpoint_id: &str, spec_id: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            checkpoint_id: checkpoint_id.to_string(),
            spec_id: spec_id.to_string(),
            status: RollbackStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            restored_items: vec![],
            failed_items: vec![],
            error: None,
        }
    }

    pub fn start(&mut self) {
        self.status = RollbackStatus::Started;
    }
    pub fn add_restored(&mut self, item: &str) {
        self.restored_items.push(item.to_string());
    }
    pub fn add_failed(&mut self, item: &str) {
        self.failed_items.push(item.to_string());
    }
    pub fn complete(&mut self) {
        self.status = if self.failed_items.is_empty() {
            RollbackStatus::Completed
        } else {
            RollbackStatus::Partial
        };
        self.completed_at = Some(Utc::now());
    }
    pub fn fail(&mut self, err: &str) {
        self.status = RollbackStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(err.to_string());
    }
}

pub struct RollbackEngine {
    records: Vec<RollbackRecord>,
    checkpoints: Vec<(String, String)>,
}

impl Default for RollbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackEngine {
    pub fn new() -> Self {
        Self { records: vec![], checkpoints: vec![] }
    }

    pub fn register(&mut self, checkpoint_id: &str, spec_id: &str) -> Uuid {
        let id = Uuid::new_v4();
        self.checkpoints.push((checkpoint_id.to_string(), spec_id.to_string()));
        id
    }

    pub fn rollback(&mut self, checkpoint_id: &str) -> Option<RollbackRecord> {
        let mut record = RollbackRecord::new(checkpoint_id, "spec");
        record.start();
        record.add_restored(&format!("git:{}", checkpoint_id));
        record.complete();
        self.records.push(record.clone());
        Some(record)
    }

    pub fn verify(&self, record: &RollbackRecord) -> bool {
        matches!(record.status, RollbackStatus::Completed)
    }

    pub fn history(&self) -> &[RollbackRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback() {
        let mut engine = RollbackEngine::new();
        engine.register("chk-001", "test-spec");
        let result = engine.rollback("chk-001");
        assert!(result.is_some());
        let result = engine.rollback("chk-001");
        assert!(result.is_some());
        let record = result.expect("rollback returned a record");
        assert!(engine.verify(&record));
    }

    #[test]
    fn rollback_record_partial_when_failures_present() {
        let mut record = RollbackRecord::new("chk-002", "spec");
        record.start();
        record.add_restored("file-a");
        record.add_failed("file-b");
        record.complete();
        assert_eq!(record.status, RollbackStatus::Partial);
    }

    // ------------------------------------------------------------------
    // Additional tests
    // ------------------------------------------------------------------

    /// RollbackRecord::new initializes with correct default state.
    #[test]
    fn rollback_record_new_defaults() {
        let record = RollbackRecord::new("cp-100", "my-spec");
        assert_eq!(record.checkpoint_id, "cp-100");
        assert_eq!(record.spec_id, "my-spec");
        assert_eq!(record.status, RollbackStatus::Pending);
        assert!(record.completed_at.is_none());
        assert!(record.restored_items.is_empty());
        assert!(record.failed_items.is_empty());
        assert!(record.error.is_none());
    }

    /// RollbackRecord::start transitions status to Started.
    #[test]
    fn rollback_record_start_transition() {
        let mut record = RollbackRecord::new("cp", "sp");
        assert_eq!(record.status, RollbackStatus::Pending);
        record.start();
        assert_eq!(record.status, RollbackStatus::Started);
    }

    /// RollbackRecord::fail records error and sets Failed status.
    #[test]
    fn rollback_record_fail_records_error() {
        let mut record = RollbackRecord::new("cp", "sp");
        record.start();
        record.fail("something went wrong");
        assert_eq!(record.status, RollbackStatus::Failed);
        assert_eq!(record.error.as_deref(), Some("something went wrong"));
        assert!(record.completed_at.is_some());
    }

    /// RollbackRecord: complete with no failures yields Completed.
    #[test]
    fn rollback_record_complete_no_failures() {
        let mut record = RollbackRecord::new("cp", "sp");
        record.start();
        record.add_restored("item-a");
        record.complete();
        assert_eq!(record.status, RollbackStatus::Completed);
        assert!(record.completed_at.is_some());
    }

    /// RollbackEngine: default creates empty engine.
    #[test]
    fn rollback_engine_default_empty() {
        let engine = RollbackEngine::default();
        assert!(engine.history().is_empty());
    }

    /// RollbackEngine: register stores checkpoint mapping.
    #[test]
    fn rollback_engine_register_checkpoint() {
        let mut engine = RollbackEngine::new();
        let id = engine.register("cp-a", "spec-a");
        // Register returns a valid UUID.
        assert!(!id.is_nil());
    }

    /// RollbackEngine: history tracks all rollback records.
    #[test]
    fn rollback_engine_history_tracking() {
        let mut engine = RollbackEngine::new();
        engine.rollback("cp-1");
        engine.rollback("cp-2");
        let history = engine.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].checkpoint_id, "cp-1");
        assert_eq!(history[1].checkpoint_id, "cp-2");
    }

    /// RollbackEngine: verify fails on partial records.
    #[test]
    fn rollback_engine_verify_partial() {
        let engine = RollbackEngine::new();
        let mut record = RollbackRecord::new("cp", "spec");
        record.start();
        record.add_restored("ok-item");
        record.add_failed("bad-item");
        record.complete();
        assert!(!engine.verify(&record));
    }

    /// RollbackEngine: verify passes on completed records.
    #[test]
    fn rollback_engine_verify_completed() {
        let mut engine = RollbackEngine::new();
        let record = engine.rollback("cp-ok").unwrap();
        assert!(engine.verify(&record));
    }

    /// RollbackStatus: serialization round-trip.
    #[test]
    fn rollback_status_serialization_roundtrip() {
        let statuses = [
            RollbackStatus::Pending,
            RollbackStatus::Started,
            RollbackStatus::Completed,
            RollbackStatus::Failed,
            RollbackStatus::Partial,
        ];
        for s in &statuses {
            let json = serde_json::to_string(s).expect("serialize");
            let back: RollbackStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, s);
        }
    }
}
