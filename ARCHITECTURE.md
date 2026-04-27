# Architecture

## Overview
- helios-cli is a Rust workspace with a CLI, core libraries, and supporting integration packages.
- The codebase splits user-facing commands from reusable protocol and execution layers.
- This document is a skeleton for describing the command, SDK, and runtime boundaries.

## Components
## helios-cli
- Top-level CLI entrypoint and user-facing command surface.

## codex-rs
- Shared Rust workspace for core client, execution, protocol, and support crates.

## helios-rs
- Rust application layer and supporting CLI/package modules for helios functionality.

## shell-tool-mcp
- MCP-facing shell tool integration used by the CLI ecosystem.

## sdk/typescript
- TypeScript SDK surface for external automation or integration.

## Data flow
```text
command line -> CLI layer -> shared Rust crates -> tool/runtime adapters -> external systems
```

## Key invariants
- Keep command parsing separate from execution and protocol handling.
- Treat shared workspace crates as the primary home for reusable logic.
- Preserve stable interfaces between CLI commands and tool adapters.

## Cross-cutting concerns (config, telemetry, errors)
- Config: centralize runtime flags, environment values, and workspace settings.
- Telemetry: instrument command execution and tool calls consistently.
- Errors: return actionable CLI failures while preserving internal context.

## Future considerations
- Expand the component map to specific crates once ownership settles.
- Add startup and execution diagrams for command routing and tool invocation.
- Document integration points for SDK consumers and MCP transports.
