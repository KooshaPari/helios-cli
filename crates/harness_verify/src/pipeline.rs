// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Verification pipeline

use crate::error::Result;
use crate::result::{GateDetail, GateResult, VerificationResult};
use crate::runners::run_cargo_test;
use harness_spec::models::{Specification, VerificationRule};
use tracing::{debug, instrument};

/// Verification pipeline
pub struct VerificationPipeline {
    _runners: PipelineRunners,
}

impl Default for VerificationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationPipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self { _runners: PipelineRunners::default() }
    }

    /// Run verification for a spec
    #[instrument(skip(self, spec), fields(spec_id = %spec.spec.name, rules = spec.spec.verification.len()))]
    pub async fn verify(&self, spec: &Specification) -> Result<Vec<VerificationResult>> {
        let mut results = Vec::new();

        for rule in &spec.spec.verification {
            let result = self.run_verification(rule, &spec.spec.name).await?;
            results.push(result);
        }

        debug!(count = results.len(), "verification finished");
        Ok(results)
    }

    /// Run single verification
    #[instrument(skip(self, rule), fields(spec_id = %spec_id))]
    async fn run_verification(
        &self,
        rule: &VerificationRule,
        spec_id: &str,
    ) -> Result<VerificationResult> {
        match rule {
            VerificationRule::Test { name: _, timeout_seconds } => {
                // Default to cargo test
                let timeout = *timeout_seconds as u64;
                if timeout > 0 {
                    run_cargo_test(spec_id, timeout).await
                } else {
                    run_cargo_test(spec_id, 300).await // 5 min default
                }
            }
            VerificationRule::Security { scanner, critical_only: _ } => {
                // Placeholder for security scanning
                Ok(VerificationResult {
                    id: uuid::Uuid::new_v4(),
                    spec_id: spec_id.to_string(),
                    verification_type: crate::result::VerificationType::Security,
                    status: crate::result::VerificationStatus::Skipped,
                    started_at: chrono::Utc::now(),
                    completed_at: Some(chrono::Utc::now()),
                    duration_ms: 0,
                    output: format!("Security scanner '{}' not implemented yet", scanner),
                    errors: vec![],
                    metrics: Default::default(),
                })
            }
            VerificationRule::Performance { metric, threshold } => {
                // Placeholder for performance testing
                Ok(VerificationResult {
                    id: uuid::Uuid::new_v4(),
                    spec_id: spec_id.to_string(),
                    verification_type: crate::result::VerificationType::Performance,
                    status: crate::result::VerificationStatus::Skipped,
                    started_at: chrono::Utc::now(),
                    completed_at: Some(chrono::Utc::now()),
                    duration_ms: 0,
                    output: format!(
                        "Performance benchmark '{}' with threshold '{}' not implemented yet",
                        metric, threshold
                    ),
                    errors: vec![],
                    metrics: Default::default(),
                })
            }
            VerificationRule::Custom { command, expected_exit_code } => {
                // Run custom command
                let output =
                    tokio::process::Command::new("sh").args(["-c", command]).output().await?;

                let passed = output.status.code() == Some(*expected_exit_code);

                Ok(VerificationResult {
                    id: uuid::Uuid::new_v4(),
                    spec_id: spec_id.to_string(),
                    verification_type: crate::result::VerificationType::Custom,
                    status: if passed {
                        crate::result::VerificationStatus::Passed
                    } else {
                        crate::result::VerificationStatus::Failed
                    },
                    started_at: chrono::Utc::now(),
                    completed_at: Some(chrono::Utc::now()),
                    duration_ms: 0,
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    errors: vec![String::from_utf8_lossy(&output.stderr).to_string()],
                    metrics: Default::default(),
                })
            }
        }
    }

    /// Run verification gates
    #[instrument(skip(self, results, gates), fields(gates = gates.len()))]
    pub fn run_gates(
        &self,
        results: &[VerificationResult],
        gates: &[GateConfig],
    ) -> Vec<GateResult> {
        let mut gate_results = Vec::new();

        for gate in gates {
            let passed = self.evaluate_gate(gate, results);

            let details: Vec<GateDetail> = results
                .iter()
                .map(|r| {
                    let check_passed =
                        matches!(r.status, crate::result::VerificationStatus::Passed);
                    GateDetail {
                        check: format!("{:?}", r.verification_type),
                        passed: check_passed,
                        message: r.output.clone(),
                    }
                })
                .collect();

            gate_results.push(GateResult {
                name: gate.name.clone(),
                passed,
                message: if passed {
                    "All gates passed".to_string()
                } else {
                    "Gate check failed".to_string()
                },
                details,
            });
        }

        gate_results
    }

    fn evaluate_gate(&self, gate: &GateConfig, results: &[VerificationResult]) -> bool {
        match gate.criteria.as_str() {
            "all_passed" => results
                .iter()
                .all(|r| matches!(r.status, crate::result::VerificationStatus::Passed)),
            "any_passed" => results
                .iter()
                .any(|r| matches!(r.status, crate::result::VerificationStatus::Passed)),
            "no_failures" => !results
                .iter()
                .any(|r| matches!(r.status, crate::result::VerificationStatus::Failed)),
            _ => false,
        }
    }
}

/// Pipeline runners (extensible)
#[derive(Default)]
pub struct PipelineRunners {
    // Can add custom runners here
}

/// Gate configuration
#[derive(Debug, Clone)]
pub struct GateConfig {
    pub name: String,
    pub criteria: String,
    pub threshold: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{VerificationResult, VerificationStatus, VerificationType};
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_result(status: VerificationStatus) -> VerificationResult {
        VerificationResult {
            id: Uuid::new_v4(),
            spec_id: "demo-spec".to_string(),
            verification_type: VerificationType::Test,
            status,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            duration_ms: 1,
            output: "ok".to_string(),
            errors: vec![],
            metrics: Default::default(),
        }
    }

    #[test]
    fn run_gates_all_passed_requires_every_result_passed() {
        let pipeline = VerificationPipeline::new();
        let results = vec![
            sample_result(VerificationStatus::Passed),
            sample_result(VerificationStatus::Passed),
        ];
        let gates = vec![GateConfig {
            name: "all".to_string(),
            criteria: "all_passed".to_string(),
            threshold: None,
        }];

        let gate_results = pipeline.run_gates(&results, &gates);
        assert_eq!(gate_results.len(), 1);
        assert!(gate_results[0].passed);
    }

    #[test]
    fn run_gates_no_failures_allows_skipped() {
        let pipeline = VerificationPipeline::new();
        let results = vec![
            sample_result(VerificationStatus::Passed),
            sample_result(VerificationStatus::Skipped),
        ];
        let gates = vec![GateConfig {
            name: "no_failures".to_string(),
            criteria: "no_failures".to_string(),
            threshold: None,
        }];

        let gate_results = pipeline.run_gates(&results, &gates);
        assert!(gate_results[0].passed);
    }

    #[test]
    fn run_gates_unknown_criteria_fails() {
        let pipeline = VerificationPipeline::new();
        let results = vec![sample_result(VerificationStatus::Passed)];
        let gates = vec![GateConfig {
            name: "unknown".to_string(),
            criteria: "unsupported".to_string(),
            threshold: None,
        }];

        let gate_results = pipeline.run_gates(&results, &gates);
        assert!(!gate_results[0].passed);
    }

    #[tokio::test]
    async fn security_rule_is_skipped_with_message() {
        let pipeline = VerificationPipeline::new();
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "security-demo".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Security {
                    scanner: "trivy".to_string(),
                    critical_only: true,
                }],
                rollback: Default::default(),
                success_criteria: vec![],
                behavior: None,
                resources: None,
                metadata: Default::default(),
            },
        };

        let results = pipeline.verify(&spec).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, VerificationStatus::Skipped));
        assert!(results[0].output.contains("trivy"));
    }

    #[tokio::test]
    async fn performance_rule_is_skipped_with_message() {
        let pipeline = VerificationPipeline::new();
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "perf-demo".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Performance {
                    metric: "p95_latency".to_string(),
                    threshold: "250ms".to_string(),
                }],
                rollback: Default::default(),
                success_criteria: vec![],
                behavior: None,
                resources: None,
                metadata: Default::default(),
            },
        };

        let results = pipeline.verify(&spec).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, VerificationStatus::Skipped));
        assert!(results[0].output.contains("p95_latency"));
    }
}
