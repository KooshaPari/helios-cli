# Contributing

## Overview

This is a fork of the GitHub Codex CLI. For upstream contributions, see [github.com/github/CursorHb3](https://github.com/github/CursorHb3).

## Development Setup

### Prerequisites

- Node.js >= 22
- pnpm >= 10.29.3
- Rust (for `codex-rs` components)
- Python 3.x (for `sdk/python/`)

### Installation

```bash
# Install dependencies
pnpm install

# Build all packages
pnpm build
```

## Testing

```bash
# Run all tests (via pnpm filter)
pnpm -r test

# Run tests for a specific package
pnpm --filter codex-cli test
```

## Code Style

```bash
# Lint
pnpm lint

# Format check
pnpm format

# Auto-fix formatting
pnpm format:fix
```

## Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `pnpm format:fix` and `pnpm lint` to ensure code quality
5. Run tests
6. Submit a pull request
