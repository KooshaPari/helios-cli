# codex-app-server

`codex app-server` is the interface Codex uses to power rich clients such as the Codex VS Code
extension and other local integrations.

## Overview

The app server exposes a bidirectional JSON-RPC API over MCP-style transports. It manages:

- Threads: persistent conversations stored on disk
- Turns: individual agent runs within a thread
- Items: persisted user and agent events that make up a turn
- Approvals: client-side allow/deny requests for commands and patch application

Supported transports:

- stdio (`--listen stdio://`, default)
- websocket (`--listen ws://IP:PORT`, experimental / unsupported)

When running over websocket, the listener also serves `/readyz` and `/healthz` probes.

## Lifecycle

1. Send `initialize` once per connection.
2. Emit `initialized` after the handshake succeeds.
3. Start, resume, or fork a thread.
4. Start a turn with `turn/start`.
5. Stream notifications until `turn/completed`.

Thread creation and resume flow:

- `thread/start` creates a new thread and emits `thread/started`
- `thread/resume` reopens an existing thread
- `thread/fork` copies an existing thread into a new one
- `thread/start` and `thread/fork` both support `ephemeral: true` for in-memory threads

Turn flow:

- `turn/start` begins generation for a thread
- `turn/steer` appends input to an already in-flight regular turn
- `turn/interrupt` stops an in-flight turn

## API surface

### Thread APIs

- `thread/start`
- `thread/resume`
- `thread/fork`
- `thread/list`
- `thread/read`
- `thread/archive`
- `thread/unarchive`
- `thread/unsubscribe`
- `thread/name/set`
- `thread/metadata/update`
- `thread/rollback`
- `thread/compact/start`
- `thread/backgroundTerminals/clean`

### Turn and review APIs

- `turn/start`
- `turn/steer`
- `turn/interrupt`
- `review/start`

### Command and filesystem utilities

- `command/exec`
- `command/exec/write`
- `command/exec/resize`
- `command/exec/terminate`
- `fs/readFile`
- `fs/writeFile`
- `fs/createDirectory`
- `fs/getMetadata`
- `fs/readDirectory`
- `fs/remove`
- `fs/copy`
- `fs/watch`
- `fs/unwatch`

### Models, skills, apps, and plugins

- `model/list`
- `experimentalFeature/list`
- `experimentalFeature/enablement/set`
- `collaborationMode/list`
- `skills/list`
- `skills/changed`
- `skills/config/write`
- `app/list`
- `plugin/list`
- `plugin/read`
- `plugin/install`
- `plugin/uninstall`

### Auth and config

- `mcpServer/oauth/login`
- `config/mcpServer/reload`
- `config/read`
- `config/value/write`
- `config/batchWrite`
- `configRequirements/read`

### Miscellaneous

- `tool/requestUserInput`
- `feedback/upload`
- `externalAgentConfig/detect`
- `externalAgentConfig/import`
- `mcpServerStatus/list`
- `windowsSandbox/setupStart`

## Initialization

Clients must send a single `initialize` request before any other method on the connection. Repeated
`initialize` calls are rejected. Use `clientInfo` to identify the integration.

`initialize.params.capabilities.optOutNotificationMethods` can suppress specific notifications for
that connection.

## Thread and turn semantics

- `thread/start` creates a thread and auto-subscribes the client to thread events
- `thread/fork` creates a new thread from existing history
- `thread/list` supports filters such as `archived`, `cwd`, `modelProviders`, `sourceKinds`, and `searchTerm`
- `thread/read` can return the thread with or without turns
- `turn/start` accepts collaboration mode and prompt overrides
- `turn/start` emits the streaming `item/*` and `turn/*` notification sequence

## Event stream

The server streams events and notifications while a turn is running:

- `thread/started`
- `thread/status/changed`
- `item/started`
- `item/completed`
- `item/agentMessage/delta`
- `turn/completed`
- `codex/event/*`

## Approvals

When Codex needs approval for a command or patch application, the server sends a request to the
client. The client must reply with `allow` or `deny`.

## Tool responses

Tool responses follow standard MCP `CallToolResult` semantics. For compatibility, Codex mirrors the
content payload in `structuredContent` where needed.

## Stability

This interface is experimental. Method names, fields, and event shapes may evolve. For authoritative
types, consult the protocol definitions under `app-server-protocol/src/protocol/`.
