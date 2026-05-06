# ADR-010: Plugin Architecture

**Status:** Proposed

**Date:** 2026-05-05

## Context

Helios-CLI must support extensible functionality for custom tools, providers, and integrations. Users need to add capabilities without modifying the core codebase. The plugin architecture must balance flexibility with security and stability.

## Decision

We adopt a plugin system with isolated WASM-based sandboxing and a manifest-based discovery mechanism.

### Plugin Discovery

```
~/.helios/plugins/
├── my-tool/
│   ├── plugin.toml       # Plugin manifest
│   ├── plugin.wasm       # Compiled WASM module
│   └── resources/        # Static assets
└── custom-provider/
    ├── plugin.toml
    └── plugin.wasm
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "my-tool"
version = "1.0.0"
description = "Custom linting tool"
authors = ["Author Name <author@example.com>"]
license = "MIT"

[plugin.capabilities]
tools = ["lint", "format"]
providers = []
workflows = []

[plugin.requirements]
helios_version = ">=2026.5.0"
sandbox = true

[plugin.security]
network_access = false
filesystem_access = "read-only:./src"
environment_variables = []
process_spawn = false

[plugin.config]
setting1 = "default"
setting2 = 42
```

### Plugin Lifecycle

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ DISCOVER │ ──▶ │  LOAD    │ ──▶ │ VALIDATE │ ──▶ │  INIT    │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
                                                              │
                                                              ▼
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  SHUTDOWN│ ◀── │  UNLOAD  │ ◀── │   IDLE   │ ◀── │  ACTIVE  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
```

1. **DISCOVER**: Scan plugin directories, parse manifests
2. **LOAD**: Instantiate WASM module, allocate resources
3. **VALIDATE**: Verify capabilities, check permissions
4. **INIT**: Execute plugin initialization function
5. **ACTIVE**: Plugin ready to serve requests
6. **IDLE**: No active requests, may hibernate
7. **UNLOAD**: Graceful shutdown, release resources
8. **SHUTDOWN**: Process exit

### WASM Sandbox Model

Each plugin runs in an isolated WASM runtime (WASI-compatible):

```rust
// Plugin host interface (imported by plugin)
interface helios-plugin {
    // Tool execution
    invoke-tool(name: string, args: json) -> result<json, error>;

    // Provider integration
    register-provider(endpoint: string) -> result<(), error>;

    // Logging
    log(level: string, message: string);

    // Configuration access
    get-config(key: string) -> option<string>;
}

// Plugin exports
interface plugin {
    // Required exports
    allocate() -> plugin-instance;
    get-manifest() -> plugin-manifest;

    // Optional exports
    on-activate();
    on-deactivate();
}
```

### Security Model

| Permission | Description | Default |
|------------|-------------|---------|
| `network_access` | Outbound HTTP/gRPC | `false` |
| `filesystem_access` | Read/write paths | `none` |
| `environment_variables` | Env var access | `[]` |
| `process_spawn` | Fork/exec | `false` |
| `tool_injection` | Modify tool registry | `false` |

### Capability Categories

1. **Tools**: Add new CLI commands or tool functions
2. **Providers**: Add new AI model providers
3. **Workflows**: Custom task pipelines
4. **Renderers**: Output format transformers

## Consequences

### Positive
- Strong isolation via WASM sandboxing
- Declarative permissions in manifest
- Versioned plugin API for compatibility
- Tool/provider extensibility without core changes

### Negative
- WASM compilation adds build complexity
- Performance overhead for plugin calls
- Plugin API versioning across releases
- Limited debugging for WASM plugins

### Open Questions

1. Should plugins support hot reload during development?
2. How do we handle plugin dependency conflicts?
3. What is the trusted plugin registry/gallery?
4. Do we need plugin signing and verification?
