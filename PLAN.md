# helios-cli: Implementation Plan

## Phase 1: Codex Core (Inherited, Complete)

| Task | Description | Depends On |
|------|-------------|------------|
| P1.1 | Rust exec engine with sandbox | - |
| P1.2 | JSON-RPC app-server protocol | P1.1 |
| P1.3 | TypeScript CLI with TUI (Ink) | P1.2 |
| P1.4 | OpenAI provider integration | P1.1 |
| P1.5 | Bazel build system setup | - |

## Phase 2: Multi-Provider Extension (In Progress)

| Task | Description | Depends On |
|------|-------------|------------|
| P2.1 | Provider-agnostic backend client interface | P1.1 |
| P2.2 | Anthropic Claude provider | P2.1 |
| P2.3 | Google Gemini provider | P2.1 |
| P2.4 | Provider-agnostic tool calling | P2.1 |

## Phase 3: Helios-Specific Extensions

| Task | Description | Depends On |
|------|-------------|------------|
| P3.1 | thegent orchestration integration | P1.3 |
| P3.2 | Patch superset sync tooling | P1.1 |
| P3.3 | AgilePlus gRPC integration | P2.1 |

## Phase 4: Distribution and CI

| Task | Description | Depends On |
|------|-------------|------------|
| P4.1 | npm package publishing | P1.3 |
| P4.2 | Homebrew cask formula | P1.1 |
| P4.3 | Bazel CI (build + test all targets) | P1.5 |
| P4.4 | Rust CI (clippy, fmt, test) | P1.1 |
| P4.5 | Policy gate workflow | P4.3 |
