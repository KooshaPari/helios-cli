# helios-cli: Code Entity Map

## Rust Core (codex-rs)

| Entity | Path | Maps To |
|--------|------|---------|
| Core types and config | `codex-rs/core/` | FR-EXEC-001, FR-CLI-003 |
| Execution engine | `codex-rs/exec/` | FR-EXEC-001, FR-PROV-004 |
| App server (JSON-RPC) | `codex-rs/app-server/` | FR-EXEC-003 |
| App server protocol | `codex-rs/app-server-protocol/` | FR-EXEC-003 |
| Backend client (providers) | `codex-rs/backend-client/` | FR-EXEC-002, FR-PROV-001, FR-PROV-002, FR-PROV-003 |
| Artifacts manager | `codex-rs/artifacts/` | FR-EXEC-004 |
| ChatGPT auth | `codex-rs/chatgpt/` | FR-CLI-004 |
| CLI binary | `codex-rs/cli/` | FR-CLI-001 |
| ANSI escape handling | `codex-rs/ansi-escape/` | FR-CLI-001 |
| Async utilities | `codex-rs/async-utils/` | FR-EXEC-001 |
| Patch application | `codex-rs/apply-patch/` | FR-EXEC-001 |

## TypeScript CLI (codex-cli)

| Entity | Path | Maps To |
|--------|------|---------|
| CLI entry point | `codex-cli/src/` | FR-CLI-001, FR-CLI-002 |
| Commands | `codex-cli/src/commands/` | FR-CLI-002 |

## Helios Extensions

| Entity | Path | Maps To |
|--------|------|---------|
| Helios Rust extensions | `helios-rs/` | FR-HELIOS-001, FR-HELIOS-003 |
| SDKs | `sdk/` | FR-HELIOS-001 |
| AgilePlus integration | `agileplus/` | FR-HELIOS-003 |

## Build System

| Entity | Path | Maps To |
|--------|------|---------|
| Root Bazel config | `BUILD.bazel`, `MODULE.bazel` | FR-BUILD-001 |
| Rust workspace | `Cargo.toml` | FR-BUILD-001 |
| Node workspace | `pnpm-workspace.yaml` | FR-BUILD-002 |
