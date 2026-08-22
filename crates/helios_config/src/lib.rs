// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Centralized configuration for HeliosCLI.
//!
//! Consolidates all magic numbers, default timeouts, ports, paths, and
//! thresholds that were previously hardcoded across individual crates into
//! a single `HeliosConfig` struct loaded from:
//!
//! 1. A config file (`helios.toml` or `helios.yaml` in the project root or
//!    a path pointed to by `HELIOS_CONFIG_PATH`).
//! 2. Environment variables prefixed with `HELIOS_` (e.g. `HELIOS_CACHE_TTL`).
//! 3. Sensible hardcoded defaults.
//!
//! # Example
//!
//! ```rust
//! use helios_config::HeliosConfig;
//!
//! let config = HeliosConfig::default();
//! assert_eq!(config.cache.max_capacity, 10_000);
//! assert_eq!(config.runner.timeout_secs, 30);
//! ```

use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("failed to read config file {path}: {inner}")]
    FileRead { path: PathBuf, inner: std::io::Error },

    /// The config file could not be parsed.
    #[error("failed to parse config file {path}: {inner}")]
    FileParse { path: PathBuf, inner: serde_yaml::Error },

    /// An environment variable had an invalid value.
    #[error("invalid value for env var {var}: {inner}")]
    EnvVar { var: String, inner: String },
}

/// Result alias for config operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

// ---------------------------------------------------------------------------
// Top-level configuration
// ---------------------------------------------------------------------------

/// Top-level HeliosCLI configuration.
///
/// Each sub-struct maps to a domain that previously had hardcoded defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HeliosConfig {
    /// Cache configuration.
    pub cache: CacheConfig,

    /// Process runner configuration.
    pub runner: RunnerConfig,

    /// Auto-scaling configuration.
    pub scaling: ScalingConfig,

    /// Circuit breaker configuration.
    pub circuit_breaker: CircuitBreakerConfig,

    /// Teammate default configuration.
    pub teammate: TeammateConfig,

    /// Specification / rollback configuration.
    pub spec: SpecConfig,

    /// Checkpoint / git signature configuration.
    pub checkpoint: CheckpointConfig,

    /// Elicitation/classifier configuration.
    pub elicitation: ElicitationConfig,

    /// Verification pipeline configuration.
    pub verify: VerifyConfig,

    /// Predictive scaler configuration.
    pub predictive_scaler: PredictiveScalerConfig,

    /// Token bucket rate limiter configuration.
    pub token_bucket: TokenBucketConfig,
}

impl HeliosConfig {
    /// Load configuration from the default locations.
    ///
    /// Priority (highest wins):
    /// 1. Environment variables (prefix `HELIOS_`)
    /// 2. Config file (`helios.toml` or `helios.yaml`, or `HELIOS_CONFIG_PATH`)
    /// 3. Defaults
    pub fn load() -> Self {
        Self::load_from(None)
    }

    /// Load configuration, optionally specifying a config file path.
    pub fn load_from(config_path: Option<&std::path::Path>) -> Self {
        let mut config = HeliosConfig::default();

        // 1) Try to overlay from config file
        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(loaded) = Self::from_file(path) {
                    config = loaded;
                }
            }
        } else {
            // Try default paths
            let candidates = [
                PathBuf::from("helios.toml"),
                PathBuf::from("helios.yaml"),
                PathBuf::from("config/helios.toml"),
                PathBuf::from("config/helios.yaml"),
                PathBuf::from(".helios.toml"),
                PathBuf::from(".helios.yaml"),
            ];
            if let Ok(config_path_env) = env::var("HELIOS_CONFIG_PATH") {
                let p = PathBuf::from(&config_path_env);
                if p.exists() {
                    if let Ok(loaded) = Self::from_file(&p) {
                        config = loaded;
                    }
                }
            }
            for candidate in &candidates {
                if candidate.exists() {
                    if let Ok(loaded) = Self::from_file(candidate) {
                        config = loaded;
                    }
                    break;
                }
            }
        }

        // 2) Overlay from environment variables
        config.apply_env_overrides();

        config
    }

    /// Parse config from a TOML or YAML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead { path: path.to_owned(), inner: e })?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match ext.as_str() {
            "yaml" | "yml" => serde_yaml::from_str(&contents)
                .map_err(|e| ConfigError::FileParse { path: path.to_owned(), inner: e }),
            "toml" => toml::from_str(&contents).map_err(|e| ConfigError::FileParse {
                path: path.to_owned(),
                inner: serde_yaml::Error::custom(e.to_string()),
            }),
            _ => {
                // Try YAML first, then TOML
                serde_yaml::from_str(&contents).or_else(|_| {
                    toml::from_str(&contents).map_err(|e| ConfigError::FileParse {
                        path: path.to_owned(),
                        inner: serde_yaml::Error::custom(e.to_string()),
                    })
                })
            }
        }
    }

    /// Apply environment variable overrides.
    ///
    /// Variables follow the pattern `HELIOS_<SECTION>_<KEY>`.
    /// For example `HELIOS_CACHE_MAX_CAPACITY=5000`.
    fn apply_env_overrides(&mut self) {
        // Cache
        self.cache.max_capacity =
            env_override("HELIOS_CACHE_MAX_CAPACITY", self.cache.max_capacity);
        self.cache.ttl_secs = env_override("HELIOS_CACHE_TTL", self.cache.ttl_secs);

        // Runner
        self.runner.timeout_secs = env_override("HELIOS_RUNNER_TIMEOUT", self.runner.timeout_secs);

        // Scaling
        self.scaling.min_instances =
            env_override("HELIOS_SCALING_MIN_INSTANCES", self.scaling.min_instances);
        self.scaling.max_instances =
            env_override("HELIOS_SCALING_MAX_INSTANCES", self.scaling.max_instances);
        self.scaling.target_cpu_percent =
            env_override("HELIOS_SCALING_TARGET_CPU", self.scaling.target_cpu_percent);
        self.scaling.target_memory_percent =
            env_override("HELIOS_SCALING_TARGET_MEMORY", self.scaling.target_memory_percent);
        self.scaling.scale_up_threshold =
            env_override("HELIOS_SCALING_SCALE_UP", self.scaling.scale_up_threshold);
        self.scaling.scale_down_threshold =
            env_override("HELIOS_SCALING_SCALE_DOWN", self.scaling.scale_down_threshold);
        self.scaling.cooldown_secs =
            env_override("HELIOS_SCALING_COOLDOWN", self.scaling.cooldown_secs);

        // Circuit breaker
        self.circuit_breaker.failure_threshold =
            env_override("HELIOS_CB_FAILURE_THRESHOLD", self.circuit_breaker.failure_threshold);
        self.circuit_breaker.success_threshold =
            env_override("HELIOS_CB_SUCCESS_THRESHOLD", self.circuit_breaker.success_threshold);
        self.circuit_breaker.retry_timeout_secs =
            env_override("HELIOS_CB_RETRY_TIMEOUT", self.circuit_breaker.retry_timeout_secs);

        // Teammate
        self.teammate.max_concurrent =
            env_override("HELIOS_TEAMMATE_MAX_CONCURRENT", self.teammate.max_concurrent);
        self.teammate.timeout_secs =
            env_override("HELIOS_TEAMMATE_TIMEOUT", self.teammate.timeout_secs);

        // Spec / rollback
        self.spec.default_version =
            env_override_string("HELIOS_SPEC_VERSION", &self.spec.default_version);
        self.spec.default_timeout_secs =
            env_override("HELIOS_SPEC_TIMEOUT", self.spec.default_timeout_secs);

        // Checkpoint
        self.checkpoint.git_signature_name = env_override_string(
            "HELIOS_CHECKPOINT_SIGNATURE_NAME",
            &self.checkpoint.git_signature_name,
        );
        self.checkpoint.git_signature_email = env_override_string(
            "HELIOS_CHECKPOINT_SIGNATURE_EMAIL",
            &self.checkpoint.git_signature_email,
        );

        // Elicitation
        self.elicitation.confidence_threshold =
            env_override("HELIOS_ELICITATION_CONFIDENCE", self.elicitation.confidence_threshold);

        // Verification
        self.verify.test_timeout_secs =
            env_override("HELIOS_VERIFY_TEST_TIMEOUT", self.verify.test_timeout_secs);
        self.verify.smoke_test_timeout_secs =
            env_override("HELIOS_VERIFY_SMOKE_TIMEOUT", self.verify.smoke_test_timeout_secs);
    }
}

/// Parse an env var as `u64`, falling back to the default.
fn env_override<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    match env::var(key) {
        Ok(val) => val.parse().unwrap_or_else(|_| {
            tracing::warn!("invalid value for {} (using default)", key);
            default
        }),
        Err(_) => default,
    }
}

/// Parse an env var as `String`, falling back to the default.
fn env_override_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

// ---------------------------------------------------------------------------
// Cache configuration
// ---------------------------------------------------------------------------

/// Cache configuration defaults.
///
/// Previously hardcoded in:
/// - `crates/harness_cache/src/lib.rs` (CacheConfig)
/// - `crates/harness_cache/src/domain/mod.rs` (CachePolicy)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache.
    pub max_capacity: u64,
    /// Default TTL in seconds.
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { max_capacity: 10_000, ttl_secs: 300 }
    }
}

// ---------------------------------------------------------------------------
// Runner configuration
// ---------------------------------------------------------------------------

/// Runner configuration defaults.
///
/// Previously hardcoded in:
/// - `crates/harness_runner/src/lib.rs` (RunnerConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RunnerConfig {
    /// Default process timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

// ---------------------------------------------------------------------------
// Scaling configuration
// ---------------------------------------------------------------------------

/// Scaling configuration defaults.
///
/// Previously hardcoded in:
/// - `crates/harness_scaling/src/lib.rs` (ScalingConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScalingConfig {
    /// Minimum number of instances.
    pub min_instances: u32,
    /// Maximum number of instances.
    pub max_instances: u32,
    /// Target CPU usage percentage.
    pub target_cpu_percent: f64,
    /// Target memory usage percentage.
    pub target_memory_percent: f64,
    /// Scale-up threshold fraction (relative to target).
    pub scale_up_threshold: f64,
    /// Scale-down threshold fraction (relative to target).
    pub scale_down_threshold: f64,
    /// Cooldown period between scaling events in seconds.
    pub cooldown_secs: u64,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            target_cpu_percent: 50.0,
            target_memory_percent: 70.0,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            cooldown_secs: 60,
        }
    }
}

// ---------------------------------------------------------------------------
// Circuit breaker configuration
// ---------------------------------------------------------------------------

/// Circuit breaker configuration defaults.
///
/// Previously hardcoded in:
/// - `crates/harness_scaling/src/lib.rs` (CircuitBreaker)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to open the circuit.
    pub failure_threshold: u32,
    /// Number of consecutive successes in half-open state to close.
    pub success_threshold: u32,
    /// Seconds to wait before retrying after circuit opens.
    pub retry_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self { failure_threshold: 5, success_threshold: 3, retry_timeout_secs: 30 }
    }
}

// ---------------------------------------------------------------------------
// Teammate configuration
// ---------------------------------------------------------------------------

/// Teammate default configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_teammates/src/domain/mod.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TeammateConfig {
    /// Default max concurrent tasks per teammate.
    pub max_concurrent: usize,
    /// Default teammate timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for TeammateConfig {
    fn default() -> Self {
        Self { max_concurrent: 1, timeout_secs: 300 }
    }
}

// ---------------------------------------------------------------------------
// Specification configuration
// ---------------------------------------------------------------------------

/// Specification / rollback default configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_spec/src/models.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpecConfig {
    /// Default version for new specs.
    pub default_version: String,
    /// Default rollback timeout in seconds.
    pub default_timeout_secs: u32,
}

impl Default for SpecConfig {
    fn default() -> Self {
        Self { default_version: "1.0.0".to_string(), default_timeout_secs: 30 }
    }
}

// ---------------------------------------------------------------------------
// Checkpoint configuration
// ---------------------------------------------------------------------------

/// Checkpoint / git signature configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_checkpoint/src/git.rs`
/// - `crates/harness_checkpoint/src/config.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckpointConfig {
    /// Git commit author name for checkpoints.
    pub git_signature_name: String,
    /// Git commit author email for checkpoints.
    pub git_signature_email: String,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            git_signature_name: "heliosHarness".to_string(),
            git_signature_email: "checkpoint@helios.local".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Elicitation configuration
// ---------------------------------------------------------------------------

/// Elicitation / intent classification configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_elicitation/src/generator.rs`
/// - `crates/harness_elicitation/src/classifier.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ElicitationConfig {
    /// Minimum confidence threshold for intent classification.
    pub confidence_threshold: f64,
}

impl Default for ElicitationConfig {
    fn default() -> Self {
        Self { confidence_threshold: 0.1 }
    }
}

// ---------------------------------------------------------------------------
// Verification configuration
// ---------------------------------------------------------------------------

/// Verification pipeline configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_verify/src/pipeline.rs`
/// - `crates/harness_verify/src/runners.rs`
/// - `crates/harness_elicitation/src/generator.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VerifyConfig {
    /// Default timeout for test verification in seconds.
    pub test_timeout_secs: u64,
    /// Default timeout for smoke tests in seconds.
    pub smoke_test_timeout_secs: u64,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self { test_timeout_secs: 300, smoke_test_timeout_secs: 60 }
    }
}

// ---------------------------------------------------------------------------
// Predictive scaler configuration
// ---------------------------------------------------------------------------

/// Predictive scaler configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_scaling/src/lib.rs` (PredictiveScaler)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PredictiveScalerConfig {
    /// Maximum number of history samples to retain.
    pub max_history: usize,
    /// Default prediction horizon (steps ahead).
    pub prediction_horizon: usize,
}

impl Default for PredictiveScalerConfig {
    fn default() -> Self {
        Self { max_history: 100, prediction_horizon: 5 }
    }
}

// ---------------------------------------------------------------------------
// Token bucket configuration
// ---------------------------------------------------------------------------

/// Token bucket rate limiter configuration.
///
/// Previously hardcoded in:
/// - `crates/harness_scaling/src/lib.rs` (TokenBucket)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenBucketConfig {
    /// Default bucket capacity.
    pub default_capacity: f64,
    /// Default refill rate (tokens per second).
    pub default_refill_rate: f64,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self { default_capacity: 100.0, default_refill_rate: 10.0 }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the default configuration has expected values.
    #[test]
    fn test_default_config_values() {
        let config = HeliosConfig::default();

        // Cache
        assert_eq!(config.cache.max_capacity, 10_000);
        assert_eq!(config.cache.ttl_secs, 300);

        // Runner
        assert_eq!(config.runner.timeout_secs, 30);

        // Scaling
        assert_eq!(config.scaling.min_instances, 1);
        assert_eq!(config.scaling.max_instances, 10);
        assert!((config.scaling.target_cpu_percent - 50.0).abs() < f64::EPSILON);

        // Circuit breaker
        assert_eq!(config.circuit_breaker.failure_threshold, 5);
        assert_eq!(config.circuit_breaker.success_threshold, 3);

        // Teammate
        assert_eq!(config.teammate.max_concurrent, 1);
        assert_eq!(config.teammate.timeout_secs, 300);

        // Spec
        assert_eq!(config.spec.default_version, "1.0.0");
        assert_eq!(config.spec.default_timeout_secs, 30);

        // Checkpoint
        assert_eq!(config.checkpoint.git_signature_name, "heliosHarness");
        assert_eq!(config.checkpoint.git_signature_email, "checkpoint@helios.local");

        // Elicitation
        assert!((config.elicitation.confidence_threshold - 0.1).abs() < f64::EPSILON);

        // Verify
        assert_eq!(config.verify.test_timeout_secs, 300);
        assert_eq!(config.verify.smoke_test_timeout_secs, 60);
    }

    /// Verify that the config can be serialized and deserialized (round-trip).
    #[test]
    fn test_config_roundtrip_yaml() {
        let config = HeliosConfig::default();
        let yaml = serde_yaml::to_string(&config).expect("serialize to yaml");
        let deserialized: HeliosConfig =
            serde_yaml::from_str(&yaml).expect("deserialize from yaml");

        assert_eq!(deserialized.cache.max_capacity, config.cache.max_capacity);
        assert_eq!(deserialized.runner.timeout_secs, config.runner.timeout_secs);
        assert_eq!(
            deserialized.checkpoint.git_signature_name,
            config.checkpoint.git_signature_name
        );
    }

    /// Verify partial overlay from YAML preserves defaults for unspecified fields.
    #[test]
    fn test_config_partial_overlay() {
        let partial_yaml = r#"
cache:
  max_capacity: 5000
runner:
  timeout_secs: 60
"#;
        let partial: HeliosConfig =
            serde_yaml::from_str(partial_yaml).expect("deserialize partial config");

        // Overridden values
        assert_eq!(partial.cache.max_capacity, 5000);
        assert_eq!(partial.runner.timeout_secs, 60);

        // Defaults preserved
        assert_eq!(partial.cache.ttl_secs, 300);
        assert_eq!(partial.scaling.min_instances, 1);
        assert_eq!(partial.teammate.timeout_secs, 300);
    }

    /// Verify that config file loading from a YAML file works.
    #[test]
    fn test_config_from_yaml_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("helios_test_config.yaml");
        let yaml_content = r#"
cache:
  max_capacity: 7777
  ttl_secs: 600
runner:
  timeout_secs: 120
"#;
        std::fs::write(&path, yaml_content).expect("write test config");

        let config = HeliosConfig::from_file(&path).expect("load from file");
        assert_eq!(config.cache.max_capacity, 7777);
        assert_eq!(config.cache.ttl_secs, 600);
        assert_eq!(config.runner.timeout_secs, 120);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_config_from_toml_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("helios_test_config.toml");
        let toml_content = r#"
[cache]
max_capacity = 4242
[runner]
timeout_secs = 99
"#;
        std::fs::write(&path, toml_content).expect("write test config");

        let config = HeliosConfig::from_file(&path).expect("load from toml");
        assert_eq!(config.cache.max_capacity, 4242);
        assert_eq!(config.runner.timeout_secs, 99);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_from_explicit_path_overrides_defaults() {
        let dir = std::env::temp_dir();
        let path = dir.join("helios_load_from_test.yaml");
        let yaml_content = r#"
cache:
  max_capacity: 1111
"#;
        std::fs::write(&path, yaml_content).expect("write test config");

        let config = HeliosConfig::load_from(Some(&path));
        assert_eq!(config.cache.max_capacity, 1111);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_env_overrides_apply_on_load() {
        let key = "HELIOS_CACHE_MAX_CAPACITY";
        let prior = env::var(key).ok();
        env::set_var(key, "9090");

        let config = HeliosConfig::load();
        assert_eq!(config.cache.max_capacity, 9090);

        match prior {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }

    // ------------------------------------------------------------------
    // Additional tests
    // ------------------------------------------------------------------

    /// ConfigError: FileRead variant displays path and message.
    #[test]
    fn config_error_file_read_display() {
        let err = ConfigError::FileRead {
            path: PathBuf::from("/bad/config.yaml"),
            inner: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let msg = err.to_string();
        assert!(msg.contains("/bad/config.yaml"));
        assert!(msg.contains("not found"));
    }

    /// ConfigError: EnvVar variant displays variable name.
    #[test]
    fn config_error_env_var_display() {
        let err = ConfigError::EnvVar {
            var: "HELIOS_FOO".into(),
            inner: "not a number".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("HELIOS_FOO"));
        assert!(msg.contains("not a number"));
    }

    /// load_from with non-existent path falls back to defaults.
    #[test]
    fn load_from_nonexistent_path_uses_defaults() {
        let path = std::path::Path::new("/nonexistent/helios_config_test.yaml");
        let config = HeliosConfig::load_from(Some(path));
        assert_eq!(config.cache.max_capacity, 10_000);
        assert_eq!(config.runner.timeout_secs, 30);
    }

    /// load_from with None path uses defaults when no config files exist.
    #[test]
    fn load_from_none_uses_defaults() {
        // Save and clear HELIOS_CONFIG_PATH to ensure defaults
        let prior = env::var("HELIOS_CONFIG_PATH").ok();
        env::remove_var("HELIOS_CONFIG_PATH");
        let config = HeliosConfig::load_from(None);
        assert_eq!(config.cache.max_capacity, 10_000);
        if let Some(value) = prior {
            env::set_var("HELIOS_CONFIG_PATH", value);
        }
    }

    /// Config round-trip through TOML serialization.
    #[test]
    fn test_config_roundtrip_toml() {
        let config = HeliosConfig::default();
        let toml_str = toml::to_string(&config).expect("serialize to toml");
        let deserialized: HeliosConfig = toml::from_str(&toml_str).expect("deserialize from toml");
        assert_eq!(deserialized.cache.max_capacity, config.cache.max_capacity);
        assert_eq!(deserialized.runner.timeout_secs, config.runner.timeout_secs);
        assert_eq!(
            deserialized.checkpoint.git_signature_email,
            config.checkpoint.git_signature_email
        );
    }

    /// Sub-config: scaling defaults are sensible.
    #[test]
    fn test_scaling_config_defaults() {
        let cfg = ScalingConfig::default();
        assert_eq!(cfg.min_instances, 1);
        assert_eq!(cfg.max_instances, 10);
        assert!((cfg.target_cpu_percent - 50.0).abs() < f64::EPSILON);
        assert!((cfg.target_memory_percent - 70.0).abs() < f64::EPSILON);
        assert!((cfg.scale_up_threshold - 0.8).abs() < f64::EPSILON);
        assert!((cfg.scale_down_threshold - 0.3).abs() < f64::EPSILON);
        assert_eq!(cfg.cooldown_secs, 60);
    }

    /// Sub-config: predictive scaler defaults.
    #[test]
    fn test_predictive_scaler_defaults() {
        let cfg = PredictiveScalerConfig::default();
        assert_eq!(cfg.max_history, 100);
        assert_eq!(cfg.prediction_horizon, 5);
    }

    /// Sub-config: token bucket defaults.
    #[test]
    fn test_token_bucket_defaults() {
        let cfg = TokenBucketConfig::default();
        assert!((cfg.default_capacity - 100.0).abs() < f64::EPSILON);
        assert!((cfg.default_refill_rate - 10.0).abs() < f64::EPSILON);
    }

    /// Sub-config: elicitation defaults.
    #[test]
    fn test_elicitation_config_defaults() {
        let cfg = ElicitationConfig::default();
        assert!((cfg.confidence_threshold - 0.1).abs() < f64::EPSILON);
    }

    /// Sub-config: verify config defaults.
    #[test]
    fn test_verify_config_defaults() {
        let cfg = VerifyConfig::default();
        assert_eq!(cfg.test_timeout_secs, 300);
        assert_eq!(cfg.smoke_test_timeout_secs, 60);
    }

    /// Sub-config: circuit breaker defaults.
    #[test]
    fn test_circuit_breaker_config_defaults() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.failure_threshold, 5);
        assert_eq!(cfg.success_threshold, 3);
        assert_eq!(cfg.retry_timeout_secs, 30);
    }

    /// Sub-config: teammate defaults.
    #[test]
    fn test_teammate_config_defaults() {
        let cfg = TeammateConfig::default();
        assert_eq!(cfg.max_concurrent, 1);
        assert_eq!(cfg.timeout_secs, 300);
    }

    /// Sub-config: spec defaults.
    #[test]
    fn test_spec_config_defaults() {
        let cfg = SpecConfig::default();
        assert_eq!(cfg.default_version, "1.0.0");
        assert_eq!(cfg.default_timeout_secs, 30);
    }

    /// Sub-config: checkpoint defaults.
    #[test]
    fn test_checkpoint_config_defaults() {
        let cfg = CheckpointConfig::default();
        assert_eq!(cfg.git_signature_name, "heliosHarness");
        assert_eq!(cfg.git_signature_email, "checkpoint@helios.local");
    }

    /// Env override: string fields (checkpoint signature).
    #[test]
    fn test_env_override_string_fields() {
        let name_key = "HELIOS_CHECKPOINT_SIGNATURE_NAME";
        let email_key = "HELIOS_CHECKPOINT_SIGNATURE_EMAIL";
        let prior_name = env::var(name_key).ok();
        let prior_email = env::var(email_key).ok();
        env::set_var(name_key, "test-author");
        env::set_var(email_key, "test@example.com");

        let config = HeliosConfig::load();
        assert_eq!(config.checkpoint.git_signature_name, "test-author");
        assert_eq!(config.checkpoint.git_signature_email, "test@example.com");

        match prior_name {
            Some(v) => env::set_var(name_key, v),
            None => env::remove_var(name_key),
        }
        match prior_email {
            Some(v) => env::set_var(email_key, v),
            None => env::remove_var(email_key),
        }
    }

    /// Env override: runner timeout.
    #[test]
    fn test_env_override_runner_timeout() {
        let key = "HELIOS_RUNNER_TIMEOUT";
        let prior = env::var(key).ok();
        env::set_var(key, "120");

        let config = HeliosConfig::load();
        assert_eq!(config.runner.timeout_secs, 120);

        match prior {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }

    /// Config: partial YAML overlay preserves non-specified sub-configs.
    #[test]
    fn test_partial_overlay_preserves_all_subconfigs() {
        let yaml = r#"
cache:
  max_capacity: 123
"#;
        let partial: HeliosConfig = serde_yaml::from_str(yaml).expect("parse");
        // Overridden
        assert_eq!(partial.cache.max_capacity, 123);
        // All sub-configs should have their defaults
        assert_eq!(partial.scaling.min_instances, 1);
        assert_eq!(partial.circuit_breaker.failure_threshold, 5);
        assert_eq!(partial.teammate.timeout_secs, 300);
        assert_eq!(partial.spec.default_version, "1.0.0");
        assert_eq!(partial.predictive_scaler.max_history, 100);
        assert!((partial.token_bucket.default_capacity - 100.0).abs() < f64::EPSILON);
    }

    /// Config: from_file with invalid YAML returns error.
    #[test]
    fn test_config_from_invalid_yaml() {
        let dir = std::env::temp_dir();
        let path = dir.join("helios_test_invalid_config.yaml");
        std::fs::write(&path, "{{{{invalid yaml!!!").expect("write");
        let result = HeliosConfig::from_file(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    /// Config: from_file with non-existent file returns FileRead error.
    #[test]
    fn test_config_from_nonexistent_file() {
        let path = std::path::Path::new("/no/such/helios_test_file.toml");
        let result = HeliosConfig::from_file(path);
        assert!(matches!(result, Err(ConfigError::FileRead { .. })));
    }

    /// HeliosConfig: clone produces independent copy.
    #[test]
    fn test_config_clone() {
        let mut config = HeliosConfig::default();
        config.cache.max_capacity = 42;
        let cloned = config.clone();
        assert_eq!(cloned.cache.max_capacity, 42);
        // Mutating original does not affect clone
        config.cache.max_capacity = 99;
        assert_eq!(cloned.cache.max_capacity, 42);
    }
}
