# helios-cli: Architecture Decision Records

## ADR-001: Fork of OpenAI Codex CLI

- **Status:** Accepted
- **Context:** Need a multi-runtime AI coding CLI; Codex CLI provides excellent foundation.
- **Decision:** Fork openai/codex, extend with multi-provider support and Phenotype integrations.
- **Rationale:** Codex CLI has production-quality sandbox, TUI, and tool-calling infrastructure.

## ADR-002: Bazel Monorepo Build System

- **Status:** Accepted
- **Context:** Rust (codex-rs) and TypeScript (codex-cli) in one repo need unified build.
- **Decision:** Bazel as primary build system with rules_rust and rules_nodejs.
- **Rationale:** Bazel provides hermetic builds, caching, and cross-language dependency management.
- **Alternatives:** Cargo + pnpm separately (no unified build graph), Nx (poor Rust support).

## ADR-003: Rust Core for Execution Engine

- **Status:** Accepted (inherited from Codex)
- **Context:** Command execution needs sandboxing, performance, and safety.
- **Decision:** Rust workspace (codex-rs) for exec engine, protocol, sandbox, and backend clients.
- **Rationale:** Rust provides memory safety, performance, and platform-native sandboxing.

## ADR-004: TypeScript CLI Frontend

- **Status:** Accepted (inherited from Codex)
- **Context:** Rich TUI with conversation display and interactive command input.
- **Decision:** TypeScript CLI (codex-cli) using Ink for React-based TUI rendering.
- **Rationale:** Ink provides React component model for terminal UIs; good developer experience.

## ADR-005: JSON-RPC App Server Protocol

- **Status:** Accepted (inherited from Codex)
- **Context:** CLI frontend and Rust backend need a clean IPC boundary.
- **Decision:** JSON-RPC over stdio between TypeScript CLI and Rust app-server.
- **Rationale:** Simple, debuggable, language-agnostic protocol; easy to add new frontends.

## ADR-006: Multi-Provider Architecture

- **Status:** Accepted
- **Context:** Need to support OpenAI, Anthropic, Google, and custom providers.
- **Decision:** Provider-agnostic backend client interface in codex-rs; provider selection at config time.
- **Rationale:** Allows switching models without changing tool-calling or sandbox logic.
