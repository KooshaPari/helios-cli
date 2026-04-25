# HeliosCLI

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

## Documentation

- **Contributing**: See `docs/contributing.md`
- **Building**: See `docs/install.md`
- **Governance**: See `CLAUDE.md` for agent contract
- **Worklogs**: Audit trail in `docs/worklogs/` (if present)

This repository is licensed under the [Apache-2.0 License](LICENSE).
