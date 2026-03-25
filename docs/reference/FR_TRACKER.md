# helios-cli: FR Implementation Tracker

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-EXEC-001 | Sandboxed command execution | Implemented | `codex-rs/exec/`, `codex-rs/core/` |
| FR-EXEC-002 | Multi-provider backends | In Progress | `codex-rs/backend-client/` |
| FR-EXEC-003 | JSON-RPC protocol | Implemented | `codex-rs/app-server-protocol/` |
| FR-EXEC-004 | Artifact tracking | Implemented | `codex-rs/artifacts/` |
| FR-CLI-001 | Interactive TUI | Implemented | `codex-cli/src/` |
| FR-CLI-002 | Command registry | Implemented | `codex-cli/src/commands/` |
| FR-CLI-003 | Configuration management | Implemented | `codex-rs/core/` |
| FR-CLI-004 | ChatGPT sign-in | Implemented | `codex-rs/chatgpt/` |
| FR-PROV-001 | OpenAI provider | Implemented | `codex-rs/backend-client/` |
| FR-PROV-002 | Anthropic Claude provider | In Progress | `codex-rs/backend-client/` |
| FR-PROV-003 | Google Gemini provider | In Progress | `codex-rs/backend-client/` |
| FR-PROV-004 | Provider-agnostic tool calling | In Progress | `codex-rs/exec/` |
| FR-HELIOS-001 | thegent integration | Planned | `helios-rs/` |
| FR-HELIOS-002 | Patch superset sync | In Progress | `scripts/` |
| FR-HELIOS-003 | AgilePlus integration | Planned | `helios-rs/` |
| FR-BUILD-001 | Bazel build | Implemented | `BUILD.bazel`, `MODULE.bazel` |
| FR-BUILD-002 | npm distribution | Implemented | `codex-cli/package.json` |
| FR-BUILD-003 | Homebrew distribution | Implemented | - |
