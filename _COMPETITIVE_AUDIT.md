# HELIOS-CLI: Competitive Feature Audit

**Date:** 2026-08-21
**Scope:** helios-cli vs OpenAI Codex CLI vs Claude Code vs Aider

---

## Architecture Reality

helios-cli is **two separate projects in one repo**:

1. **codex-rs/** — Vendored OpenAI Codex CLI (50+ Rust crates). This IS the working CLI.
2. **crates/** — 24 harness crates (~3,400 lines). These are NOT wired into any binary.

**There is no unified helios-cli binary.** The only binary is `kla` (a Playwright-for-CLI recorder) which doesn't consume any harness crate.

---

## Per-Crate Audit (crates/ directory)

### REAL Implementations (10 crates)

| Crate | Lines | What It Does | Value Over Upstream |
|-------|-------|-------------|-------------------|
| **harness_queue** | 379 | MPSC channel + ring buffer + work-stealing | Task queuing for multi-agent |
| **harness_runner** | 416 | `tokio::process::Command` with timeout, env, stdin | Process orchestration |
| **harness_scaling** | 391 | Predictive scaler (linear regression), circuit breaker, token bucket | Auto-scaling agents |
| **harness_checkpoint** | 18* | libgit2 commit/restore/status, file snapshots | Git-level checkpointing |
| **harness_elicitation** | 16* | Regex intent classifier, spec generator | Spec generation from prompts |
| **harness_spec** | 17* | YAML/JSON spec parser, semver validation | Spec format standardization |
| **harness_interfaces** | 183 | Handler/Publisher/Subscriber traits | Extensibility contracts |
| **harness_normalizer** | 183 | Text/JSON/URL/path normalization | Input sanitization |
| **harness_orchestrator** | 328 | Task dependency resolution, agent assignment | Multi-agent coordination |
| **harness_cache** | 233 | TTL expiry, capacity eviction, async RwLock | Caching layer |

*Lines are lib.rs only; full implementations span multiple files

### PARTIAL Implementations (5 crates)

| Crate | What Works | What's Stubbed |
|-------|-----------|---------------|
| **harness_verify** | cargo test / pytest runner | Security scanning returns Skipped, perf benchmark returns Skipped |
| **harness_teammates** | Registry + delegation trait | `SimpleDelegationAdapter` always returns Completed (no real subprocess) |
| **harness_discoverer** | Manual registration | No network discovery, no mDNS/DNS-SD |
| **arch_test** | Property tests, TDD tests | `BoundaryEnforcer` holds empty Vec, no actual code scanning |
| **KLA recorder** | CLI parsing, PTY capture | Media encoding (GIF/screenshot) may be stubbed |

### STUB (1 crate)

| Crate | Issue |
|-------|-------|
| **harness_rollback** | State machine is real, but `rollback()` is a no-op — never touches files or git |

---

## Feature Matrix: helios-cli vs Competitors

| Feature | helios-cli (harness) | Codex CLI (vendored) | Claude Code | Aider |
|---------|---------------------|---------------------|-------------|-------|
| **LLM Providers** | NONE (no provider code) | OpenAI, Bedrock, Ollama, LM Studio, custom | Anthropic + 3rd party | OpenAI, Anthropic, Gemini, local |
| **Streaming** | NONE | YES (real-time TUI) | YES | YES |
| **Tool Use** | NONE (traits only) | shell, apply-patch, file_search, computer_use | 14 tools (Read/Write/Edit/Bash/Glob/Grep/LSP/Web) | edit, browser, dir |
| **File Ops** | NONE | Structured patches (apply-patch) | Full read/write/edit/checkpoint/undo | Multi-file edit, repo map |
| **Shell Execution** | harness_runner (real) | YES (sandboxed) | YES (sandboxed) | YES |
| **Git Integration** | harness_checkpoint (libgit2) | Deep (PR, review, worktree) | Deep (Bash + skills) | Deep (auto-commit, diff) |
| **MCP Support** | NONE | Client + Server | Client + Server (HTTP/SSE/stdio/WS) | Client |
| **Session Mgmt** | NONE | .codex/ local state, memories | CLAUDE.md + auto memory + sessions | .aider/ config |
| **Sandboxing** | NONE | bubblewrap (Linux), windows-sandbox | Permission modes (Manual/Auto/Plan) | NONE (runs locally) |
| **Multi-Agent** | harness_orchestrator + teammates (partial) | NONE | Subagents, teams, worktrees, SDK | NONE |
| **Rollback/Checkpoint** | harness_rollback (STUB) + harness_checkpoint (real) | NONE (apply-patch is atomic) | File checkpointing + undo | Git-based undo |
| **Task Queuing** | harness_queue (REAL MPSC) | NONE | Background tasks | NONE |
| **Verification** | harness_verify (partial — cargo test works) | Auto-Review mode | Lint/test hooks | Pre-commit hooks |
| **Scaling** | harness_scaling (REAL — predictive, circuit breaker) | NONE | NONE | NONE |
| **Spec Generation** | harness_elicitation (REAL — regex classifier) | NONE | NONE | NONE |
| **Spec Format** | harness_spec (REAL — YAML/JSON parser) | NONE | NONE | NONE |
| **Configuration** | helios_config (cache/scaling params ONLY) | Full (provider, sandbox, MCP, approval) | Full (permissions, MCP, hooks, system prompt) | Full (.aider.yml) |
| **TUI Quality** | NONE (no unified binary) | Full TUI (ratatui) | Rich terminal UI | Rich terminal UI |
| **Distribution** | NONE (no release binary) | crates.io, npm, brew, curl | npm, brew | pip, brew |

---

## Honest Assessment

### What helios-cli's harness crates ACTUALLY add over upstream Codex:

1. **Task queuing** (harness_queue) — Real MPSC with work-stealing. Codex has nothing like this.
2. **Process orchestration** (harness_runner) — Real process spawning with timeout. Codex has exec but not orchestration.
3. **Auto-scaling** (harness_scaling) — Predictive scaler with circuit breaker. Unique.
4. **Git checkpointing** (harness_checkpoint) — libgit2-based commit/restore. Codex uses apply-patch (atomic but not restorable).
5. **Spec generation** (harness_elicitation) — Intent classification + spec generation. Unique.
6. **Multi-agent coordination** (harness_orchestrator) — Task dependency resolution. Codex has none.
7. **Architecture testing** (arch_test) — Hexagonal boundary enforcement. Unique.

### What's MISSING that competitors have:

1. **No unified binary** — Nothing wires the harness crates into a CLI
2. **No LLM provider integration** — Zero code for talking to any AI model
3. **No tool system** — No file ops, no shell sandbox, no MCP
4. **No TUI** — The harness crates are libraries, not a user-facing tool
5. **No distribution** — No release pipeline, no install method
6. **Rollback is a no-op** — The one crate that should connect checkpoint to undo doesn't work

### The Verdict

helios-cli's harness crates are **real, well-tested infrastructure libraries** solving problems that Codex CLI, Claude Code, and Aider don't address (queuing, scaling, checkpointing, spec generation, multi-agent coordination). But they are **disconnected from any user-facing binary**.

The repo is essentially:
- A Codex CLI fork (vendored, working, but unmodified in the harness layer)
- A collection of infrastructure libraries (real code, real tests, but unused)

**To be usable, helios-cli needs a binary that wires the harness crates into the Codex CLI workflow.** Without that, it's a Codex fork with some libraries in a subdirectory.

---

## What Would Make helios-cli Competitive

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| **P0** | Wire harness crates into a `helios` binary that extends Codex CLI | 1-2 weeks | Makes it a real tool |
| **P0** | Connect harness_queue + harness_runner to Codex's exec system | 3 days | Task queuing works |
| **P1** | Connect harness_checkpoint + harness_rollback to Codex's file ops | 3 days | Undo/checkpoint works |
| **P1** | Connect harness_scaling to Codex's process management | 2 days | Auto-scaling works |
| **P2** | Connect harness_orchestrator + harness_teammates for multi-agent | 1 week | Multi-agent works |
| **P2** | Wire harness_verify to Codex's auto-review mode | 2 days | Verification works |
| **P3** | Connect harness_elicitation + harness_spec to Codex's prompt system | 3 days | Spec generation works |
