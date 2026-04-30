# Codex MCP Server Interface

This document describes the experimental MCP interface used by Codex clients to control a local
Codex engine.

- Status: experimental and subject to change
- Server binary: `codex mcp-server` or `codex-mcp-server`
- Transport: MCP over stdio / JSON-RPC

The authoritative protocol types live in `app-server-protocol/src/protocol/{common,v1,v2}.rs` and
the server wiring lives in `app-server/`.

## Core surface

Use the v2 thread and turn APIs for new integrations:

- `thread/start`, `thread/resume`, `thread/fork`, `thread/read`, `thread/list`
- `turn/start`, `turn/steer`, `turn/interrupt`
- `review/start`
- `model/list`, `collaborationMode/list`, `app/list`
- `config/read`, `config/value/write`, `config/batchWrite`
- `command/exec`
- `fs/readFile`, `fs/writeFile`, `fs/createDirectory`, `fs/getMetadata`, `fs/readDirectory`, `fs/remove`
- `plugin/list`, `plugin/read`

Legacy compatibility methods remain available for older clients:

- `getConversationSummary`
- `getAuthStatus`
- `gitDiffToRemote`
- `fuzzyFileSearch`, `fuzzyFileSearch/sessionStart`, `fuzzyFileSearch/sessionUpdate`, `fuzzyFileSearch/sessionStop`

## Starting the server

Run Codex as an MCP server and connect any compatible client:

```bash
codex mcp-server | your_mcp_client
```

For inspection, use the MCP inspector:

```bash
npx @modelcontextprotocol/inspector codex mcp-server
```

## Threads and turns

- `thread/start` opens a new thread and emits `thread/started`
- `thread/resume` reopens a persisted thread
- `thread/fork` creates a new thread from stored history
- `turn/start` begins generation and streams turn/item notifications
- `turn/steer` injects more input into a live regular turn
- `turn/interrupt` cancels an in-flight turn

`thread/list` supports filters such as `archived`, `cwd`, `searchTerm`, `modelProviders`, and
`sourceKinds`. `thread/read` can include turns on demand.

## Models and collaboration modes

`model/list` returns available models, reasoning efforts, and optional upgrade metadata.

`collaborationMode/list` returns built-in presets. When sending `turn/start` with a collaboration
mode, `settings.developer_instructions: null` means use the built-in instructions for that mode.

## Event stream

The server emits:

- `thread/started`
- `thread/status/changed`
- `turn/completed`
- `account/login/completed`
- `codex/event/*`
- `fuzzyFileSearch/sessionUpdated`
- `fuzzyFileSearch/sessionCompleted`

## Approvals

When Codex needs to apply a patch or run a command, the server sends approval requests to the
client:

- `applyPatchApproval`
- `execCommandApproval`

The client responds with `allow` or `deny`.

## Auth helpers

See the app-server README for the auth endpoint flow and request/response shapes.

## Stability

The interface is experimental. Consult the protocol definitions and the app-server README for the
current canonical behavior.
