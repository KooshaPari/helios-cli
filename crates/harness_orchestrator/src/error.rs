// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Error types for orchestration
//!
//! [`OrchestratorError`] implements `From<std::io::Error>` so that I/O
//! failures (e.g. agent log reads, checkpoint persistence) bubble up with
//! `?` without bespoke `.map_err` plumbing at every call site.

use thiserror::Error;

/// Errors that can occur during orchestration
#[derive(Error, Debug)]
pub enum OrchestratorError {
    /// Task decomposition failed
    #[error("Task decomposition failed: {0}")]
    DecompositionError(String),

    /// Agent execution failed
    #[error("Agent execution failed: {0}")]
    ExecutionError(String),

    /// Agent not available
    #[error("Agent not available: {0}")]
    AgentNotAvailable(String),

    /// Queue error
    #[error("Queue error: {0}")]
    QueueError(String),

    /// Timeout
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// I/O error while reading/writing orchestration state
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for orchestration
pub type Result<T> = std::result::Result<T, OrchestratorError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Traces to: FR-HELIOS-IO-003
    /// `From<io::Error>` must produce the `Io` variant.
    #[test]
    fn from_io_error_maps_to_io_variant() {
        let io_err = std::io::Error::other("boom");
        let err: OrchestratorError = io_err.into();
        assert!(matches!(err, OrchestratorError::Io(_)));
    }

    /// Traces to: FR-HELIOS-IO-003
    /// The `Display` impl must include the underlying I/O error message.
    #[test]
    fn io_error_display_includes_message() {
        let io_err = std::io::Error::other("agent-store closed");
        let err: OrchestratorError = io_err.into();
        assert!(err.to_string().contains("agent-store closed"));
    }
}
