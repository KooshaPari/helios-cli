# Specifications: Remove Ambient OpenAI API Key Fallback from Realtime Auth

## Problem Statement

The `realtime_api_key()` function in `core/src/realtime_conversation.rs` contained a temporary ambient fallback that read the `OPENAI_API_KEY` environment variable when no explicit API-key auth was available. This silently bypassed the requirement for explicit realtime auth, creating a security and behavioral inconsistency.

Specifically, a session authenticated via ChatGPT auth (e.g., SIWC) could still start a realtime conversation by having `OPENAI_API_KEY` present in the process environment -- even though ChatGPT auth does not itself grant realtime API key auth.

## Scope of the Change

### What Changes

**`core/src/realtime_conversation.rs`**

- Remove the import: `use crate::auth::read_openai_api_key_from_env;`
- Remove the fallback block (lines previously ~662-667):

```rust
// TODO(aibrahim): Remove this temporary fallback once realtime auth no longer
// requires API key auth for ChatGPT/SIWC sessions.
if provider.is_openai()
    && let Some(api_key) = read_openai_api_key_from_env()
{
    return Ok(api_key);
}
```

- The `realtime_api_key()` function signature and all other callers remain unchanged.

### What Does Not Change

- The existing `InvalidRequest("realtime conversation requires API key auth")` error remains the error emitted when no API key is available. No error wording changes.
- Other providers (non-OpenAI) are unaffected; they did not use this fallback.
- All other `read_openai_api_key_from_env()` call sites in `login/`, `tui/`, and `tui_app_server/` remain unchanged. Those modules serve onboarding flows and are out of scope.
- The telemetry in `auth_env_telemetry.rs` (which reports ambient env key presence) remains in place.

## Behavior After the Change

| Scenario | Before | After |
|----------|--------|-------|
| ChatGPT auth + no OPENAI_API_KEY env | Fails with "realtime conversation requires API key auth" | Same (unchanged) |
| ChatGPT auth + OPENAI_API_KEY env set | Succeeds using env key (bug) | Fails with "realtime conversation requires API key auth" |
| Explicit API-key auth + OPENAI_API_KEY env | Uses explicit key (env not consulted) | Same (unchanged) |
| SIWC auth + OPENAI_API_KEY env set | Succeeds using env key (bug) | Fails with "realtime conversation requires API key auth" |

## Test Update

**`core/tests/suite/realtime_conversation.rs`**

Rename and invert the existing regression test:

| Before | After |
|--------|-------|
| `conversation_start_uses_openai_env_key_fallback_with_chatgpt_auth` | `conversation_start_rejects_ambient_openai_env_key_with_chatgpt_auth` |

The subprocess entry point string and all internal test scaffolding (server setup, auth builder, websocket handshake) are preserved. Only the assertions change:

- Remove: asserts successful `RealtimeConversationStarted` and `SessionUpdated` events
- Remove: asserts `Authorization: Bearer env-realtime-key` header on second handshake
- Add: asserts an `Error(RealtimeEvent::Error(...))` event with message `"realtime conversation requires API key auth"`
- Add: asserts exactly 1 handshake (no second connection was made)

## Success Criteria

1. `realtime_api_key()` no longer calls `read_openai_api_key_from_env()`.
2. The inverted regression test passes: ambient `OPENAI_API_KEY` does NOT unblock a ChatGPT-auth realtime start.
3. The error message emitted remains `"realtime conversation requires API key auth"` (unchanged wording).
4. `cargo check -p codex-core` succeeds with no errors related to the removed code.
