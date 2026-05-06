# ADR-008: NDJSON Streaming Output Protocol

**Status:** Proposed

**Date:** 2026-05-05

## Context

Helios-CLI needs to communicate task progress, agent messages, and execution results to consuming applications (TUI, thegent dispatcher, external tools) in real-time. The current protocol uses ad-hoc JSON streaming which lacks versioning, schema validation, and clear backward compatibility guarantees.

## Decision

We adopt a structured NDJSON (Newline-Delimited JSON) streaming protocol with explicit versioning.

### Protocol Format

Each line is a valid JSON object with a required `type` field and `version` field:

```json
{"type": "message", "version": "1.0", "payload": {...}}
{"type": "chunk", "version": "1.0", "payload": {"delta": "text"}}
{"type": "tool_call", "version": "1.0", "payload": {...}}
{"type": "tool_result", "version": "1.0", "payload": {...}}
{"type": "error", "version": "1.0", "payload": {"code": "...", "message": "..."}}
{"type": "complete", "version": "1.0", "payload": {"exit_code": 0}}
```

### Message Types

| Type | Description | Required Fields |
|------|-------------|-----------------|
| `message` | Agent text message | `role`, `content` |
| `chunk` | Streaming token | `delta` |
| `tool_call` | Tool invocation request | `tool`, `arguments` |
| `tool_result` | Tool execution result | `tool`, `output`, `duration_ms` |
| `error` | Error condition | `code`, `message` |
| `complete` | Session completion | `exit_code` |
| `heartbeat` | Keepalive | `timestamp` |

### Versioning Strategy

- **Major version** (`X.0`): Breaking changes to message structure or required fields
- **Minor version** (`0.X`): Additive changes (new optional fields)
- Consumers negotiate minimum supported version via `X-HELIOS-PROTO-VERSION` header

### Backward Compatibility

1. **Unknown message types**: Must be ignored (forward compatibility)
2. **Missing optional fields**: Use sensible defaults
3. **New required fields**: Trigger version negotiation failure
4. **Deprecation**: Two-version deprecation window before removal

## Consequences

### Positive
- Clear contract between producer and consumer
- Easy to parse with line-oriented tools (jq, awk)
- Version negotiation prevents protocol mismatches
- Schema validation possible via JSON Schema

### Negative
- More verbose than binary protocols
- Requires version coordination in multi-component systems
- Potential for version drift if not enforced

### Open Questions

1. Should we support binary frames (MessagePack/BSON) for high-throughput scenarios?
2. How do we handle backpressure when consumers are slow?
3. Do we need a WebSocket transport for browser-based consumers?
