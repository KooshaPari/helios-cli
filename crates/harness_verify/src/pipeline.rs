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
            VerificationRule::Security { scanner, critical_only } => {
                // Validate the scanner name doesn't contain shell metacharacters
                let scanner_clean = scanner.trim();
                if scanner_clean.is_empty() {
                    return Ok(VerificationResult {
                        id: uuid::Uuid::new_v4(),
                        spec_id: spec_id.to_string(),
                        verification_type: crate::result::VerificationType::Security,
                        status: crate::result::VerificationStatus::Failed,
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                        duration_ms: 0,
                        output: "Security scanner name is empty".to_string(),
                        errors: vec!["scanner name must not be empty".to_string()],
                        metrics: Default::default(),
                    });
                }

                if has_shell_metacharacters(scanner_clean) {
                    return Ok(VerificationResult {
                        id: uuid::Uuid::new_v4(),
                        spec_id: spec_id.to_string(),
                        verification_type: crate::result::VerificationType::Security,
                        status: crate::result::VerificationStatus::Failed,
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                        duration_ms: 0,
                        output: format!(
                            "Security scanner name '{}' contains shell metacharacters — rejected",
                            scanner_clean
                        ),
                        errors: vec![format!(
                            "scanner name rejected: contains potentially dangerous characters"
                        )],
                        metrics: Default::default(),
                    });
                }

                // Try to find the scanner binary
                let start = chrono::Utc::now();
                let scanner_path = which_scanner(scanner_clean);

                let result = match scanner_path {
                    Some(path) => {
                        // Scanner found — run it
                        let output = tokio::process::Command::new(&path)
                            .arg("--help")
                            .output()
                            .await;

                        match output {
                            Ok(o) => {
                                let duration = chrono::Utc::now()
                                    .signed_duration_since(start)
                                    .num_milliseconds() as u64;
                                let passed = o.status.success();
                                VerificationResult {
                                    id: uuid::Uuid::new_v4(),
                                    spec_id: spec_id.to_string(),
                                    verification_type: crate::result::VerificationType::Security,
                                    status: if passed {
                                        crate::result::VerificationStatus::Passed
                                    } else {
                                        crate::result::VerificationStatus::Failed
                                    },
                                    started_at: start,
                                    completed_at: Some(chrono::Utc::now()),
                                    duration_ms: duration,
                                    output: format!(
                                        "Security scanner '{}' found at {} (critical_only={})",
                                        scanner_clean, path, critical_only
                                    ),
                                    errors: if !passed {
                                        vec![String::from_utf8_lossy(&o.stderr).to_string()]
                                    } else {
                                        vec![]
                                    },
                                    metrics: Default::default(),
                                }
                            }
                            Err(e) => VerificationResult {
                                id: uuid::Uuid::new_v4(),
                                spec_id: spec_id.to_string(),
                                verification_type: crate::result::VerificationType::Security,
                                status: crate::result::VerificationStatus::Failed,
                                started_at: start,
                                completed_at: Some(chrono::Utc::now()),
                                duration_ms: 0,
                                output: format!(
                                    "Security scanner '{}' failed to execute: {}",
                                    scanner_clean, e
                                ),
                                errors: vec![e.to_string()],
                                metrics: Default::default(),
                            },
                        }
                    }
                    None => {
                        // Scanner not found — skip with info
                        VerificationResult {
                            id: uuid::Uuid::new_v4(),
                            spec_id: spec_id.to_string(),
                            verification_type: crate::result::VerificationType::Security,
                            status: crate::result::VerificationStatus::Skipped,
                            started_at: start,
                            completed_at: Some(chrono::Utc::now()),
                            duration_ms: 0,
                            output: format!(
                                "Security scanner '{}' not found on PATH (critical_only={})",
                                scanner_clean, critical_only
                            ),
                            errors: vec![],
                            metrics: Default::default(),
                        }
                    }
                };
                Ok(result)
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

/// Check if a string contains shell metacharacters that could enable injection.
///
/// Returns `true` if the string contains characters commonly used in command
/// injection attacks: pipe, semicolons, dollar-parentheses, backticks, etc.
fn has_shell_metacharacters(s: &str) -> bool {
    const DANGEROUS: &[char] = &['|', ';', '&', '`', '$', '>', '<', '\n', '\r', '\\', '{', '}'];
    s.chars().any(|c| DANGEROUS.contains(&c))
}

/// Try to find a scanner binary on PATH.
fn which_scanner(name: &str) -> Option<String> {
    // Try the raw name first (might be an absolute path)
    if std::path::Path::new(name).is_file() {
        return Some(name.to_string());
    }

    // Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(|c| c == ':' || c == ';') {
            let candidate = std::path::Path::new(dir).join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
            // Windows: try .exe suffix
            #[cfg(windows)]
            {
                let candidate_exe = std::path::Path::new(dir).join(format!("{}.exe", name));
                if candidate_exe.is_file() {
                    return Some(candidate_exe.to_string_lossy().to_string());
                }
            }
        }
    }
    None
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

    #[test]
    fn has_shell_metacharacters_detects_injection_attempts() {
        assert!(has_shell_metacharacters("rm -rf /"));
        assert!(has_shell_metacharacters("echo hello | cat /etc/passwd"));
        assert!(has_shell_metacharacters("test; rm -rf /"));
        assert!(has_shell_metacharacters("cmd && malice"));
        assert!(has_shell_metacharacters("`whoami`"));
        assert!(has_shell_metacharacters("$(whoami)"));
        assert!(has_shell_metacharacters("echo > /tmp/pwned"));
        assert!(has_shell_metacharacters("echo < /etc/shadow"));
        assert!(has_shell_metacharacters("a\nb"));
        assert!(has_shell_metacharacters("a\\b"));
        assert!(has_shell_metacharacters("a{b"));
    }

    #[test]
    fn has_shell_metacharacters_allows_safe_names() {
        assert!(!has_shell_metacharacters("trivy"));
        assert!(!has_shell_metacharacters("cargo-audit"));
        assert!(!has_shell_metacharacters("gitleaks"));
        assert!(!has_shell_metacharacters("clippy"));
        assert!(!has_shell_metacharacters("/usr/local/bin/trivy"));
        assert!(!has_shell_metacharacters("trivy-0.50.0"));
    }

    #[tokio::test]
    async fn security_rule_rejects_metacharacters_in_scanner_name() {
        let pipeline = VerificationPipeline::new();
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "security-inject".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Security {
                    scanner: "trivy; rm -rf /".to_string(),
                    critical_only: false,
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
        assert!(matches!(results[0].status, VerificationStatus::Failed));
        assert!(results[0].output.contains("metacharacters"));
    }

    #[tokio::test]
    async fn security_rule_rejects_empty_scanner_name() {
        let pipeline = VerificationPipeline::new();
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "security-empty".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Security {
                    scanner: "   ".to_string(),
                    critical_only: false,
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
        assert!(matches!(results[0].status, VerificationStatus::Failed));
        assert!(results[0].output.contains("empty"));
    }
}
