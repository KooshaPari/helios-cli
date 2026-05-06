# Implementation: Remove Ambient OpenAI API Key Fallback from Realtime Auth

## Changes Made

### `core/src/realtime_conversation.rs`

**1. Remove the import (line 3)**

```diff
 use crate::CodexAuth;
 use crate::api_bridge::map_api_error;
-use crate::auth::read_openai_api_key_from_env;
 use crate::codex::Session;
```

The import `read_openai_api_key_from_env` is no longer referenced after the fallback block is removed.

**2. Remove the ambient fallback block (previously lines 662-667)**

```diff
 fn realtime_api_key(auth: &CodexAuth, provider: &ApiProvider) -> CodexResult<String> {
     if let Some(api_key) = auth.openai_api_key() {
         return Ok(api_key.to_string());
     }
-
-    // TODO(aibrahim): Remove this temporary fallback once realtime auth no longer
-    // requires API key auth for ChatGPT/SIWC sessions.
-    if provider.is_openai()
-        && let Some(api_key) = read_openai_api_key_from_env()
-    {
-        return Ok(api_key);
-    }

     Err(CodexErr::InvalidRequest(
         "realtime conversation requires API key auth".to_string(),
     ))
 }
```

The function `realtime_api_key()` now has two explicit paths:

1. `auth.openai_api_key()` -- explicit API key from session auth (already present, unchanged)
2. `Err(...)` -- explicit failure when no key is available (unchanged)

The ambient env-read path is removed entirely.

---

### `core/tests/suite/realtime_conversation.rs`

**3. Rename and invert the regression test**

```diff
-async fn conversation_start_uses_openai_env_key_fallback_with_chatgpt_auth() -> Result<()> {
+async fn conversation_start_rejects_ambient_openai_env_key_with_chatgpt_auth() -> Result<()> {
     if std::env::var_os(REALTIME_CONVERSATION_TEST_SUBPROCESS_ENV_VAR).is_none() {
         return run_realtime_conversation_test_in_subprocess(
-            "suite::realtime_conversation::conversation_start_uses_openai_env_key_fallback_with_chatgpt_auth",
+            "suite::realtime_conversation::conversation_start_rejects_ambient_openai_env_key_with_chatgpt_auth",
             Some("env-realtime-key"),
         );
     }
```

The subprocess bootstrap preserves `OPENAI_API_KEY=env-realtime-key` so the test still runs with the env var present. The new assertions verify that even with the env var, the realtime start fails.

```diff
-    let started = wait_for_event_match(&test.codex, |msg| match msg {
-        EventMsg::RealtimeConversationStarted(started) => Some(Ok(started.clone())),
-        EventMsg::Error(err) => Some(Err(err.clone())),
-        _ => None,
-    })
-    .await
-    .unwrap_or_else(|err: ErrorEvent| panic!("conversation start failed: {err:?}"));
-    assert!(started.session_id.is_some());
-
-    let session_updated = wait_for_event_match(&test.codex, |msg| match msg {
-        EventMsg::RealtimeConversationRealtime(RealtimeConversationRealtimeEvent {
-            payload: RealtimeEvent::SessionUpdated { session_id, .. },
-        }) => Some(session_id.clone()),
-        _ => None,
-    })
-    .await;
-    assert_eq!(session_updated, "sess_env");
-
-    assert_eq!(
-        server.handshakes()[1].header("authorization").as_deref(),
-        Some("Bearer env-realtime-key")
-    );
+    // After removing the ambient OPENAI_API_KEY fallback, a ChatGPT-auth session
+    // must NOT be able to start a realtime conversation even when the env var is set.
+    let err = wait_for_event_match(&test.codex, |msg| match msg {
+        EventMsg::RealtimeConversationRealtime(RealtimeConversationRealtimeEvent {
+            payload: RealtimeEvent::Error(message),
+        }) => Some(message.clone()),
+        _ => None,
+    })
+    .await;
+    assert_eq!(err, "realtime conversation requires API key auth");
+
+    // No second websocket connection should have been made.
+    assert_eq!(server.handshakes().len(), 1);
```

The test body is updated from asserting success to asserting the correct failure mode:
- Expects an `Error` event with the expected error string
- Asserts that only 1 handshake occurred (the initial test handshake), confirming no second realtime connection was attempted

---

## Verification

`cargo check -p codex-core` succeeds without errors related to the removed code. The `read_openai_api_key_from_env` import was the only use of that symbol within `codex-core`.

The full test suite cannot be compiled in this session due to unrelated `code-mode` V8 `PinScope` API incompatibilities (a pre-existing issue unrelated to this change).
