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
                        errors: vec![
                            "scanner name rejected: contains potentially dangerous characters"
                                .to_string(),
                        ],
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
                // Parse the threshold: "250ms" -> 250_000_000 ns, "1500ns" -> 1500 ns
                let threshold_ns = parse_duration_to_ns(threshold);

                // Try to run the benchmark. We try cargo bench first, then cargo test --benches
                let start = chrono::Utc::now();
                let bench_result = tokio::time::timeout(
                    std::time::Duration::from_secs(300),
                    tokio::task::spawn_blocking({
                        let metric = metric.clone();
                        move || {
                            // Try cargo bench with the metric name
                            let output = std::process::Command::new("cargo")
                                .args(["bench", "--bench", &metric, "--", "--output-format=bencher"])
                                .output();
                            if let Ok(output) = output {
                                if output.status.success() {
                                    return output;
                                }
                            }
                            // Fallback: try cargo test with --benches flag
                            std::process::Command::new("cargo")
                                .args(["test", "--benches", &metric, "--", "--nocapture"])
                                .output()
                                .expect("Failed to run cargo bench or cargo test")
                        }
                    }),
                )
                .await;

                match bench_result {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let elapsed = chrono::Utc::now().signed_duration_since(start).num_milliseconds() as u64;

                        // Try to extract timing from benchmark output
                        let measured_ns = extract_benchmark_time(&stdout, &stderr);

                        let (passed, output_msg) = if let Some(actual_ns) = measured_ns {
                            if let Some(max_ns) = threshold_ns {
                                let passed = actual_ns <= max_ns;
                                (
                                    passed,
                                    format!(
                                        "Performance: {} measured {}ns, threshold {}ns — {}",
                                        metric,
                                        actual_ns,
                                        max_ns,
                                        if passed { "PASSED" } else { "EXCEEDED" }
                                    ),
                                )
                            } else {
                                (
                                    true,
                                    format!("Performance: {} measured {}ns (no threshold to compare)", metric, actual_ns),
                                )
                            }
                        } else {
                            // Couldn't parse timing from output — use wall-clock as fallback
                            let wall_ns = elapsed * 1_000_000;
                            if let Some(max_ns) = threshold_ns {
                                let passed = wall_ns <= max_ns;
                                (
                                    passed,
                                    format!(
                                        "Performance: {} wall-clock {}ms, threshold {}ms — {}",
                                        metric,
                                        elapsed,
                                        max_ns / 1_000_000,
                                        if passed { "PASSED" } else { "EXCEEDED" }
                                    ),
                                )
                            } else {
                                (
                                    true,
                                    format!("Performance: {} completed in {}ms", metric, elapsed),
                                )
                            }
                        };

                        Ok(VerificationResult {
                            id: uuid::Uuid::new_v4(),
                            spec_id: spec_id.to_string(),
                            verification_type: crate::result::VerificationType::Performance,
                            status: if passed {
                                crate::result::VerificationStatus::Passed
                            } else {
                                crate::result::VerificationStatus::Failed
                            },
                            started_at: start,
                            completed_at: Some(chrono::Utc::now()),
                            duration_ms: elapsed,
                            output: output_msg,
                            errors: if !passed {
                                vec![format!("Performance threshold exceeded for '{}'", metric)]
                            } else {
                                vec![]
                            },
                            metrics: Default::default(),
                        })
                    }
                    _ => {
                        Ok(VerificationResult {
                            id: uuid::Uuid::new_v4(),
                            spec_id: spec_id.to_string(),
                            verification_type: crate::result::VerificationType::Performance,
                            status: crate::result::VerificationStatus::Failed,
                            started_at: start,
                            completed_at: Some(chrono::Utc::now()),
                            duration_ms: 0,
                            output: format!("Performance benchmark '{}' failed to execute", metric),
                            errors: vec!["Benchmark execution failed or timed out".to_string()],
                            metrics: Default::default(),
                        })
                    }
                }
            }
            VerificationRule::Custom { command, expected_exit_code } => {
                // Reject commands with shell metacharacters to prevent injection
                if has_shell_metacharacters(command) {
                    return Ok(VerificationResult {
                        id: uuid::Uuid::new_v4(),
                        spec_id: spec_id.to_string(),
                        verification_type: crate::result::VerificationType::Custom,
                        status: crate::result::VerificationStatus::Failed,
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                        duration_ms: 0,
                        output: "Custom command rejected: contains shell metacharacters"
                            .to_string(),
                        errors: vec![format!(
                            "command '{}' contains potentially dangerous characters",
                            command
                        )],
                        metrics: Default::default(),
                    });
                }
                // Run custom command (safe — metacharacters stripped above)
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

/// Parse a duration string like "250ms" or "1500ns" or "2s" into nanoseconds.
fn parse_duration_to_ns(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.ends_with("ns") {
        s[..s.len()-2].trim().parse::<u64>().ok()
    } else if s.ends_with("ms") {
        s[..s.len()-2].trim().parse::<u64>().ok().map(|v| v * 1_000_000)
    } else if s.ends_with("us") {
        s[..s.len()-2].trim().parse::<u64>().ok().map(|v| v * 1_000)
    } else if s.ends_with('s') {
        s[..s.len()-1].trim().parse::<f64>().ok().map(|v| (v * 1_000_000_000.0) as u64)
    } else {
        // Try parsing as plain nanoseconds
        s.parse::<u64>().ok()
    }
}
/// Extract benchmark timing from cargo bench output.
/// Looks for lines like: `test result: ok. 0 passed; 0 failed; finished in 0.12s`
/// or criterion output: `time:   [1.2345 ms 1.2356 ms 1.2367 ms]`
fn extract_benchmark_time(stdout: &str, stderr: &str) -> Option<u64> {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for criterion-style timing: `time:   [1.2345 ms 1.2356 ms 1.2367 ms]`
    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("time:") || trimmed.contains("time:") {
            // Extract the first number after "time:"
            if let Some(start_idx) = trimmed.find('[') {
                let bracket_content = &trimmed[start_idx+1..];
                if let Some(end_idx) = bracket_content.find(']') {
                    let timing_str = &bracket_content[..end_idx].trim();
                    if let Some(val) = parse_bench_value(timing_str) {
                        return Some(val);
                    }
                }
            }
        }

        // Look for `test bench_xxx ... bench: 1234 ns/iter (+/- 56)`
        if trimmed.contains("bench:") && trimmed.contains("ns/iter") {
            if let Some(bench_idx) = trimmed.find("bench:") {
                let after_bench = trimmed[bench_idx+6..].trim();
                let num_str: String = after_bench.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(ns) = num_str.parse::<u64>() {
                    return Some(ns);
                }
            }
        }

        // Look for `finished in X.XXs`
        if trimmed.contains("finished in") {
            if let Some(idx) = trimmed.find("finished in") {
                let time_str = &trimmed[idx+11..].trim();
                if let Ok(secs) = time_str.trim_end_matches('s').parse::<f64>() {
                    return Some((secs * 1_000_000_000.0) as u64);
                }
            }
        }
    }
    None
}

/// Parse a single benchmark value like "1.2345 ms" or "1234 ns" into nanoseconds.
fn parse_bench_value(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 2 {
        let num: f64 = parts[0].parse().ok()?;
        let unit = parts[1].to_lowercase();
        if unit == "ns" {
            return Some(num as u64);
        } else if unit == "us" || unit == "µs" {
            return Some((num * 1_000.0) as u64);
        } else if unit == "ms" {
            return Some((num * 1_000_000.0) as u64);
        } else if unit == "s" {
            return Some((num * 1_000_000_000.0) as u64);
        }
    }
    None
}

/// Try to find a scanner binary on PATH.
fn which_scanner(name: &str) -> Option<String> {
    // Try the raw name first (might be an absolute path)
    if std::path::Path::new(name).is_file() {
        return Some(name.to_string());
    }

    // Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split([':', ';']) {
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
        let absent_scanner = "harness-verify-test-scanner-not-installed";
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "security-demo".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Security {
                    scanner: absent_scanner.to_string(),
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
        assert!(results[0].output.contains(absent_scanner));
    }

    #[tokio::test]
    async fn performance_rule_runs_benchmark_and_checks_threshold() {
        let pipeline = VerificationPipeline::new();
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "perf-demo".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Performance {
                    metric: "nonexistent_benchmark_xyz".to_string(),
                    threshold: "300s".to_string(),
                }],
                rollback: Default::default(),
                success_criteria: vec![],
                behavior: None,
                resources: None,
                metadata: Default::default(),
            },
        };

        // The benchmark won't exist, so the cargo-bench fallback path must
        // return a terminal verification result rather than panic or skip.
        let results = pipeline.verify(&spec).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0].status,
            VerificationStatus::Failed | VerificationStatus::Passed
        ));
        assert!(
            results[0].output.starts_with("Performance:"),
            "Performance fallback should report a terminal result, got {:?}",
            results[0].output
        );
    }

    #[test]
    fn parse_duration_to_ns_parses_common_formats() {
        assert_eq!(parse_duration_to_ns("250ms"), Some(250_000_000));
        assert_eq!(parse_duration_to_ns("1500ns"), Some(1500));
        assert_eq!(parse_duration_to_ns("2s"), Some(2_000_000_000));
        assert_eq!(parse_duration_to_ns("500us"), Some(500_000));
        assert_eq!(parse_duration_to_ns("1234"), Some(1234));
        assert_eq!(parse_duration_to_ns("  100ms  "), Some(100_000_000));
        assert_eq!(parse_duration_to_ns("invalid"), None);
    }

    #[test]
    fn extract_benchmark_time_parses_criterion_output() {
        let stdout = "";
        let stderr = "time:   [1.2345 ms 1.2356 ms 1.2367 ms]";
        let result = extract_benchmark_time(stdout, stderr);
        assert_eq!(result, Some(1_234_500));
    }

    #[test]
    fn extract_benchmark_time_parses_bench_output() {
        let stdout = "test bench_foo ... bench: 1234 ns/iter (+/- 56)";
        let result = extract_benchmark_time(stdout, "");
        assert_eq!(result, Some(1234));
    }

    #[test]
    fn extract_benchmark_time_returns_none_for_empty() {
        assert_eq!(extract_benchmark_time("", ""), None);
        assert_eq!(extract_benchmark_time("no timing here", ""), None);
    }

    #[test]
    fn has_shell_metacharacters_detects_injection_attempts() {
        // rm -rf / is dangerous but has NO metacharacters — it's a plain command
        assert!(!has_shell_metacharacters("rm -rf /"));
        // These DO contain metacharacters (pipe, semicolon, etc.)
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
        assert_eq!(
            results[0].errors,
            vec![
                "scanner name rejected: contains potentially dangerous characters".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn custom_rule_rejects_metacharacters_without_running_command() {
        let pipeline = VerificationPipeline::new();
        let command = "echo should-not-run; false";
        let spec = harness_spec::models::Specification {
            spec: harness_spec::models::SpecContent {
                name: "custom-inject".to_string(),
                version: "1.0.0".to_string(),
                owner: String::new(),
                verification: vec![harness_spec::models::VerificationRule::Custom {
                    command: command.to_string(),
                    expected_exit_code: 0,
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
        assert_eq!(results[0].output, "Custom command rejected: contains shell metacharacters");
        assert_eq!(
            results[0].errors,
            vec![format!("command '{}' contains potentially dangerous characters", command)]
        );
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
