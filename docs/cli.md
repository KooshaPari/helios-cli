# Helios CLI Reference

**Document Version:** 1.0.0
**Last Updated:** 2026-05-05

A complete reference for all `helios` CLI subcommands, flags, and options.

## Global Flags

These flags apply to every subcommand:

| Flag | Description |
|------|-------------|
| `-c <key>=<value>` | Override a config TOML key (repeatable) |
| `--profile <name>` | Use a named config profile |
| `--enable <feature>` | Enable a feature flag (repeatable) |
| `--disable <feature>` | Disable a feature flag (repeatable) |
| `-m, --model <model>` | Select the AI model to use |
| `--oss` | Use the OSS (local/open-source) provider |
| `--sandbox <mode>` | Set sandbox mode (`off`, `read-only`, `workspace-write`) |
| `--full-auto` | Run in fully-automated mode (no prompts) |
| `--dangerously-bypass-approvals-and-sandbox` | Skip all approval prompts and sandboxing |
| `--search` | Enable live web search |
| `-i, --image <path>` | Attach an image file to the prompt (repeatable) |
| `-C, --cwd <dir>` | Set the working directory |
| `-p, --profile <name>` | Use a named config profile |
| `--ask-for-approval <mode>` | When to ask for approval (`on-request`, `always`, `never`) |
| `--output-format <format>` | Output format (`text`, `json`) |

## Interactive Mode (default)

```bash
helios [OPTIONS] [PROMPT]
```

When run without a subcommand, Helios starts the interactive TUI. Pass a prompt as a positional argument to run a single-shot task non-interactively.

**Examples:**
```bash
helios "Write a hello world in Rust"
helios -m claude-sonnet-4-20250514 "Review this PR diff"
helios --full-auto --sandbox workspace-write "Fix the failing tests"
```

---

## Subcommands

### `helios exec` (alias: `helios e`)

Run Helios non-interactively with a task prompt.

```bash
helios exec [OPTIONS] [--] [TASK]
```

**Examples:**
```bash
helios exec "Add authentication to this API"
helios exec --json --model gpt-5.1-test "Generate tests for auth.rs"
helios exec --resume sid-123 "Continue the previous task"
```

### `helios review`

Run a code review non-interactively.

```bash
helios review [OPTIONS] [--] [TASK]
```

**Examples:**
```bash
helios review "Review auth.rs for security issues"
helios review --model claude-sonnet-4-20250514 .
```

### `helios login`

Manage authentication credentials.

```bash
helios login [OPTIONS] [action]
```

**Actions:**
- (none) — Interactive login via ChatGPT OAuth
- `status` — Show current login status
- `--with-api-key` — Read API key from stdin (recommended)
- `--device-auth` — Use device code flow

**Examples:**
```bash
helios login
printenv OPENAI_API_KEY | helios login --with-api-key
helios login status
```

> **Note:** `--api-key <KEY>` is deprecated. Use `--with-api-key` instead.

### `helios logout`

Remove stored authentication credentials.

```bash
helios logout
```

### `helios mcp`

Manage external MCP (Model Context Protocol) servers.

```bash
helios mcp <subcommand>
```

**Subcommands:**

| Subcommand | Description |
|-----------|-------------|
| `helios mcp list` | List configured MCP servers |
| `helios mcp add <name> <command>` | Add an MCP server |
| `helios mcp remove <name>` | Remove an MCP server |
| `helios mcp start <name>` | Start an MCP server |
| `helios mcp stop <name>` | Stop an MCP server |

### `helios app-server`

Start the Helios app server or related tooling.

```bash
helios app-server [OPTIONS] [subcommand]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--listen <url>` | Transport endpoint (`stdio://` or `ws://IP:PORT`) |
| `--analytics-default-enabled` | Enable analytics by default (VSCode extension use case) |

**Subcommands:**

| Subcommand | Description |
|-----------|-------------|
| `helios app-server generate-ts` | Generate TypeScript bindings for the app server protocol |
| `helios app-server generate-json-schema` | Generate JSON Schema for the app server protocol |

**Examples:**
```bash
helios app-server
helios app-server --listen ws://127.0.0.1:8080
helios app-server generate-ts --out ./generated --prettier ./node_modules/.bin/prettier
helios app-server generate-json-schema --out ./schemas
```

### `helios completion`

Generate shell completion scripts.

```bash
helios completion <shell>
```

**Supported shells:** `bash`, `elvish`, `fish`, `powershell`, `zsh`

**Example:**
```bash
# Add to ~/.bashrc or ~/.zshrc
helios completion bash >> ~/.bashrc
helios completion zsh > /usr/local/share/zsh/site-functions/_helios
```

### `helios sandbox`

Run commands within a Helios-provided sandbox.

```bash
helios sandbox <subcommand> [args]
```

**Subcommands:**

| Subcommand | Description | Platform |
|-----------|-------------|----------|
| `helios sandbox macos` / `helios sandbox seatbelt` | Run under Seatbelt MACF | macOS only |
| `helios sandbox linux` / `helios sandbox landlock` | Run under Landlock+seccomp | Linux only |
| `helios sandbox windows` | Run under Windows restricted token | Windows only |

### `helios apply` (alias: `helios a`)

Apply the latest diff produced by Helios as a `git apply` to your local working tree.

```bash
helios apply [OPTIONS]
```

### `helios resume`

Resume a previous interactive session.

```bash
helios resume [SESSION_ID] [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--last` | Resume the most recent session (no picker) |
| `--all` | Show all sessions (disable cwd filtering) |

**Examples:**
```bash
helios resume
helios resume --last
helios resume 123e4567-e89b-12d3-a456-426614174000
```

### `helios fork`

Fork a previous interactive session into a new branch.

```bash
helios fork [SESSION_ID] [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--last` | Fork the most recent session (no picker) |
| `--all` | Show all sessions |

### `helios cloud`

**[EXPERIMENTAL]** Browse tasks from Helios Cloud and apply changes locally.

```bash
helios cloud [subcommand]
```

Aliases: `helios cloud`, `helios cloud-tasks`

### `helios features`

Inspect and manage feature flags.

```bash
helios features <subcommand>
```

**Subcommands:**

| Subcommand | Description |
|-----------|-------------|
| `helios features list` | List all known features with stage and effective state |
| `helios features enable <key>` | Enable a feature in `config.toml` |
| `helios features disable <key>` | Disable a feature in `config.toml` |

**Example feature keys:**

| Key | Description |
|-----|-------------|
| `unified_exec` | Unified execution engine |
| `shell_tool` | Shell tool support |
| `web_search_request` | Live web search |

### `helios debug`

Debugging tools.

```bash
helios debug <subcommand>
```

| Subcommand | Description |
|-----------|-------------|
| `helios debug app-server send-message-v2 <msg>` | Send a message to app server V2 |

### `helios execpolicy`

Execpolicy tooling (internal).

```bash
helios execpolicy check <command>
```

Check execpolicy files against a command.

### `helios mcp-server`

Start Helios as an MCP server over stdio.

```bash
helios mcp-server
```

### `helios responses-api-proxy`

**Internal.** Run the responses API proxy.

```bash
helios responses-api-proxy [args]
```

### `helios stdio-to-uds`

**Internal.** Relay stdio to a Unix domain socket.

```bash
helios stdio-to-uds <socket_path>
```

### `helios app`

Launch the Helios desktop app (macOS only; downloads the installer if missing).

```bash
helios app
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Fatal error (error message printed to stderr) |

## Configuration

Helios is configured via `~/.helios/config.toml` (or `$HELIOS_HOME/config.toml`).

**Key config sections:**

```toml
# Provider selection
provider = "openai"  # or "anthropic", "google", "oss_provider"

[models]
default = "gpt-5.1"

# Sandbox
[execution]
sandbox_mode = "workspace-write"
max_tool_calls = 100

# Analytics
[analytics]
enabled = false
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `GOOGLE_API_KEY` | Google API key |
| `HELIOS_HOME` | Override the Helios config directory |
| `HELIOS_USE_BUN` | Use Bun as package manager (`0` or `1`) |
| `HELIOS_USE_JAQ` | Use jaq as JSON parser (`0` or `1`) |
| `HELIOS_USE_PY314` | Use Python 3.14 (`0` or `1`) |
| `HELIOS_USE_TS_NATIVE` | Use native TypeScript tools (`0` or `1`) |
