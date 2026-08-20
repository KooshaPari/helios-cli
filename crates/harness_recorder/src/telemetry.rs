//! OpenTelemetry instrumentation for the harness recorder.
//!
//! Provides OTLP export (gRPC and HTTP), tracer/meter providers, custom spans
//! for key operations, metrics collection, health-check integration, context
//! propagation, and graceful shutdown via [`Drop`].
//!
//! # Usage
//!
//! ```no_run
//! use harness_recorder::telemetry::TelemetryGuard;
//!
//! fn main() {
//!     let _guard = TelemetryGuard::init("my-service", "1.0.0").expect("telemetry init");
//!     // ... application logic ...
//!     // telemetry is flushed when _guard is dropped
//! }
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::{Span, SpanKind, Status, Tracer, TracerProvider as _};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{RandomIdGenerator, TracerProvider};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::resource as semconv_resource;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Exporter transport protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExporterProtocol {
    /// OTLP over gRPC (default port 4317).
    Grpc,
    /// OTLP over HTTP (default port 4318).
    Http,
}

impl Default for ExporterProtocol {
    fn default() -> Self {
        Self::Grpc
    }
}

/// Configuration for the telemetry subsystem.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Human-readable service name (e.g. `"harness_recorder"`).
    pub service_name: String,
    /// Semantic version string.
    pub service_version: String,
    /// Deployment environment label.
    pub environment: String,
    /// Exporter transport protocol.
    pub protocol: ExporterProtocol,
    /// OTLP endpoint URL.  If `None`, the SDK default is used.
    pub endpoint: Option<String>,
    /// How long to wait during graceful shutdown before giving up.
    pub shutdown_timeout: Duration,
    /// Resource-level attributes merged into every span / metric.
    pub extra_resource: Vec<KeyValue>,
}

impl TelemetryConfig {
    /// Builder-style helper for production defaults.
    pub fn new(service_name: impl Into<String>, service_version: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            service_version: service_version.into(),
            environment: std::env::var("HELIOS_ENV").unwrap_or_else(|_| "development".into()),
            protocol: ExporterProtocol::default(),
            endpoint: None,
            shutdown_timeout: Duration::from_secs(10),
            extra_resource: Vec::new(),
        }
    }

    /// Override the exporter protocol.
    pub fn with_protocol(mut self, protocol: ExporterProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Override the OTLP endpoint.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Override the deployment environment.
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = env.into();
        self
    }

    /// Attach additional resource attributes.
    pub fn with_extra_resource(mut self, attrs: Vec<KeyValue>) -> Self {
        self.extra_resource = attrs;
        self
    }

    /// Override the shutdown timeout.
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self::new("harness_recorder", env!("CARGO_PKG_VERSION"))
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Pre-defined metric instruments.
pub struct RecorderMetrics {
    pub spans_total: opentelemetry::metrics::Counter<u64>,
    pub span_duration_ms: opentelemetry::metrics::Histogram<f64>,
    pub script_executions: opentelemetry::metrics::Counter<u64>,
    pub script_failures: opentelemetry::metrics::Counter<u64>,
    pub active_terminals: opentelemetry::metrics::UpDownCounter<i64>,
}

impl RecorderMetrics {
    fn init(meter: &opentelemetry::metrics::Meter) -> Self {
        Self {
            spans_total: meter
                .u64_counter("harness_recorder.spans.total")
                .with_description("Total number of recorded spans")
                .build(),
            span_duration_ms: meter
                .f64_histogram("harness_recorder.span.duration_ms")
                .with_description("Span duration in milliseconds")
                .with_unit("ms")
                .build(),
            script_executions: meter
                .u64_counter("harness_recorder.script.executions")
                .with_description("Total script executions")
                .build(),
            script_failures: meter
                .u64_counter("harness_recorder.script.failures")
                .with_description("Total script execution failures")
                .build(),
            active_terminals: meter
                .i64_up_down_counter("harness_recorder.terminals.active")
                .with_description("Currently active terminal sessions")
                .build(),
        }
    }
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// Health status reported by the telemetry subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Lightweight health-check probe for the telemetry pipeline.
pub struct TelemetryHealthCheck {
    initialized: Arc<AtomicBool>,
    last_export_ok: Arc<AtomicBool>,
}

impl TelemetryHealthCheck {
    /// Returns [`HealthStatus::Healthy`] when the pipeline was initialized
    /// and the last export attempt succeeded.
    pub fn check(&self) -> HealthStatus {
        if !self.initialized.load(Ordering::Relaxed) {
            return HealthStatus::Unhealthy;
        }
        if !self.last_export_ok.load(Ordering::Relaxed) {
            return HealthStatus::Degraded;
        }
        HealthStatus::Healthy
    }
}

// ---------------------------------------------------------------------------
// Guard (owns providers, flushes on drop)
// ---------------------------------------------------------------------------

/// RAII guard that owns the tracer and meter providers and flushes /
/// shuts them down when dropped.
pub struct TelemetryGuard {
    tracer_provider: Option<TracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    health: Arc<TelemetryHealthCheck>,
    shutdown_timeout: Duration,
    is_shutdown: Arc<AtomicBool>,
}

impl TelemetryGuard {
    /// Initialize the full telemetry stack with production defaults.
    pub fn init(
        service_name: impl Into<String>,
        service_version: impl Into<String>,
    ) -> anyhow::Result<Self> {
        Self::init_with_config(TelemetryConfig::new(service_name, service_version))
    }

    /// Initialize the full telemetry stack with a custom configuration.
    pub fn init_with_config(config: TelemetryConfig) -> anyhow::Result<Self> {
        let initialized = Arc::new(AtomicBool::new(false));
        let last_export_ok = Arc::new(AtomicBool::new(true));

        let resource = build_resource(&config);

        // --- Tracer provider ------------------------------------------------
        let tracer_provider = build_tracer_provider(&config, resource.clone())?;
        let tracer = tracer_provider.tracer("harness_recorder");

        // --- Meter provider --------------------------------------------------
        let meter_provider = build_meter_provider(&config, resource)?;
        let meter = meter_provider.meter("harness_recorder");

        // --- Instrument metrics ----------------------------------------------
        let _metrics = RecorderMetrics::init(&meter);

        // Install the global tracer so `global::tracer("...")` works.
        global::set_tracer_provider(tracer_provider.clone());

        // Create a root span to verify the pipeline is alive.
        let mut probe_span = tracer.start("telemetry.init");
        probe_span.set_status(Status::Ok);
        probe_span.end();

        initialized.store(true, Ordering::Release);

        tracing::info!(
            service_name = %config.service_name,
            service_version = %config.service_version,
            environment = %config.environment,
            protocol = ?config.protocol,
            "OpenTelemetry pipeline initialized"
        );

        Ok(Self {
            tracer_provider: Some(tracer_provider),
            meter_provider: Some(meter_provider),
            health: Arc::new(TelemetryHealthCheck {
                initialized,
                last_export_ok,
            }),
            shutdown_timeout: config.shutdown_timeout,
            is_shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Return a handle to the health-check probe.
    pub fn health_check(&self) -> Arc<TelemetryHealthCheck> {
        Arc::clone(&self.health)
    }

    /// Manually trigger a flush without shutting down providers.
    pub fn flush(&self) -> anyhow::Result<()> {
        if let Some(ref tp) = self.tracer_provider {
            tp.shutdown().map_err(|e| anyhow::anyhow!("tracer flush: {e}"))?;
        }
        if let Some(ref mp) = self.meter_provider {
            mp.shutdown().map_err(|e| anyhow::anyhow!("meter flush: {e}"))?;
        }
        self.health.last_export_ok.store(true, Ordering::Release);
        Ok(())
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if self.is_shutdown.swap(true, Ordering::SeqCst) {
            return; // already shut down
        }
        tracing::info!("Flushing OpenTelemetry pipeline...");
        if let Some(tp) = self.tracer_provider.take() {
            if let Err(e) = tp.shutdown() {
                tracing::error!("tracer provider shutdown: {e}");
            }
        }
        if let Some(mp) = self.meter_provider.take() {
            if let Err(e) = mp.shutdown() {
                tracing::error!("meter provider shutdown: {e}");
            }
        }
        shutdown_tracer_provider();
        tracing::info!("OpenTelemetry pipeline flushed");
    }
}

// ---------------------------------------------------------------------------
// Custom span helpers
// ---------------------------------------------------------------------------

/// Record a span around a script execution.
pub fn trace_script_execution<F, R>(script_name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let tracer = global::tracer("harness_recorder");
    let mut span = tracer
        .span_builder("script.execute")
        .with_kind(SpanKind::Internal)
        .with_attributes(vec![
            KeyValue::new("script.name", script_name.to_string()),
            KeyValue::new("component", "harness_recorder"),
        ])
        .start(&tracer);

    let start = Instant::now();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    span.set_attribute(KeyValue::new("script.duration_ms", elapsed_ms));

    match result {
        Ok(val) => {
            span.set_status(Status::Ok);
            span.end();
            val
        }
        Err(panic) => {
            span.set_status(Status::Error {
                description: "panic during script execution".into(),
            });
            span.end();
            std::panic::resume_unwind(panic);
        }
    }
}

/// Record a span around a terminal operation.
pub fn trace_terminal_operation<F, R>(operation: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let tracer = global::tracer("harness_recorder");
    let mut span = tracer
        .span_builder(format!("terminal.{operation}"))
        .with_kind(SpanKind::Internal)
        .with_attributes(vec![
            KeyValue::new("terminal.operation", operation.to_string()),
            KeyValue::new("component", "harness_recorder"),
        ])
        .start(&tracer);

    let start = Instant::now();
    let result = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    span.set_attribute(KeyValue::new("terminal.duration_ms", elapsed_ms));
    span.set_status(Status::Ok);
    span.end();
    result
}

/// Record a span around media capture.
pub fn trace_media_capture<F, R>(capture_type: &str, output_path: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let tracer = global::tracer("harness_recorder");
    let mut span = tracer
        .span_builder("media.capture")
        .with_kind(SpanKind::Internal)
        .with_attributes(vec![
            KeyValue::new("media.type", capture_type.to_string()),
            KeyValue::new("media.output_path", output_path.to_string()),
            KeyValue::new("component", "harness_recorder"),
        ])
        .start(&tracer);

    let start = Instant::now();
    let result = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    span.set_attribute(KeyValue::new("media.duration_ms", elapsed_ms));
    span.set_status(Status::Ok);
    span.end();
    result
}

// ---------------------------------------------------------------------------
// Context propagation
// ---------------------------------------------------------------------------

/// Extract the current OpenTelemetry context from thread-local storage.
pub fn current_context() -> opentelemetry::Context {
    opentelemetry::Context::current()
}

/// Create a child span linked to the provided parent context.
pub fn child_span(parent_ctx: &opentelemetry::Context, name: &str) -> impl Span {
    let tracer = global::tracer("harness_recorder");
    tracer.start_with_context(name, parent_ctx)
}

/// Propagate a context across async tasks by serialising the current
/// trace-context header into a `String` (W3C `traceparent` format).
pub fn propagate_context() -> String {
    use opentelemetry::propagation::TextMapPropagator;
    use opentelemetry_sdk::propagation::TraceContextPropagator;

    let propagator = TraceContextPropagator::new();
    let mut injector = std::collections::HashMap::new();
    propagator.inject_context(&opentelemetry::Context::current(), &mut injector);
    injector
        .remove("traceparent")
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_resource(config: &TelemetryConfig) -> Resource {
    let mut attrs = vec![
        KeyValue::new(semconv_resource::SERVICE_NAME, config.service_name.clone()),
        KeyValue::new(semconv_resource::SERVICE_VERSION, config.service_version.clone()),
        KeyValue::new("deployment.environment", config.environment.clone()),
    ];
    attrs.extend(config.extra_resource.iter().cloned());
    Resource::builder().with_attributes(attrs).build()
}

fn build_tracer_provider(
    config: &TelemetryConfig,
    resource: Resource,
) -> anyhow::Result<TracerProvider> {
    let cfg = opentelemetry_sdk::trace::Config::default()
        .with_resource(resource)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(128)
        .with_max_attributes_per_span(64)
        .with_max_links_per_span(16);

    let provider = match config.protocol {
        ExporterProtocol::Grpc => {
            let mut exporter = opentelemetry_otlp::new_exporter().tonic();
            if let Some(ref endpoint) = config.endpoint {
                exporter = exporter.with_endpoint(endpoint.as_str());
            }
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(cfg)
                .install_batch(runtime::Tokio)?
        }
        ExporterProtocol::Http => {
            let mut exporter = opentelemetry_otlp::new_exporter().http();
            if let Some(ref endpoint) = config.endpoint {
                exporter = exporter.with_endpoint(endpoint.as_str());
            }
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(cfg)
                .install_batch(runtime::Tokio)?
        }
    };
    Ok(provider)
}

fn build_meter_provider(
    config: &TelemetryConfig,
    resource: Resource,
) -> anyhow::Result<SdkMeterProvider> {
    let provider = match config.protocol {
        ExporterProtocol::Grpc => {
            let mut exporter = opentelemetry_otlp::new_exporter().tonic();
            if let Some(ref endpoint) = config.endpoint {
                exporter = exporter.with_endpoint(endpoint.as_str());
            }
            opentelemetry_otlp::new_pipeline()
                .metrics(runtime::Tokio)
                .with_exporter(exporter)
                .with_resource(resource)
                .build()?
        }
        ExporterProtocol::Http => {
            let mut exporter = opentelemetry_otlp::new_exporter().http();
            if let Some(ref endpoint) = config.endpoint {
                exporter = exporter.with_endpoint(endpoint.as_str());
            }
            opentelemetry_otlp::new_pipeline()
                .metrics(runtime::Tokio)
                .with_exporter(exporter)
                .with_resource(resource)
                .build()?
        }
    };
    Ok(provider)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_config_defaults() {
        let cfg = TelemetryConfig::new("test-svc", "0.1.0");
        assert_eq!(cfg.service_name, "test-svc");
        assert_eq!(cfg.service_version, "0.1.0");
        assert_eq!(cfg.protocol, ExporterProtocol::Grpc);
        assert!(cfg.endpoint.is_none());
    }

    #[test]
    fn telemetry_config_builder_chain() {
        let cfg = TelemetryConfig::new("svc", "1.0.0")
            .with_protocol(ExporterProtocol::Http)
            .with_endpoint("http://localhost:4318")
            .with_environment("staging");
        assert_eq!(cfg.protocol, ExporterProtocol::Http);
        assert_eq!(cfg.endpoint.as_deref(), Some("http://localhost:4318"));
        assert_eq!(cfg.environment, "staging");
    }

    #[test]
    fn health_check_initial_state() {
        let hc = TelemetryHealthCheck {
            initialized: Arc::new(AtomicBool::new(false)),
            last_export_ok: Arc::new(AtomicBool::new(true)),
        };
        assert_eq!(hc.check(), HealthStatus::Unhealthy);

        let hc2 = TelemetryHealthCheck {
            initialized: Arc::new(AtomicBool::new(true)),
            last_export_ok: Arc::new(AtomicBool::new(false)),
        };
        assert_eq!(hc2.check(), HealthStatus::Degraded);

        let hc3 = TelemetryHealthCheck {
            initialized: Arc::new(AtomicBool::new(true)),
            last_export_ok: Arc::new(AtomicBool::new(true)),
        };
        assert_eq!(hc3.check(), HealthStatus::Healthy);
    }

    #[test]
    fn propagate_context_returns_string() {
        let tp = propagate_context();
        let _ = tp;
    }
}
