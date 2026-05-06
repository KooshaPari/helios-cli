# Validation: Remove Ambient OpenAI API Key Fallback from Realtime Auth

## Static Verification

### 1. No Remaining Uses of `read_openai_api_key_from_env` in `codex-core`

```bash
rg 'read_openai_api_key_from_env' /Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/codex-rs/core/src -g '*.rs'
# Expected: no matches (exit code 1)
```

### 2. Compilation Check

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/codex-rs
cargo check -p codex-core
# Expected: clean compilation, no errors
```

### 3. Verify the Error Message Is Unchanged

The error string `"realtime conversation requires API key auth"` must still appear exactly once in the modified function. Confirm:

```bash
rg 'realtime conversation requires API key auth' /Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/codex-rs/core/src/realtime_conversation.rs
# Expected: 1 match in realtime_api_key()
```

## Dynamic Verification (Test Execution)

### 4. Run the Inverted Regression Test

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/codex-rs
cargo test -p codex-core --test all \
  suite::realtime_conversation::conversation_start_rejects_ambient_openai_env_key_with_chatgpt_auth \
  -- --nocapture
```

**Expected outcome:**
- The test passes.
- With `OPENAI_API_KEY=env-realtime-key` injected by the subprocess harness, the realtime start still fails with the exact error `"realtime conversation requires API key auth"`.
- The test asserts exactly 1 websocket handshake occurred (no second realtime connection was made).

### 5. Verify Related Tests Are Not Broken

Run the full realtime conversation suite:

```bash
cargo test -p codex-core --test all suite::realtime_conversation -- --nocapture
```

Any test that previously relied on ambient env-key fallback will now fail with the expected error -- this is the intended behavior.

## Manual Verification Steps

### 6. Confirm `OPENAI_API_KEY` Is Not Consulted at Runtime

1. Set `OPENAI_API_KEY` in the shell: `export OPENAI_API_KEY=sk-test-fake-key`
2. Start a realtime conversation with a ChatGPT-authenticated session.
3. Observe: the session should fail with `"realtime conversation requires API key auth"` rather than silently succeeding.
4. Provide explicit API key auth and repeat: realtime conversation should succeed.

### 7. Confirm Error Wording Is Unchanged

Any error message surfaced to the user referencing realtime API key auth should still read: `"realtime conversation requires API key auth"`. Search for this string across the codebase to ensure no callers were updated to match a different wording:

```bash
rg 'realtime conversation requires API key auth' /Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/codex-rs
```

Only the `realtime_api_key()` function in `core/src/realtime_conversation.rs` should contain this string.

## Regression Checklist

| Check | Pass Criterion |
|-------|---------------|
| `read_openai_api_key_from_env` import removed from `core/src/realtime_conversation.rs` | No matches in `core/src` |
| Ambient fallback block removed | No matches for the `TODO(aibrahim)` comment in `core/src` |
| Error message unchanged | Exact string `"realtime conversation requires API key auth"` present exactly once |
| Regression test renamed and inverted | Test passes with `OPENAI_API_KEY` set but ChatGPT auth in use |
| No other call sites in `codex-core` affected | All other realtime conversation tests still compile and pass |
| `cargo check -p codex-core` clean | Zero errors |
