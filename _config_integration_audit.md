# helios_config Integration Audit

**Date:** 2026-08-21
**File analyzed:** `crates/helios_config/src/lib.rs` (934 lines)

---

## 1. What providers/models does it configure?

**None.** `helios_config` has zero LLM provider or model configuration. It does not reference OpenAI, Anthropic, Bedrock, Ollama, or any model identifier. It does not configure API keys, endpoints, or model names.

The config is purely about **harness infrastructure** — cache TTLs, process timeouts, scaling thresholds, circuit breaker parameters, etc.

## 2. Does it support multi-provider?

**No.** There is no provider abstraction, no provider enum, no per-provider config sections. The config is provider-agnostic in the LLM sense — it doesn't interact with LLM providers at all.

## 3. What sandbox modes does it configure?

**None.** There is no sandbox mode configuration. The closest concept is `RunnerConfig` (`timeout_secs`) which controls process execution timeout, but it does not configure any sandboxing (no seccomp, bwrap, Landlock, network policies, filesystem restrictions, etc.).

## 4. Does it support MCP?

**No.** There is no MCP (Model Context Protocol) configuration anywhere in the file — no MCP server definitions, no tool schemas, no transport config (stdio/SSE/streamable HTTP).

## 5. What session management does it configure?

**Minimal/none in the LLM sense.** It configures:

- **`TeammateConfig`**: max concurrent tasks per teammate (default: 1), timeout (300s). This is orchestration concurrency, not conversation/session state.
- **`SpecConfig`**: version strings and rollback timeouts — not session state.
- **`CheckpointConfig`**: git signature metadata for checkpoint commits — not session state.

There is no conversation history, message threading, or session persistence configuration.

## 6. How does its config compare to Codex CLI's config?

**They are fundamentally different domains:**

| Dimension | `helios_config` | Codex CLI |
|---|---|---|
| **Purpose** | Harness infrastructure tuning (cache, scaling, circuit breaker) | CLI UX + LLM provider routing |
| **Provider config** | None | `model_providers` map with per-provider API keys, base URLs, wire-protocol |
| **Model selection** | None | `default_model`, per-provider model aliases |
| **Sandbox** | None | Full sandbox config: `sandbox_mode` (off/docker/ptrace), `sandbox_workspace`, `sandbox_network` |
| **MCP** | None | MCP server config via `[mcp_servers]` |
| **Approvals** | None | `approval_policy` (suggest/auto-edit/full-auto), `require_json_schema` |
| **Project doc** | None | `project_doc_max_bytes` for AGENTS.md injection |
| **Session** | None | Handled at runtime, not in config file |

Codex CLI's config (`~/.codex/config.toml`) is a full developer-agent configuration file. `helios_config` is a backend infrastructure tuning file — they serve completely different purposes.

## 7. How does it compare to Claude Code's settings?

**Also fundamentally different:**

| Dimension | `helios_config` | Claude Code (`settings.json` / `CLAUDE.md`) |
|---|---|---|
| **Provider config** | None | Uses Anthropic API; env vars for keys |
| **Model selection** | None | Model override via `/model` or config |
| **Permissions** | None | `allowedTools`, `blockedCommands`, permission policies |
| **MCP** | None | Full MCP server configuration |
| **System prompt** | None | `systemPrompt`, `CLAUDE.md` project docs |
| **Hooks** | None | Pre/post tool-use hooks |
| **Session** | None | Conversation threading, history persistence |
| **Domain** | Infrastructure tuning | Developer-facing agent settings |

## 8. Binary Entry Point Analysis

### Is there a binary crate that wires the helios crates together?

**No.** The workspace has no dedicated `helios` binary crate. The only binary crate in the helios workspace is:

- **`harness_recorder`** (`crates/harness_recorder/src/main.rs`) — a standalone "KLA" (Kommand Line Automation) recorder tool, a Playwright-for-CLI concept. It does **not** wire together the harness crates or use `helios_config`.

### Top-level `src/`

The top-level `src/` directory contains:
- `lib.rs` — a one-line stub: `//! Root integration-test harness for heliosCLI E2E smoke checks.`
- `helios_router_ui/` — a **Python** package (with `nats_client.py`, `__init__.py`, `db/`, `pareto/`, `ui/`), not a Rust binary.

### Workspace `Cargo.toml`

The workspace `Cargo.toml` is configured as a library workspace (`lib.rs` only, no `[[bin]]`). It lists 20+ member crates but none are configured as binary crates in the workspace manifest.

### Binary crates found (none are helios-specific):

All `[[bin]]` entries found in the repo belong to the **vendored `codex-rs/` workspace**, not to helios.

## Summary

`helios_config` is a **narrow infrastructure configuration crate** for tuning backend harness defaults (cache, scaling, circuit breaker, process timeouts). It has:

- **No LLM provider/model configuration**
- **No sandbox configuration**
- **No MCP support**
- **No session management**
- **No binary entry point** that wires it into a runnable CLI

The helios-cli repo currently **lacks a unified binary** that composes these library crates into a working CLI. The `helios_config` crate exists but is not consumed by any binary in the workspace — it is a library crate with no `[[bin]]` target and no top-level main.rs. The only binary (`harness_recorder`/KLA) is a standalone tool that doesn't use it.

**Gap vs Codex CLI / Claude Code:** Codex and Claude Code have rich config files covering provider routing, sandbox policies, tool permissions, MCP servers, and session behavior. `helios_config` has none of these. To reach feature parity with those tools, helios would need entirely new config structures for LLM provider selection, sandbox modes, MCP server definitions, approval policies, and session persistence.
