# Session Overview

## Goal
Remove the ambient `OPENAI_API_KEY` fallback from realtime conversation startup so realtime sessions require explicit auth or provider config.

## Success Criteria
- Realtime auth no longer reads `read_openai_api_key_from_env()`.
- A focused regression test proves an injected `OPENAI_API_KEY` does not unblock realtime startup.
- Existing error wording remains unchanged.

