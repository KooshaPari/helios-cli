# helios-cli: Product Requirements Document

**Version:** 1.0 | **Status:** Draft | **Date:** 2026-03-25

## Product Vision

helios-cli is a multi-runtime AI coding CLI forked from OpenAI Codex CLI. It extends the original with support for multiple AI providers (Claude, Gemini, Cursor, Copilot) while maintaining the Codex CLI core (Rust exec engine + TypeScript CLI). Uses a Bazel monorepo with Rust core (`codex-rs`) and TypeScript frontend (`codex-cli`).

## Epics

### E1: Core Execution Engine (codex-rs)

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E1.1 | Sandboxed command execution | Commands run in isolated sandbox with configurable permissions |
| E1.2 | Multi-provider backend support | Engine supports OpenAI, Anthropic, Google, and custom providers |
| E1.3 | Protocol layer (app-server) | JSON-RPC protocol between CLI frontend and Rust backend |
| E1.4 | Artifact management | Track and manage files created/modified during sessions |

### E2: CLI Frontend (codex-cli)

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E2.1 | Interactive TUI with conversation display | Rich terminal UI showing conversation, tool use, and output |
| E2.2 | Command registry and execution | Extensible command system for user-facing operations |
| E2.3 | Configuration management | User config for providers, permissions, and preferences |
| E2.4 | ChatGPT account integration | Sign in with ChatGPT for Plus/Pro/Team plan usage |

### E3: Multi-Provider Integration

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E3.1 | OpenAI provider (default) | Full Codex/GPT model support |
| E3.2 | Anthropic Claude provider | Claude model support via API |
| E3.3 | Google Gemini provider | Gemini model support via API |
| E3.4 | Provider-agnostic tool calling | Tool/function calling works across all providers |

### E4: Helios Extensions

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E4.1 | thegent orchestration integration | Dispatch to thegent for multi-agent coordination |
| E4.2 | Patch superset sync | Sync patches across forked repos |
| E4.3 | AgilePlus integration | Feature lifecycle management via AgilePlus gRPC |

### E5: Build and Distribution

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E5.1 | Bazel monorepo build | `bazel build //...` compiles all targets |
| E5.2 | npm distribution | `npm i -g @openai/codex` installs the CLI |
| E5.3 | Homebrew distribution | `brew install --cask codex` installs the CLI |
