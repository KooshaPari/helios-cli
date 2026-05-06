# Helios-CLI Architecture

**Document Version:** 1.0.0

**Last Updated:** 2026-05-05

## Overview

Helios-CLI is a multi-model coding agent CLI framework that provides a unified interface for integrating coding agents from OpenAI Codex, Claude, Gemini, and other AI models with a local sandboxing and execution engine.

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              HELIOS-CLI SYSTEM                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         USER INTERFACE LAYER                         │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │    │
│  │  │   codex-cli │  │   Server    │  │   HTTP API  │  │  SDK      │  │    │
│  │  │  (TypeScript│  │  (REST/gRPC)│  │  (External  │  │ (TypeScript│  │    │
│  │  │    TUI)     │  │             │  │  Consumers) │  │   Apps)   │  │    │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────┬─────┘  │    │
│  └─────────┼─────────────────┼─────────────────┼────────────────┼───────┘    │
│            │                 │                 │                │              │
│  ┌─────────▼─────────────────▼─────────────────▼────────────────▼─────────┐  │
│  │                          CORE EXECUTION LAYER                           │  │
│  │                                                                         │  │
│  │  ┌────────────────────┐    ┌────────────────────┐                      │  │
│  │  │   codex_delegate   │    │   codex_thread    │                      │  │
│  │  │   (Orchestration)  │    │   (Task Mgmt)      │                      │  │
│  │  └─────────┬──────────┘    └─────────┬──────────┘                      │  │
│  │            │                          │                                  │  │
│  │  ┌─────────▼─────────────────────────▼──────────┐                      │  │
│  │  │                    codex.rs                   │                      │  │
│  │  │            (Main Agent Orchestrator)          │                      │  │
│  │  └─────────────────────┬───────────────────────┘                      │  │
│  │                        │                                              │  │
│  │  ┌─────────────────────▼───────────────────────┐                      │  │
│  │  │                  agent/                     │                      │  │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌───────┐│                      │  │
│  │  │  │  TaskQueue  │ │  Executor   │ │ Logger││                      │  │
│  │  │  └─────────────┘ └─────────────┘ └───────┘│                      │  │
│  │  └───────────────────────────────────────────┘                      │  │
│  │                                                                       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│  ┌─────────────────────────────────▼─────────────────────────────────────┐  │
│  │                      PROVIDER ABSTRACTION LAYER                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │  │
│  │  │  connectors  │  │  anthropic  │  │   openai    │  │  gemini   │  │  │
│  │  │  (Unified    │  │   (Claude)  │  │   (Codex)   │  │ (Gemini)  │  │  │
│  │  │   Interface) │  │              │  │             │  │           │  │  │
│  │  └──────────────┘  └──────────────┘  └─────────────┘  └───────────┘  │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│  ┌─────────────────────────────────▼─────────────────────────────────────┐  │
│  │                        SECURITY & SANDBOX LAYER                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │  │
│  │  │   artifacts  │  │    network   │  │   keyring    │  │  sandbox  │  │  │
│  │  │   (Runtime   │  │   -proxy     │  │   -store     │  │  (Docker/│  │  │
│  │  │   Manager)   │  │   (MITM)     │  │  (Secrets)   │  │  Orbstack)│ │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └───────────┘  │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

                          │                                    │
                          ▼                                    ▼
              ┌───────────────────────┐           ┌───────────────────────┐
              │   AI PROVIDER APIs    │           │    EXTERNAL APIS      │
              │  ┌─────────────────┐  │           │  ┌─────────────────┐  │
              │  │ OpenAI (GPT-4) │  │           │  │   Anthropic     │  │
              │  │ Anthropic Claude│  │           │  │   (Claude)      │  │
              │  │ Google Gemini   │  │           │  │   Google        │  │
              │  │ GitHub Copilot  │  │           │  │   GitHub        │  │
              │  └─────────────────┘  │           │  └─────────────────┘  │
              └───────────────────────┘           └───────────────────────┘
```

## Component Responsibilities

### Layer 1: User Interface

| Component | Language | Responsibility |
|-----------|----------|----------------|
| `codex-cli` | TypeScript | Terminal UI, command parsing, user interaction |
| `server` | TypeScript/Rust | HTTP/gRPC server for programmatic access |
| `sdk/` | TypeScript | Client library for external applications |

### Layer 2: Core Execution

| Component | Responsibility |
|-----------|----------------|
| `codex_delegate` | Multi-agent orchestration and delegation |
| `codex_thread` | Task queue management and lifecycle |
| `codex.rs` | Main agent orchestrator, message routing |
| `agent/` | Task execution, logging, metrics |

### Layer 3: Provider Abstraction

| Component | Responsibility |
|-----------|----------------|
| `connectors/` | Unified provider interface |
| `anthropic/` | Claude API integration |
| `openai/` | GPT-4/Codex API integration |
| `gemini/` | Gemini API integration |

### Layer 4: Security & Sandboxing

| Component | Responsibility |
|-----------|----------------|
| `artifacts/` | JS runtime sandbox for code execution |
| `network-proxy/` | MITM proxy for network policy enforcement |
| `keyring-store/` | Secure credential storage |
| `sandbox/` | Container isolation (Docker/Orbstack/Podman) |

## Data Flow: Session Management

```
┌────────────────────────────────────────────────────────────────────────┐
│                        SESSION LIFECYCLE                               │
├────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. SESSION INITIALIZATION                                              │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐    │
│  │   CLI/User  │────▶│ codex_deleg │────▶│  codex_thread      │    │
│  │   Request  │     │             │     │  (Create Session)   │    │
│  └─────────────┘     └─────────────┘     └──────────┬──────────┘    │
│                                                       │                │
│                                                       ▼                │
│  2. MESSAGE STREAMING                                                  │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐    │
│  │   Agent     │────▶│  Provider   │────▶│  NDJSON Output      │    │
│  │   (Claude)  │     │  (API Call) │     │  (Streaming)       │    │
│  └─────────────┘     └─────────────┘     └──────────┬──────────┘    │
│                                                        │                │
│                                                        ▼                │
│  3. TOOL EXECUTION (Sandboxed)                                          │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐    │
│  │   Tool Call │────▶│  Sandbox    │────▶│  Execution Result   │    │
│  │   Request   │     │  (Docker)   │     │  (stdout/stderr)   │    │
│  └─────────────┘     └─────────────┘     └──────────┬──────────┘    │
│                                                        │                │
│                                                        ▼                │
│  4. SESSION BUNDLE (On Complete)                                        │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐    │
│  │   Session    │────▶│  Manifest   │────▶│  Signed Bundle      │    │
│  │   State      │     │  + Hash     │     │  (zstd compressed)  │    │
│  └─────────────┘     └─────────────┘     └─────────────────────┘    │
│                                                                         │
└────────────────────────────────────────────────────────────────────────┘
```

## Session Bundle Format

```
session_<uuid>/
├── manifest.json        # Session metadata, signing info
├── messages.jsonl       # All protocol messages (NDJSON)
├── diffs/
│   └── <hash>.patch     # Applied code changes
├── artifacts/
│   └── <sha256>         # Generated files
└── _signature           # HMAC-SHA256 verification
```

## Security Model

### Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                      TRUST BOUNDARY                          │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  USER MACHINE                                          │ │
│  │  ┌─────────────────────────────────────────────────┐   │ │
│  │  │  helios-cli process                             │   │ │
│  │  │  ┌─────────────┐  ┌─────────────────────────┐  │   │ │
│  │  │  │ User Code  │  │  API Keys (keyring)     │  │   │ │
│  │  │  │ (sandboxed)│  │  (encrypted at rest)   │  │   │ │
│  │  │  └─────┬──────┘  └─────────────────────────┘  │   │ │
│  │  │        │                                      │   │ │
│  │  │        ▼                                      │   │ │
│  │  │  ┌─────────────┐                            │   │ │
│  │  │  │ Network     │  ──▶ External APIs          │   │ │
│  │  │  │ Proxy       │     (HTTPS only)            │   │ │
│  │  │  └─────────────┘                            │   │ │
│  │  └─────────────────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Security Controls

| Layer | Control | Mechanism |
|-------|---------|-----------|
| Code Execution | Sandboxing | Docker/Orbstack containers with resource limits |
| Network | Policy Enforcement | MITM proxy with allowlist |
| Credentials | Storage | System keyring (macOS Keychain, Linux Secret Service) |
| Bundles | Integrity | HMAC-SHA256 signing |
| Plugins | Isolation | WASM sandboxing |
| Input | Validation | Schema validation for all external input |

### Permission Model

```toml
[sandbox.limits]
# Resource constraints
cpu_time_seconds = 300      # 5 minutes max
memory_mb = 512             # 512MB max
disk_mb = 1024              # 1GB max
network_egress = false      # No outbound network

[sandbox.network]
# Network policy
allowed_hosts = []          # Empty = block all
allowed_ports = []          # Service ports only
proxy_enabled = true       # Use MITM proxy
```

## Deployment Topology

### Local Development

```
┌─────────────────────────────────────────────────────────────┐
│                   LOCAL DEVELOPMENT                          │
│                                                              │
│   ┌──────────────┐                                          │
│   │  Terminal    │  ◀── User runs `helios-cli`             │
│   │  (TUI)       │                                          │
│   └──────┬───────┘                                          │
│          │                                                   │
│          │ IPC (stdio)                                        │
│          ▼                                                   │
│   ┌──────────────┐                                          │
│   │  helios-cli  │  ◀── Rust backend process                │
│   │  (codex-rs)  │                                          │
│   └──────┬───────┘                                          │
│          │                                                   │
│          │ HTTP/HTTPS                                        │
│          ▼                                                   │
│   ┌──────────────────────────────────────────────────┐      │
│   │              AI PROVIDERS                         │      │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐          │      │
│   │  │ OpenAI  │  │Anthropic│  │ Google  │          │      │
│   │  └─────────┘  └─────────┘  └─────────┘          │      │
│   └──────────────────────────────────────────────────┘      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Multi-User Server

```
┌─────────────────────────────────────────────────────────────┐
│                      SERVER DEPLOYMENT                       │
│                                                              │
│   ┌──────────────────────────────────────────────────────┐  │
│   │                    Load Balancer                      │  │
│   │               (nginx / cloud LB)                      │  │
│   └────────────────────┬───────────────────────────────┘  │
│                        │                                    │
│         ┌───────────────┼───────────────┐                  │
│         │               │               │                   │
│         ▼               ▼               ▼                   │
│   ┌───────────┐  ┌───────────┐  ┌───────────┐           │
│   │ Instance 1│  │ Instance 2│  │ Instance N│           │
│   │ helios-cli│  │ helios-cli│  │ helios-cli│           │
│   └─────┬─────┘  └─────┬─────┘  └─────┬─────┘           │
│         │               │               │                   │
│         └───────────────┼───────────────┘                   │
│                         │                                    │
│   ┌─────────────────────▼───────────────────────────────┐  │
│   │                 Shared Storage                        │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │  │
│   │  │  Sessions   │  │  Plugins    │  │   Config    │ │  │
│   │  │  (S3/GCS)   │  │  (NFS)      │  │  (etcd)    │ │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘ │  │
│   └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Containerized (Kubernetes)

```yaml
# Deployment topology
apiVersion: apps/v1
kind: Deployment
metadata:
  name: helios-cli
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: helios-cli
        image: kooshapari/helios-cli:latest
        resources:
          limits:
            cpu: "2"
            memory: 2Gi
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: api-keys
              key: anthropic
        - name: SANDBOX_RUNTIME
          value: "docker"
      - name: docker-sidecar
        image: docker:24-dind
        securityContext:
          privileged: true
```

## Protocol Versions

| Protocol | Version | Status |
|----------|---------|--------|
| NDJSON Streaming | 1.0 | Proposed |
| Session Bundle | 1.0 | Proposed |
| Plugin API | 1.0 | Proposed |
| Codex-RS Internal | 2026.5 | Active |

## Related Documents

- [ADR-001: Architecture Decisions](./adr/0001-record-architecture-decisions.md)
- [ADR-008: Streaming Output Protocol](./adr/ADR-008-streaming-output.md)
- [ADR-009: Session Bundle Format](./adr/ADR-009-session-bundle-format.md)
- [ADR-010: Plugin Architecture](./adr/ADR-010-plugin-architecture.md)
- [ADR-011: Codex-RS API](./adr/ADR-011-codex-rs-api.md)
- [PLAN.md](./PLAN.md)
