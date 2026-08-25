# Claude Code - Competitor Feature Audit

**Date:** 2026-08-21
**Source:** https://docs.anthropic.com/en/docs/claude-code
**Purpose:** Catalog competitor features for Helios CLI strategic positioning

---

## 1. Tool System

Claude Code ships with a rich set of **built-in tools** that the model calls via function-calling. Every tool has explicit permission requirements (prompt-or-auto).

| Tool | Purpose | Permission |
|------|---------|------------|
| **Read** | Read files from filesystem | No (within working dir) |
| **Write** | Create/overwrite files | Yes (prompt) |
| **Edit** | Exact string replacement in files | Yes (prompt) |
| **Bash** | Execute shell commands | Yes (read-only cmds auto-approve) |
| **Glob** | Find files by pattern | No (within working dir) |
| **Grep** | Regex search file contents | No (within working dir) |
| **LSP** | Language server protocol queries | No |
| **WebFetch** | Fetch URL content as markdown | Yes (prompt) |
| **WebSearch** | Search the web | Yes (prompt, session-limited) |
| **Task** | Launch subagent for complex tasks | Yes (prompt) |
| **AskUserQuestion** | Prompt user for input | No |
| **NotebookEdit** | Edit Jupyter notebooks | Yes (prompt) |
| **Monitor** | WebSocket-based live monitoring | No |
| **PowerShell** | Windows PowerShell execution (preview) | Yes (prompt) |

### Key Design Points
- **Permission modes**: Manual (user approves each tool call), Auto (classifier decides), Plan (read-only).
- **Hooks system**: PreToolUse / PostToolUse hooks let teams inject custom logic (linting, validation, notifications) around any tool call.
- **Tool configuration**: Permission rules in settings, hooks, or subagent frontmatter control which tools are available.
- **MCP extensibility**: Custom tools added by connecting MCP servers.
- **Skills**: Reusable prompt-based workflows that run through the existing `Skill` tool (not a new tool entry).

---

## 2. File Operations

Claude Code has comprehensive filesystem capabilities:

- **Read**: Reads files up to 2000 lines by default; supports `range` for partial reads; can read images (PNG/JPG), PDFs (base64-encoded), and Jupyter notebooks (.ipynb as JSON).
- **Write**: Overwrites files; requires explicit `read` before editing existing files; supports `overwrite: true` flag.
- **Edit (Patch)**: Exact string replacement with unique-match requirement; `replace_all` option for global renames; `multi_patch` tool for batch edits.
- **Glob**: File pattern matching using ripgrep glob syntax (e.g., `*.ts`, `**/*.tsx`).
- **Grep**: Full regex search via ripgrep with `-A`, `-B`, `-C` context lines, multiline mode, type filtering, output modes (content, files_with_matches, count).
- **NotebookEdit**: Direct Jupyter notebook cell editing.
- **Checkpointing**: Built-in file checkpointing - every file edit is tracked and can be rewound. Users can revert to any previous state.
- **Remove**: File deletion with undo support.
- **Undo**: Revert the most recent file operation.

### Path Handling
- Absolute paths required for Read/Write/Edit
- Working directory scoping for permission checks
- Supports Windows (cmd.exe), macOS, and Linux path conventions

---

## 3. Git Integration

Claude Code has deep git integration through its Bash tool and dedicated skills:

### Built-in Capabilities
- Full git operations via Bash tool (clone, branch, commit, push, pull, merge, rebase, stash, etc.)
- Git status awareness - reads current branch, dirty state, recent commits
- Git log parsing for understanding change history
- Branch management and switching
- PR creation and management via GitHub CLI (`gh`)

### Dedicated Skills
- **github-pr-description**: Analyzes git diff and commit history to auto-generate comprehensive PR descriptions via `gh` CLI.
- **git-aware search**: Searches respect gitignore, understands repository structure.

### CI/CD Integration
- Can monitor CI check runs and workflows
- Fetches and parses GitHub Actions workflow results
- Code review workflows (Coderabbit, manual reviews)
- Release management and artifact tracking

### Advanced Patterns
- Worktree isolation for parallel branch work
- Cross-session git state management
- Automated commit message generation
- Conflict resolution assistance

---

## 4. MCP (Model Context Protocol) Support

MCP is a first-class feature - Claude Code is both an MCP **client** and can host MCP **servers**.

### Client Capabilities
- **Server types**: HTTP (streamable), SSE, stdio (local), WebSocket
- **Installation scopes**: Local (per-machine), Project (.mcp.json), User (~/.claude/)
- **Authentication**: OAuth 2.0 for remote servers, custom headers, pre-configured credentials
- **Dynamic tools**: Hot-reload - tools update without restart when MCP server adds/removes them
- **Notification streams**: v2 runtime supports real-time tool list updates
- **Automatic reconnection**: Handles server downtime gracefully
- **Push messages via channels**: External events can be pushed to Claude through MCP channels
- **Background tool calls**: Long-running MCP tools auto-background

### Server Management
- `claude mcp add` / `claude mcp remove` CLI commands
- Server status monitoring (connected, error, disabled states)
- Project-level server approval and workspace trust
- Disable servers without removing configuration
- Environment variable expansion in `.mcp.json`

### Practical Examples
- GitHub integration for code reviews
- PostgreSQL database queries
- Figma design access
- Linear issue tracking
- Any MCP-compatible tool

---

## 5. Memory System

Claude Code has a sophisticated two-tier persistent memory system:

### CLAUDE.md Files (User-Authored Instructions)
- **Project root**: `CLAUDE.md` in repo root (shared with team via git)
- **Subdirectories**: `CLAUDE.md` files in subdirs for scope-specific instructions
- **User-level**: `~/.claude/CLAUDE.md` (personal preferences across all projects)
- **Organization-wide**: Deployed via admin settings
- **Imports**: `@path/to/file.md` syntax to pull in additional files
- **AGENTS.md**: Alias/alternate naming support
- **Loading hierarchy**: Global -> Project root -> Subdirectory -> User-level
- **Additional directories**: Can load from outside working directory

### Auto Memory (Claude-Authored Notes)
- Claude **automatically writes notes** based on user corrections and preferences
- Stored in `~/.claude/CLAUDE.md` (user-level)
- Accumulates learnings across sessions without user intervention
- Toggle enable/disable via settings
- Audit and edit via `/memory` command

### Rules System (.claude/rules/)
- **Path-specific rules**: Rules that only activate for matching file types (e.g., `*.rs`, `*.ts`)
- **Glob-based activation**: Rules scoped to file patterns
- **Symlink sharing**: Rules shared across projects via symlinks
- **User-level rules**: Apply across all projects
- **Organization rules**: Deployed centrally for team-wide standards

### Session Management
- Session persistence across conversations
- `/compact` command for context compression
- Prompt caching for repeated context
- Context window management and optimization

---

## 6. Multi-Agent / Subagent System

Claude Code has a comprehensive multi-agent architecture:

### Subagents (Task Tool)
- **Custom subagents**: Define specialized agents with Markdown files (frontmatter + instructions)
- **Built-in agent types**: Pre-configured for common tasks
- **Scope**: Project-level, user-level, or organization-wide
- **Configuration**: Model selection, tool restrictions, permission modes, MCP scoping
- **Execution modes**: Foreground (blocking) or background (fire-and-forget)
- **Concurrent limit**: Configurable max parallel subagents
- **Context isolation**: Each subagent gets its own context window
- **Auto-compaction**: Subagent contexts compress automatically
- **Resumption**: Subagents can be resumed with full context

### Agent Teams
- **Parallel execution**: Run multiple subagents simultaneously
- **Cross-session messaging**: Agents communicate across sessions
- **Worktree isolation**: Each agent works in its own git worktree

### Fork Conversations
- **Fork mode**: Split current conversation into parallel branches
- **Observe and steer**: Monitor running forks, provide guidance
- **Distinct from subagents**: Forks share more context with parent

### Workflows
- **Dynamic workflows**: Multi-step orchestrated agent pipelines
- **Goal-based execution**: Define objectives, agents plan and execute
- **Scheduled tasks**: Cron-like scheduling for recurring agent work

### Agent SDK (Programmatic)
- **TypeScript and Python SDKs**: Embed Claude Code as a library
- **Full tool access**: Same tools as CLI
- **Agent loop**: Implement your own or use provided loop
- **Custom tools**: Extend with programmatic tools
- **MCP in SDK**: Connect MCP servers from SDK code
- **Streaming**: Real-time response streaming
- **Structured output**: Get typed JSON responses
- **Sessions**: Persistent session management
- **Cost tracking**: Built-in usage and cost observability
- **OpenTelemetry**: Full tracing and monitoring support
- **Hooks in SDK**: Intercept and control agent behavior

### Platform Surfaces
- **Terminal CLI**: Primary interface
- **VS Code extension**: Full IDE integration
- **JetBrains IDEs**: IntelliJ, WebStorm, etc.
- **Desktop app**: Standalone application
- **Web (claude.ai/code)**: Browser-based
- **Mobile**: iOS/Android access
- **Chrome extension**: Browser-native
- **Computer use (preview)**: GUI automation
- **Slack integration**: Team collaboration
- **Remote Control**: API for external orchestration

---

## 7. Pricing Model

Claude Code requires a **Claude subscription** or **Anthropic Console** (API) account.

### Subscription Tiers (from docs navigation)
- **Free tier**: Limited access
- **Pro plan**: Individual developer subscription
- **Max plan**: Higher limits, extended thinking
- **Team plan**: Organization features, admin controls
- **Enterprise**: Custom deployment, SSO, audit logs

### API-Based Usage (Anthropic Console)
- Pay-per-token pricing via Anthropic API
- Claude Code supports third-party model providers (OpenAI, etc.) via config
- Model selection per session/subagent

### Free vs Paid Features
- **Auto mode** (classifier-based permission): Pro, Max, and Team plans
- **Subagents**: Available on paid plans
- **Agent SDK**: Available on paid plans
- **Prompt caching**: Available to reduce costs on repeated context

### Notable Pricing Characteristics
- Credits-based system for API usage
- Different rate limits per plan tier
- Usage tracking and cost visibility in dashboard
- No per-seat pricing for Team (usage-based)

---

## 8. Additional Notable Features

### Security & Governance
- **Permission modes**: Manual, Auto, Plan (read-only)
- **Workspace trust**: Per-directory trust levels
- **Hooks for enforcement**: PreToolUse hooks can block actions regardless of AI decision
- **Admin controls**: Organization-wide settings deployment
- **Audit logging**: Full action history

### Developer Experience
- **Interactive mode**: Rich terminal UI with slash commands
- **Headless mode**: `claude -p "query"` for scripting/CI
- **Deep links**: Launch sessions from URLs
- **Prompt library**: Community prompt templates
- **Artifacts**: Share session outputs as shareable artifacts
- **Skills system**: Reusable prompt-based workflow library
- **Plugins**: Discover and install prebuilt extensions

### Code Intelligence
- **Context window**: Large context with automatic summarization
- **Prompt caching**: Repeated context cached for cost/speed
- **LSP integration**: Language server queries for type info
- **Glob/Grep**: Fast codebase search via ripgrep
- **Large codebase support**: Optimized for monorepos

### Extensibility
- **Plugins**: Pre-built community plugins
- **Skills**: Custom workflow definitions
- **MCP servers**: Unlimited tool extension
- **Agent SDK**: Full programmatic embedding
- **Channels**: External event push to Claude
- **Webhooks**: HTTP callbacks for integration

---

## 9. Competitive Implications for Helios CLI

### Strengths to Match
1. **MCP as first-class**: MCP integration is deep and well-documented; Helios should support this or comparable extensibility
2. **Multi-agent orchestration**: Subagents, teams, worktrees, cross-session messaging is sophisticated
3. **Memory system**: Two-tier CLAUDE.md + auto memory is elegant and practical
4. **Platform breadth**: Terminal, IDE, web, mobile, desktop, Chrome, Slack - massive surface area
5. **Agent SDK**: Programmatic embedding in TypeScript/Python makes Claude Code embeddable

### Differentiation Opportunities
1. **Open source / self-hosted**: If Helios is open-source or self-hostable, that's a key differentiator vs Anthropic's hosted model
2. **Model flexibility**: Support for any LLM provider without vendor lock-in
3. **Pricing**: Usage-based API pricing may be more cost-effective than subscription tiers for teams
4. **Local-first**: If Helios can run fully offline or with local models, that's a privacy/security advantage
5. **Specialization**: Claude Code is general-purpose; Helios could specialize in specific domains (e.g., specific frameworks, industries)

### Feature Gaps to Address
1. No built-in checkpointing/undo system mentioned (Claude Code has this)
2. No mention of Jupyter notebook support
3. No mention of computer use / GUI automation
4. No mention of Chrome extension or Slack integration
5. No equivalent to Claude Code's plugin marketplace

---

*Audit compiled from official Anthropic documentation at docs.anthropic.com/en/docs/claude-code as of 2026-08-21.*
