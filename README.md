> **Pinned references (Phenotype-org)**
> - MSRV: see rust-toolchain.toml
> - cargo-deny config: see deny.toml
> - cargo-audit: rustsec/audit-check@v2 weekly
> - Branch protection: 1 reviewer required, no force-push
> - Authority: phenotype-org-governance/SUPERSEDED.md

[![AI Slop Inside](https://sladge.net/badge.svg)](https://sladge.net)

<!-- PHENOTYPE FORK CONTEXT — see "About this fork" below -->

## About this fork

`KooshaPari/helios-cli` is a Phenotype-org fork of [OpenAI Codex CLI](https://github.com/openai/codex). The upstream README is preserved verbatim below; this preamble documents fork-specific divergence so downstream consumers can tell what is upstream behavior versus a Phenotype patch.

**Why we fork:** Codex CLI is one of the agent backends wired into the Phenotype `thegent` dispatcher. We carry security and workspace fixes ahead of upstream merge cadence and ship them via this fork's tagged releases.

**Recent Phenotype-specific changes:**

- **#527 — `fix(workspace): declare 19 missing workspace dependencies`** — restores `cargo metadata`/`cargo build` resolution in `codex-rs/` after upstream workspace drift.
- **#526 — `fix(deps): bump 10 HIGH CVEs in codex-rs`** — `aws-lc-sys`, `quinn-proto`, `rustls-webpki` upgrades; closes RUSTSEC advisories that upstream had not yet absorbed.
- **#525 — `fix(workspace): unblock codex-rs cargo resolution`** — preparatory unblock for the CVE sweep.
- **#524 / #523 / #522** — OpenSSF Scorecard workflow, `CODE_OF_CONDUCT.md`, `.github/FUNDING.yml` — Phenotype-org hygiene baseline.
- **#519–#521** — pinned floating external GitHub Actions to commit SHAs across `stage-gates.yml`, `issue-labeler.yml`, `issue-deduplicator.yml`.
- **#518** — bootstrapped a VitePress docs deploy workflow.

**Where to find Phenotype-specific docs:**

- Roadmap and fork strategy: [`PLAN.md`](./PLAN.md)
- Phenotype contribution conventions: [`docs/contributing.md`](./docs/contributing.md) and the upstream `AGENTS.md` chain in `docs/agents_md.md`
- Phenotype org context: [`KooshaPari/phenotype-infrakit`](https://github.com/KooshaPari/phenotype-infrakit) and the `thegent` dispatcher

**Upstream tracking:** we periodically rebase or merge from `openai/codex@main`. Open a PR against this fork for Phenotype-specific changes; upstreamable fixes should also be PR'd to `openai/codex` directly.

---

<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

# HeliosCLI

[![Build](https://img.shields.io/github/actions/workflow/status/KooshaPari/helios-cli/rust-ci.yml?branch=main&label=build)](https://github.com/KooshaPari/helios-cli/actions/workflows/rust-ci.yml)
[![Release](https://img.shields.io/github/v/release/KooshaPari/helios-cli?include_prereleases&sort=semver)](https://github.com/KooshaPari/helios-cli/releases)
[![License](https://img.shields.io/github/license/KooshaPari/helios-cli)](LICENSE)
[![Phenotype](https://img.shields.io/badge/Phenotype-org-blueviolet)](https://github.com/KooshaPari)


**Status:** stable

A multi-model coding agent CLI framework built with Bazel, Rust, and TypeScript. HeliosCLI provides a unified interface for integrating coding agents from OpenAI Codex, Claude, Gemini, and other AI models with a local sandboxing and execution engine.

## Purpose

HeliosCLI is the execution layer for AI-powered coding agents. It abstracts model complexity, providing a consistent CLI interface for code generation, review, refactoring, and debugging tasks across multiple AI providers while maintaining code safety through sandboxing.

## Architecture

```
heliosCLI/
├── codex-rs/          # Rust execution core (engines, protocols, sandboxing)
├── codex-cli/         # TypeScript CLI (commands, user interface)
├── helios-rs/         # Helios-specific extensions
├── sdk/               # SDKs for agent integration
├── scripts/           # Build and CI automation
└── docs/              # Documentation and guides
```

## Quick Start

```bash
# Build everything
bazel build //...

# Run tests
bazel test //...

# Run CLI
bazel run //codex-cli:codex -- --help
```

## Supported Models

- OpenAI Codex (cloud-based agent)
- Claude (Anthropic)
- Gemini (Google)
- Cursor
- GitHub Copilot

## Features

- **Multi-Model Integration** — Switch between AI providers at runtime
- **Local Sandboxing** — Execute code safely with resource limits
- **Code Review** — Automated code review and refactoring
- **Task Execution** — Code generation, debugging, testing workflows
- **CLI Interface** — Rich terminal experience with status and metrics

## Project Status

- **Status**: Active
- **Build System**: Bazel
- **Languages**: Rust (core) + TypeScript (CLI)
- **Part of**: Helios family of tools
- **Integration**: Integrated with thegent agent orchestration

## Configuration

HeliosCLI uses environment variables for configuration:

```bash
# OpenAI
export OPENAI_API_KEY=sk-...
export OPENAI_ORG_ID=org-...

# Anthropic (Claude)
export ANTHROPIC_API_KEY=sk-ant-...

# Google (Gemini)
export GOOGLE_API_KEY=...

# GitHub (Copilot)
export GITHUB_TOKEN=ghp_...

# Sandbox configuration
export SANDBOX_RUNTIME=docker        # docker, orbstack, podman
export SANDBOX_IMAGE=ubuntu:22.04
export EXECUTION_MODE=local          # local, local_sandbox, remote
```

## Development

```bash
# Build all targets
bazel build //...

# Run specific CLI command
bazel run //codex-cli:codex -- --help

# Run tests
bazel test //...

# Build with Cargo (alternative)
cargo build --release -p codex-rs
cargo build --release -p codex-cli

# Development server
cargo run --bin codex-cli -- server --port 8080
```

## Key Components

| Component | Language | Purpose |
|-----------|----------|---------|
| `codex-rs` | Rust | Execution engine, sandboxing, provider abstraction |
| `codex-cli` | TypeScript | User-facing CLI and command interface |
| `sdk/` | TypeScript/Rust | Integration libraries for agents |
| `scripts/` | Bash/Python | Build automation and CI glue |

## Supported Models

- **Claude** (Anthropic) — Recommended for code tasks
- **GPT-4** (OpenAI) — General-purpose code generation
- **Gemini** (Google) — Fast, cost-effective analysis
- **Copilot** (GitHub) — GitHub-integrated coding
- **Cursor** — Editor-integrated assistant

## Task Types

HeliosCLI supports diverse coding tasks:

- **Code Generation** — Write new code from specifications
- **Code Review** — Analyze code for bugs, style, performance
- **Refactoring** — Modernize and improve code quality
- **Debugging** — Identify and fix bugs with traces
- **Testing** — Generate test cases and validation
- **Documentation** — Write and improve code documentation

## Security & Sandboxing

- **Local Execution** — Run code safely in isolated containers
- **Resource Limits** — CPU, memory, and disk quotas per task
- **Network Isolation** — Controlled network access with allowlists
- **Audit Trail** — Complete execution history and logs
- **Secret Management** — Encrypted credential handling

## Integration Patterns

### As an Agent Harness

```bash
# Use with Claude Code
export ANTHROPIC_API_KEY=sk-...
codex run --agent claude-code --task "write hello world in Rust"

# Use with OpenHands
codex run --agent openhands --task "debug failing test"
```

### As a Library

```rust
use codex_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let client = CodexClient::new()?;
    let result = client.run_task(task).await?;
    println!("{}", result.output);
    Ok(())
}
```

### Via HTTP API

```bash
# Start server
codex server --port 8080

# Submit task
curl -X POST http://localhost:8080/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "claude-code",
    "model": "claude-sonnet-4",
    "task": "write a fibonacci function",
    "sandbox": "docker"
  }'
```

## Testing

```bash
# Unit tests
bazel test //codex-rs/core:tests

# Integration tests
bazel test //integration:all

# Coverage
bazel test //... --instrumentation_filter='//codex' --collect_code_coverage
```

## Troubleshooting

### Sandbox Not Available
```bash
# Check Docker/Podman
docker ps
podman ps

# List available runtimes
codex runtime list

# Force specific runtime
codex run --execution-mode local --sandbox-runtime docker
```

### API Key Issues
```bash
# Verify credentials
codex auth test --provider anthropic

# Troubleshoot provider
codex auth diagnose
```

## Documentation

- **[Quick Start](docs/guides/quick-start.md)** — Get started in 5 minutes
- **[Architecture](docs/architecture/)** — System design and patterns
- **[Contributing](docs/contributing.md)** — Development setup
- **[Building](docs/install.md)** — Build system and dependencies
- **[API Reference](docs/reference/api.md)** — HTTP API documentation
- **[Governance](CLAUDE.md)** — Agent contract and development rules

## Project Status

- **Status**: Active
- **Build System**: Bazel (primary), Cargo/pnpm (secondary)
- **Languages**: Rust (core) + TypeScript (CLI)
- **Part of**: Helios family of tools
- **Integration**: Works with thegent agent orchestration
- **Release**: CalVer versioning (YEAR.MONTH.PATCH)

## References

- **Governance**: See `CLAUDE.md` for agent contract and development rules
- **CI/CD**: See `.github/workflows/` for policy gates and release automation
- **Worklogs**: Audit trail in `worklogs/` (if present)
- **Collection**: Homepage at `helios-cli.kooshapari.com`
- **Related**: Part of Phenotype Ecosystem — see `projects.kooshapari.com`

## License

Licensed under the [Apache-2.0 License](LICENSE).
