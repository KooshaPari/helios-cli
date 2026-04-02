//! Sentry error tracking configuration for helios-cli
//!
//! Traces to: FR-HELIOS-SENTRY-001
//!
//! Provides panic capture, error tracking, and breadcrumbs for CLI operations.

use std::env;

/// Initialize Sentry for the CLI application.
///
/// # Environment Variables
/// - `SENTRY_DSN`: Sentry project DSN (required for production)
/// - `SENTRY_ENVIRONMENT`: Environment tag (defaults to "development")
///
/// # Usage
/// ```no_run
/// let _guard = sentry_config::init();
/// ```
pub fn init() -> Option<sentry::ClientInitGuard> {
    let dsn = env::var("SENTRY_DSN").ok()?;
    let environment = env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let release = concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"));

    Some(sentry::init((
        dsn,
        sentry::ClientOptions {
            environment: Some(environment.into()),
            release: Some(release.into()),
            attach_stacktrace: true,
            debug: cfg!(debug_assertions),
            max_breadcrumbs: 100,
            ..Default::default()
        },
    )))
}

/// Capture an error with context for CLI operations.
pub fn capture_cli_error(error: &impl std::error::Error, context: &str) {
    sentry::with_scope(
        |scope| {
            scope.set_extra("cli_context", context.into());
        },
        || sentry::capture_error(error),
    );
}

/// Set user context for error tracking.
pub fn set_user(user_id: &str, username: Option<&str>) {
    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: Some(user_id.into()),
            username: username.map(|s| s.into()),
            ..Default::default()
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-HELIOS-SENTRY-001
    #[test]
    fn test_sentry_init_without_dsn() {
        // Should return None when SENTRY_DSN is not set
        let result = init();
        assert!(result.is_none());
    }
}
