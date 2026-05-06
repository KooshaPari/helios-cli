# ADR-011: Codex-RS API Versioning and Backwards Compatibility

**Status:** Proposed

**Date:** 2026-05-05

## Context

The `codex-rs` crate exposes public APIs consumed by the CLI, SDK, and external integrations. As the project evolves, we must maintain API stability while enabling innovation. This ADR defines our versioning strategy and compatibility guarantees.

## Decision

We adopt a hybrid versioning approach: semantic versioning for the crate with explicit API stability tiers.

### Versioning Tiers

| Tier | Visibility | Stability Guarantee | Breaking Changes |
|------|------------|-------------------|------------------|
| `public` | All consumers | Semantic versioning | Never |
| `unstable` | All consumers | Minor version only | Allowed in minor |
| `internal` | crate-internal only | None | Any time |
| `experimental` | behind feature flag | Preview, may change | Any time |

### API Stability Annotations

```rust
// Public API - stable forever
#[helios_api(stability = "public")]
pub fn run_task(task: Task) -> Result<TaskResult> { ... }

// Unstable - minor version compatible
#[helios_api(stability = "unstable")]
pub fn experimental_stream() -> impl Stream<Item = Event> { ... }

// Experimental - behind feature flag
#[helios_api(stability = "experimental", feature = "experimental_api")]
pub fn preview_feature() -> Result<()> { ... }
```

### Backwards Compatibility Rules

1. **Public API**:
   - Never remove public functions, structs, traits
   - Never change function signatures (except adding optional params)
   - Never change enum variant meanings
   - Never break existing trait implementations

2. **Unstable API**:
   - May add new items
   - May add new variants to enums (with `#[non_exhaustive]`)
   - May add new optional parameters
   - May NOT remove items
   - May NOT change signatures

3. **Internal API**:
   - No stability guarantees
   - May change arbitrarily between versions

### Rate Limiting Strategy

The codex-rs API enforces rate limits to protect upstream services:

```rust
#[helios_api(rate_limit = "provider:100/minute")]
async fn chat_complete(request: ChatRequest) -> Result<ChatResponse> {
    // Implementation
}
```

**Rate Limit Tiers:**

| Tier | Requests/minute | Burst | Scope |
|------|-----------------|-------|-------|
| `free` | 10 | 2 | Per API key |
| `standard` | 100 | 20 | Per API key |
| `enterprise` | 1000 | 100 | Per API key |
| `internal` | Unlimited | - | Service-to-service |

**Implementation:**

```rust
pub struct RateLimiter {
    limiter: Arc<RateLimit>,
    quota: Quota,
}

impl RateLimiter {
    pub async fn acquire(&self, key: &str) -> Result<Token> {
        let token = self.limiter.acquire(key).await
            .map_err(|_| RateLimitError { retry_after: self.limiter.retry_after(key) })?;
        Ok(token)
    }
}

// Per-endpoint limits
pub const ENDPOINT_LIMITS: &[(&str, RateLimitConfig)] = &[
    ("/v1/chat/completions", RateLimitConfig::Standard),
    ("/v1/completions", RateLimitConfig::Standard),
    ("/v1/embeddings", RateLimitConfig::HighVolume),
];
```

### Version Negotiation

External consumers specify minimum supported API version:

```rust
// Client specifies minimum version
let client = CodexClient::builder()
    .min_api_version("2026.5.0")
    .build()?;

// Server validates and rejects incompatible clients
if client.min_version > server.max_compatible_version {
    return Err(IncompatibleVersionError {
        client_version: client.min_version,
        server_version: server.max_compatible_version,
    });
}
```

### Deprecation Policy

1. **Announcement**: Feature marked `#[deprecated = "Use new_api instead"]`
2. **Minimum Notice**: 2 minor versions (e.g., deprecated in 2026.5, removed in 2026.7)
3. **Migration Guide**: Published alongside deprecation
4. **Tooling**: `cargo outdated` integration for dependency checks

## Consequences

### Positive
- Clear contract for external consumers
- Predictable upgrade path
- Rate limiting protects service stability
- Tooling support for version management

### Negative
- API surface restriction slows evolution
- Version negotiation adds complexity
- Rate limit tuning requires monitoring
- Documentation burden increases

### Open Questions

1. Should we offer LTS (Long Term Support) releases?
2. How do we handle breaking changes in security patches?
3. Do we need API versioning for internal microservices?
4. What is the SLA for deprecation warnings?
