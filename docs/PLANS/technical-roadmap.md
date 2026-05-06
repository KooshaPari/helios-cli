# Helios-CLI Technical Roadmap

**Version:** 1.0.0

**Last Updated:** 2026-05-05

**Planning Horizon:** 6 months (May 2026 - November 2026)

## Executive Summary

This roadmap outlines the technical evolution of Helios-CLI from a Codex CLI fork to a mature multi-model coding agent framework with plugin extensibility, enterprise-grade security, and robust session management.

---

## Phase 1: Foundation Stabilization (May 2026)

### Milestone 1.1: Dependency Health

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D1.1.1 | Audit and resolve RUSTSEC advisories | P0 | Pending |
| D1.1.2 | Update aws-lc-sys to latest | P0 | Pending |
| D1.1.3 | Update quinn-proto for CVE fixes | P0 | Pending |
| D1.1.4 | Pin GitHub Actions to commit SHAs | P0 | Pending |

### Milestone 1.2: Build System Polish

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D1.2.1 | Resolve cargo metadata drift in codex-rs | P1 | Pending |
| D1.2.2 | Validate Bazel build across all targets | P1 | Pending |
| D1.2.3 | Fix TUI HMR for development | P1 | Pending |

**Deliverables:**
- Clean CI run with all gates passing
- Sub-5-minute incremental builds

---

## Phase 2: Protocol Formalization (June 2026)

### Milestone 2.1: NDJSON Streaming Protocol

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D2.1.1 | Implement ADR-008 streaming protocol | P0 | Pending |
| D2.1.2 | Add version negotiation headers | P0 | Pending |
| D2.1.3 | Write protocol conformance tests | P1 | Pending |
| D2.1.4 | Document protocol with JSON Schema | P1 | Pending |

**Dependencies:** Phase 1 complete

### Milestone 2.2: Session Bundle Format

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D2.2.1 | Implement ADR-009 bundle format | P0 | Pending |
| D2.2.2 | Add HMAC-SHA256 signing | P0 | Pending |
| D2.2.3 | Integrate zstd compression | P0 | Pending |
| D2.2.4 | Create bundle verification CLI | P1 | Pending |

**Dependencies:** D2.1.1 complete

---

## Phase 3: Plugin Architecture (July - August 2026)

### Milestone 3.1: Plugin Discovery and Lifecycle

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D3.1.1 | Implement plugin manifest parsing | P0 | Pending |
| D3.1.2 | Build plugin lifecycle manager | P0 | Pending |
| D3.1.3 | Add plugin validation and security checks | P0 | Pending |
| D3.1.4 | Create plugin CLI commands (list, load, unload) | P1 | Pending |

**Dependencies:** Phase 2 complete

### Milestone 3.2: WASM Sandbox

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D3.2.1 | Set up WASM runtime (Wasmtime) | P0 | Pending |
| D3.2.2 | Define plugin host interface (WIT) | P0 | Pending |
| D3.2.3 | Implement sandbox security controls | P0 | Pending |
| D3.2.4 | Create sample plugin (template) | P1 | Pending |

**Dependencies:** D3.1.1 complete

### Milestone 3.3: Plugin Capabilities

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D3.3.1 | Implement tool plugins | P1 | Pending |
| D3.3.2 | Implement provider plugins | P2 | Pending |
| D3.3.3 | Add plugin repository (optional) | P2 | Pending |

**Dependencies:** D3.2.1 complete

---

## Phase 4: API Versioning (September 2026)

### Milestone 4.1: Codex-RS API Stability

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D4.1.1 | Implement ADR-011 API versioning | P0 | Pending |
| D4.1.2 | Add stability annotations to public API | P0 | Pending |
| D4.1.3 | Create API compatibility test suite | P1 | Pending |
| D4.1.4 | Document API stability tiers | P1 | Pending |

**Dependencies:** Phase 3 complete

### Milestone 4.2: Rate Limiting

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D4.2.1 | Implement rate limiter core | P0 | Pending |
| D4.2.2 | Add per-endpoint rate limits | P0 | Pending |
| D4.2.3 | Create rate limit CLI diagnostics | P1 | Pending |
| D4.2.4 | Add rate limit metrics and alerts | P1 | Pending |

**Dependencies:** D4.1.1 complete

---

## Phase 5: Enterprise Features (October - November 2026)

### Milestone 5.1: Multi-Instance Deployment

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D5.1.1 | Design shared storage layer | P0 | Pending |
| D5.1.2 | Implement session replication | P1 | Pending |
| D5.1.3 | Add Kubernetes manifests | P1 | Pending |
| D5.1.4 | Create Helm chart | P2 | Pending |

**Dependencies:** Phase 4 complete

### Milestone 5.2: Security Hardening

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| D5.2.1 | Audit and fix security findings | P0 | Pending |
| D5.2.2 | Add OPA policy integration | P1 | Pending |
| D5.2.3 | Implement session encryption (optional) | P2 | Pending |
| D5.2.4 | Add SOC2 compliance documentation | P2 | Pending |

**Dependencies:** D5.1.1 complete

---

## Dependency Graph

```
Phase 1 ─────────────────┐
   │                     │
   └──────┬──────────────┘
          │
          ▼
Phase 2 ────────────────────────────────────────────────┐
   │                                                    │
   ├──▶ Milestone 2.1 (Streaming Protocol)              │
   │        │                                           │
   │        └──────▶ Milestone 2.2 (Session Bundles)    │
   │                     │                              │
   └─────────────────────┘                              │
                    │                                   │
                    ▼                                   │
Phase 3 ────────────────────────────────────────────────┤
   │                                                    │
   ├──▶ Milestone 3.1 (Plugin Discovery)                │
   │        │                                           │
   │        ├──▶ Milestone 3.2 (WASM Sandbox)           │
   │        │        │                                  │
   │        │        └──▶ Milestone 3.3 (Capabilities)   │
   │        │                                           │
   └────────┘                                           │
                    │                                   │
                    ▼                                   │
Phase 4 ────────────────────────────────────────────────┤
   │                                                    │
   ├──▶ Milestone 4.1 (API Versioning)                   │
   │        │                                           │
   │        └──▶ Milestone 4.2 (Rate Limiting)         │
   │                                                    │
   └────────────────────────────────────────────────────┘
                          │
                          ▼
Phase 5 ────────────────────────────────────────────────┐
   │                                                    │
   ├──▶ Milestone 5.1 (Multi-Instance)                  │
   │        │                                           │
   │        └──▶ Milestone 5.2 (Security)              │
   │                                                    │
   └────────────────────────────────────────────────────┘
```

---

## Resource Requirements

| Phase | Engineering Days | Notes |
|-------|-----------------|-------|
| Phase 1 | 5 | Small team, dependency fixes |
| Phase 2 | 10 | Protocol implementation + tests |
| Phase 3 | 20 | WASM sandbox is complex |
| Phase 4 | 10 | API versioning work |
| Phase 5 | 15 | Deployment and security |
| **Total** | **60** | ~3 months of engineering |

---

## Success Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| CI Pass Rate | 85% | 99% | Weekly |
| Build Time (incremental) | ~10 min | <5 min | Per commit |
| Protocol Coverage | 0% | 100% | ADR acceptance |
| Plugin Samples | 0 | 5 | Phase 3 end |
| API Breaking Changes | N/A | 0 | Per minor release |

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| WASM sandbox complexity | High | Medium | Prototype early, use Wasmtime |
| API versioning conflicts | Medium | High | Start with conservative stability |
| Multi-instance storage | Medium | High | Use established backends (S3) |
| GitHub Actions billing | High | Low | Use local runners where possible |

---

## Open Questions

1. **Plugin Repository**: Do we need an official plugin marketplace?
2. **LTS Releases**: Should we offer long-term support releases?
3. **Enterprise SSO**: Is OIDC/SAML integration in scope for Phase 5?
4. **Binary Protocol**: Should we add MessagePack option for high-throughput?

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-05-05 | Claude | Initial roadmap creation |
