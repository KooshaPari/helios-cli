# Known Issues: Remove Ambient OpenAI API Key Fallback from Realtime Auth

## Pre-Existing Issues (Unrelated to This Change)

### 1. `code-mode` V8 `PinScope` Compilation Failure

**Severity:** Build-breaking, pre-existing.

The `codex-code-mode` crate fails to compile with errors about `v8::PinScope` not being found. This is a V8 API version mismatch unrelated to the realtime auth change.

**Impact:** Cannot run the full `codex-core` test suite via `cargo test --test all` because the workspace-wide build fails at `codex-code-mode`.

**Workaround:** Test individual packages in isolation:

```bash
cargo test -p codex-core --test all ...
```

Note: `cargo check -p codex-core` succeeds independently and confirms the realtime auth change is syntactically correct.

**Fix owner:** `code-mode` / V8 dependency owner. Not in scope for this session.

---

## Risks Introduced by This Change

### 2. Users Who Relied on Ambient `OPENAI_API_KEY` for Realtime

**Severity:** Medium.

Some users may have been using `OPENAI_API_KEY` as the sole authentication mechanism for realtime conversations (e.g., setting it in their shell profile or CI environment). After this change, those sessions will fail with `"realtime conversation requires API key auth"`.

**Mitigation:**
- The error message is clear and actionable.
- Users can migrate to explicit API-key auth configuration via the Codex auth management system.
- The `login` crate's onboarding flow (`tui/onboarding/auth.rs`, `tui_app_server/onboarding/auth.rs`) already guides users through explicit auth setup.

**Risk level:** Low. This was a hidden, unintended behavior that should not have been relied upon in production.

### 3. Other Call Sites of `read_openai_api_key_from_env` Still Read Env

**Severity:** Low.

The `read_openai_api_key_from_env()` helper is still present and used in:
- `login/src/auth/manager.rs` (login crate -- expected)
- `tui/src/onboarding/auth.rs` (TUI onboarding -- expected)
- `tui_app_server/src/onboarding/auth.rs` (TUI app server onboarding -- expected)

These are onboarding and login flows where reading from the environment is intentional and correct. They are out of scope for this session.

**Risk level:** Low. The realtime conversation path is the only place where this fallback was inappropriate.

---

## Open Questions

### 4. Does `auth_env_telemetry` Need Adjustment?

The `auth_env_telemetry.rs` module reports telemetry events containing `openai_api_key_present: true` when `read_openai_api_key_from_env()` succeeds. After this change, this telemetry may over-report ambient env key presence for realtime scenarios without providing realtime context.

**Recommendation:** Add a context tag (`auth_source: env | session`) to the telemetry event so the `openai_api_key_present` flag can be filtered by source. This is a follow-up task and out of scope for this session.

### 5. Session Auth vs. Realtime Auth Convergence

The broader goal (per the `TODO(aibrahim)` comment) is to make realtime auth not require API key auth for ChatGPT/SIWC sessions. This change removes the env-fallback workaround but does not implement the actual convergence. After this change, ChatGPT-auth sessions are fully locked out of realtime until the auth model is extended.

**Recommendation:** File a follow-up session to implement proper ChatGPT-auth-based realtime session creation, replacing the current API-key requirement with session token passthrough.

---

## Summary Risk Table

| Issue | Severity | Scope | Action Required |
|-------|----------|-------|----------------|
| V8 `PinScope` build failure | High | Pre-existing, workspace-wide | Fix in `codex-code-mode` / V8 dep |
| Users relying on ambient env key | Medium | Production user impact | User migration docs |
| `auth_env_telemetry` env-source tagging | Low | Telemetry accuracy | Follow-up session |
| Realtime auth convergence (ChatGPT) | Low | Feature completeness | Follow-up session |
