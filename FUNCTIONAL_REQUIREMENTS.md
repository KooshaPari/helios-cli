# helios-cli: Functional Requirements

## FR-EXEC: Execution Engine

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-EXEC-001 | System SHALL execute commands in a sandboxed environment with configurable permissions | E1.1 |
| FR-EXEC-002 | System SHALL support multiple AI provider backends (OpenAI, Anthropic, Google) | E1.2 |
| FR-EXEC-003 | System SHALL communicate between CLI and engine via JSON-RPC over stdio | E1.3 |
| FR-EXEC-004 | System SHALL track files created and modified during sessions | E1.4 |

## FR-CLI: CLI Frontend

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-CLI-001 | System SHALL display interactive TUI with conversation, tool use, and output | E2.1 |
| FR-CLI-002 | System SHALL support extensible command registry | E2.2 |
| FR-CLI-003 | System SHALL manage user configuration (providers, permissions, preferences) | E2.3 |
| FR-CLI-004 | System SHALL support ChatGPT account sign-in for plan-based usage | E2.4 |

## FR-PROV: Multi-Provider

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-PROV-001 | System SHALL support OpenAI models (GPT, Codex) as default provider | E3.1 |
| FR-PROV-002 | System SHALL support Anthropic Claude models | E3.2 |
| FR-PROV-003 | System SHALL support Google Gemini models | E3.3 |
| FR-PROV-004 | Tool calling SHALL work identically across all providers | E3.4 |

## FR-HELIOS: Helios Extensions

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-HELIOS-001 | System SHALL integrate with thegent for multi-agent orchestration | E4.1 |
| FR-HELIOS-002 | System SHALL support patch superset sync across forked repos | E4.2 |
| FR-HELIOS-003 | System SHALL integrate with AgilePlus for feature lifecycle management | E4.3 |

## FR-BUILD: Build and Distribution

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-BUILD-001 | `bazel build //...` SHALL compile all Rust and TypeScript targets | E5.1 |
| FR-BUILD-002 | CLI SHALL be installable via npm (`npm i -g @openai/codex`) | E5.2 |
| FR-BUILD-003 | CLI SHALL be installable via Homebrew | E5.3 |
