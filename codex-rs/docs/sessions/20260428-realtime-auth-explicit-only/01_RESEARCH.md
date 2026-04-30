# Research

## Local Code Search
- `core/src/realtime_conversation.rs` contained the only use of `read_openai_api_key_from_env()`.
- `core/tests/suite/realtime_conversation.rs` already had a subprocess-based env-fallback test, which is the right place for the regression coverage.

## Decision
- Remove the fallback outright.
- Reuse the existing test shape and invert its expectation so the env var no longer helps.

