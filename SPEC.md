# helios-cli Specification

## Overview

Upstream OpenAI Codex CLI — a coding agent that runs locally. This fork tracks the upstream TypeScript CLI and adds Phenotype integration overlays.

## Architecture

```
┌──────────────────────────────────────────────────┐
│                   CLI Entry                       │
│           (Node.js / TypeScript)                  │
├──────────────────────────────────────────────────┤
│               Core Agent Loop                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ Provider │ │ Sandbox  │ │ Session Manager  │ │
│  │ Router   │ │ Policy   │ │ (state, history) │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
├──────────────────────────────────────────────────┤
│               Execution Layer                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ File I/O │ │ Commands │ │ Patch Engine     │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
├──────────────────────────────────────────────────┤
│               Transport                           │
│  OpenAI API  │  ChatGPT OAuth  │  Local Proxy   │
└──────────────────────────────────────────────────┘
```

## Project Layout

```
codex-cli/
  src/
    cli.ts              CLI entry point (arg parsing, dispatch)
    agent/              Agent loop, tool orchestration
    providers/          Model provider abstraction
    sandbox/            Sandbox execution (seatbelt, landlock)
    session/            Session state, conversation history
    commands/           Command implementations
    apply/              Patch/diff application engine
  tests/                Unit and integration tests
  docs/                 Upstream documentation
```

## Core Components

| Component     | Responsibility                                         |
| ------------- | ------------------------------------------------------ |
| CLI Entry     | Argument parsing, subcommand dispatch, help output     |
| Agent Loop    | Multi-turn conversation, tool call orchestration       |
| Provider      | OpenAI API client, model selection, token management   |
| Sandbox       | Platform-specific sandboxing (macOS seatbelt, Linux landlock) |
| Session       | Conversation history, session persistence, resume/fork |
| Patch Engine  | Diff generation, application, conflict resolution      |
| File I/O      | Safe file read/write within sandbox boundaries         |

## Data Model

| Entity       | Key Fields                                              |
| ------------ | ------------------------------------------------------- |
| Session      | id, messages[], model, createdAt, status                |
| Message      | role (user/assistant/system/tool), content, toolCalls[] |
| ToolCall     | id, name, arguments, result, status                     |
| Config       | model, approvalPolicy, sandboxMode, profiles            |
| Patch        | hunks[], filePath, status (pending/applied/failed)      |

## Configuration

```toml
[profile.default]
model = "gpt-4o"
approval_policy = "on-request"
sandbox_mode = "workspace-write"

[profile.ci]
model = "gpt-4o-mini"
approval_policy = "auto-edit"
```

CLI overrides: `helios-cli -c model=gpt-4o -c approval_policy=auto-edit`

## CLI Commands

| Command                         | Description                        |
| ------------------------------- | ---------------------------------- |
| `helios-cli`                    | Interactive TUI mode               |
| `helios-cli exec <prompt>`      | Non-interactive execution          |
| `helios-cli exec --json <p>`    | JSON output for scripting          |
| `helios-cli review --pr <n>`    | Code review for a PR               |
| `helios-cli resume [--last]`    | Resume a previous session          |
| `helios-cli fork <session-id>`  | Fork from a previous session       |
| `helios-cli login`              | OAuth device code flow             |
| `helios-cli sandbox <platform>` | Run command in sandbox             |
| `helios-cli apply`              | Apply latest generated diff        |
| `helios-cli completion <shell>` | Generate shell completions         |

## Sandbox Modes

| Platform | Mechanism                      |
| -------- | ------------------------------ |
| macOS    | Seatbelt (`sandbox-exec`)      |
| Linux    | Landlock + seccomp             |
| Windows  | Restricted token (WSL2)        |

## Performance Targets

| Operation           | Target        |
| ------------------- | ------------- |
| CLI startup         | < 500ms       |
| First token latency | < 2s          |
| Patch apply         | < 100ms       |
| Session resume      | < 1s          |
| Sandbox spawn       | < 200ms       |

## Quality Requirements

- TypeScript strict mode
- All tests pass before merge
- Zero lint warnings
- Upstream sync via `git fetch upstream && git merge upstream/main`
