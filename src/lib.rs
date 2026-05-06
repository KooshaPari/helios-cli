//! Helios CLI — root library crate
//!
//! This crate is the top-level library for the Helios CLI workspace.
//! It re-exports the primary CLI binary from `helios-rs/cli` for Cargo-level
//! access and provides workspace-level documentation.
//!
//! ## Workspace Structure
//!
//! - `helios-rs/` — Primary Rust workspace (all crates)
//! - `codex-rs/`  — Parallel workspace (upstream-consumable subset)
//! - `codex-cli/` — TypeScript CLI (user-facing commands)
//! - `sdk/`       — Integration SDKs
//!
//! ## Building
//!
//! ```bash
//! # Full workspace build
//! cargo build --workspace
//!
//! # CLI binary only
//! cargo build -p helios-cli
//!
//! # Run the CLI
//! cargo run -p helios-cli -- --help
//! ```

# Re-export the CLI crate for consumers who depend on this workspace as a library.
pub use helios_cli;
