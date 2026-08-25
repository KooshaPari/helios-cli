# Upstream Audit: OpenAI Codex CLI

**Date:** 2026-08-21
**Source:** [openai/codex](https://github.com/openai/codex) (Commit: 970b7f2ff4f612b8e8cd340eb6b6d789d7141dd2)

## Executive Summary
OpenAI Codex CLI is a lightweight, local coding agent built in Rust. It emphasizes security through strict sandboxing and deep git integration. It supports multiple model providers via a modular architecture and allows extensibility through MCP (Model Context Protocol) and a plugin system.

---

## 1. Supported Providers
Codex CLI is designed to be provider-agnostic while being optimized for OpenAI models.
- **OpenAI (Native):** Direct integration with `o4-mini`, `o3`, and other OpenAI models via API key or ChatGPT login.
- **Local Models:**
    - **Ollama:** Native integration for running models locally.
    - **LM Studio:** Support for LM Studio's local server.
- **Enterprise Providers:**
    - **AWS Bedrock:** Supported via Workload Identity Federation.
    - **Azure OpenAI:** Supported via Managed Identity.
- **Custom:** Any OpenAI-compatible API endpoint can be configured.

## 2. Tool System
The tool system is modular and extensible, supporting both built-in and external capabilities.
- **Built-in Tools:**
    - `shell`: Execute commands in the user's terminal.
    - `apply-patch`: Apply structured diffs (no raw `sed`/`awk` hacks).
    - `file_search`: High-performance file searching (powered by `ripgrep`).
    - `computer_use`: Automated screen interaction (experimental).
- **Extensibility:**
    - **MCP (Model Context Protocol):** Connect to any MCP server to expose custom tools and data sources.
    - **Skills:** Reusable, domain-specific workflows that can be invoked by the agent.
    - **Plugins:** Formal packaging for distributing tools and skills.
    - **Record & Replay:** Capture user actions to create repeatable automation scripts.

## 3. Sandbox Modes
Security is a core tenet, with multiple layers of isolation.
- **Local Sandbox:**
    - **Linux:** Uses `bubblewrap` (bwrap) and Landlock LSM for filesystem/process isolation.
    - **Windows:** Custom sandboxing via `windows-sandbox-rs`.
- **Permission Profiles:**
    - **Suggest (Default):** Agent proposes changes; user must approve every command and file write.
    - **Full Auto:** Agent has full access within the sandbox (network, filesystem).
    - **Auto-Review:** Agent can edit files, but runs automated checks (lint, tests) before presenting changes.
- **Cloud Environment:** Execute tasks in a remote, ephemeral container (Codex Cloud).

## 4. MCP Support
Codex CLI is a first-class citizen in the MCP ecosystem.
- **Client:** Can connect to multiple MCP servers simultaneously.
- **Server:** Can act as an MCP server, exposing its own tools to other clients.
- **Integration:** Supports MCP Resources (data) and Tools (actions).
- **Secure Tunnel:** Capability to connect to private MCP servers without exposing them to the public internet.

## 5. Session Management
Designed for continuity and context across long-running tasks.
- **Projects & Chats:** Sessions are organized by project directory and can be named/saved.
- **Local Storage:** State and history are stored locally in `.codex/`.
- **Memory:** Persistent "Memories" allow the agent to remember user preferences, coding style, and architectural decisions across sessions.
- **Computer History:** Tracks user's recent activity to provide better context.

## 6. File Operations
Capabilities for reading, writing, and analyzing the codebase.
- **Structured Patches:** Prefers `apply-patch` over direct file writes for atomic, reversible changes.
- **Bulk Operations:** Efficient handling of large codebases.
- **Artifacts Viewer:** Support for previewing non-text files (images, etc.) generated or used by the agent.
- **File Watching:** Monitors file system changes to react to external modifications.

## 7. Git Integration
Deep integration with git is fundamental to the workflow.
- **Awareness:** Automatically detects the current branch, status, and recent history.
- **Operations:**
    - **PR Creation:** `codex pr` to create pull requests with AI-generated descriptions.
    - **Code Review:** `codex review` to analyze changes.
    - **Branch Management:** Handles branching and committing with descriptive messages.
- **Worktrees:** Native support for `git worktrees` to work on multiple branches in separate directories.
- **Third-party:** Deep hooks for GitHub (primary) and GitLab (beta).

---
*Audited by Forge*
