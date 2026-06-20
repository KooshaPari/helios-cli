# ── Tier-0 justfile — HeliosCLI ────────────────────────────────────────────
# Canonical recipes mirrored in CI. Keep this file shell-portable (bash).
#
# Quick reference:
#   just build         release build
#   just test          run the test suite
#   just lint          clippy + fmt check
#   just fmt           apply formatter
#   just audit         cargo-audit (RustSec)
#   just deny          cargo-deny (licenses, bans, sources, advisories)
#   just grade         fleet-wide grading gate
#   just ci            full local CI sweep
#   just codex-*       codex-rs subdir recipes (alias of subdir's justfile)
#
# All recipes operate on the top-level `crates/` workspace by default. The
# vendored `codex-rs/` and `helios-rs/` workspaces are excluded from the
# top-level `Cargo.toml` and have their own roots. To build them use the
# dedicated `codex-*` recipes or `cd codex-rs && just`.

set shell := ["bash", "-uc"]
set positional-arguments

# Display help
help:
    @just --list

# ── Phenotype-org standard recipes (canonical across all repos) ──────────

# Build the primary harness workspace in release mode
build:
    cargo build --release --workspace

# Run the test suite (cargo nextest if available, otherwise cargo test)
test:
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        cargo nextest run --workspace --no-fail-fast; \
    else \
        cargo test --workspace --all-features; \
    fi

# Lint: clippy with warnings as errors, plus fmt --check
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo fmt --all -- --check

# Apply the formatter in place
fmt:
    cargo fmt --all

# Security advisories (cargo-audit) against the RustSec database
audit:
    @if command -v cargo-audit >/dev/null 2>&1; then \
        cargo audit --deny warnings; \
    else \
        echo "cargo-audit not installed; install with: cargo install cargo-audit --locked"; \
        exit 1; \
    fi

# License + advisory + ban + source checks (cargo-deny)
deny:
    @if command -v cargo-deny >/dev/null 2>&1; then \
        cargo deny check; \
    else \
        echo "cargo-deny not installed; install with: cargo install cargo-deny --locked"; \
        exit 1; \
    fi

# Fleet-wide grading gate (uses vendored or central grade.sh)
grade:
    @if [ -f grade.sh ]; then ./grade.sh; \
    elif [ -f ../grade.sh ]; then bash ../grade.sh; \
    else \
        echo "no grade.sh found (vendored or central); running local ci instead"; \
        just ci; \
    fi

# Fast grading gate (skip long-running jobs)
grade-fast:
    @if [ -f grade.sh ]; then ./grade.sh --fast; \
    elif [ -f ../grade.sh ]; then bash ../grade.sh --fast; \
    else \
        echo "no grade.sh found; running local lint + test instead"; \
        just lint && just test; \
    fi

# Full local CI sweep (mirrors .github/workflows/ci.yml)
ci: lint test build audit deny
    @echo "✓ CI checks pass"

# ── Hygiene: validate the tier-0 file set ─────────────────────────────────

# Check that all tier-0 hygiene files exist
hygiene-check:
    @for f in .github/CODEOWNERS \
             .github/ISSUE_TEMPLATE/bug_report.md \
             .github/ISSUE_TEMPLATE/feature_request.md \
             .github/ISSUE_TEMPLATE/config.yml \
             .github/PULL_REQUEST_TEMPLATE.md \
             CODE_OF_CONDUCT.md CONTRIBUTING.md SECURITY.md CHANGELOG.md \
             justfile deny.toml .editorconfig .gitattributes \
             .github/dependabot.yml; do \
        if [ ! -f "$f" ]; then echo "MISSING: $f"; MISSING=1; \
        else echo "ok: $f"; fi; \
    done; \
    [ -z "${MISSING:-}" ] || (echo "tier-0 hygiene: incomplete" && exit 1)

# Verify that all Rust source files carry an SPDX-License-Identifier
hygiene-spdx:
    @MISSING=0; \
    for f in $(find crates -name '*.rs' -not -name '*.pb.rs' -not -path '*/target/*'); do \
        if ! head -3 "$f" | grep -q "SPDX-License-Identifier"; then \
            echo "no SPDX: $f"; MISSING=1; \
        fi; \
    done; \
    if [ "$MISSING" -eq 1 ]; then \
        echo "Some Rust files are missing an SPDX-License-Identifier header."; \
        exit 1; \
    fi; \
    echo "✓ all Rust files have SPDX-License-Identifier"

# Verify all third-party GitHub Actions are SHA-pinned
hygiene-actions-pinned:
    @UNPINNED=0; \
    for f in $(find .github/workflows -name '*.yml' -o -name '*.yaml'); do \
        if grep -E '^\s*uses:\s+[^@]+@(v[0-9]+|main|master|stable|nightly|beta)' "$f" | grep -vE '^\s*#' > /dev/null; then \
            echo "unpinned action in $f:"; \
            grep -nE '^\s*uses:\s+[^@]+@(v[0-9]+|main|master|stable|nightly|beta)' "$f" | grep -vE '^\s*#'; \
            UNPINNED=1; \
        fi; \
    done; \
    if [ "$UNPINNED" -eq 1 ]; then \
        echo "Some workflow steps use floating refs; pin to a SHA."; \
        exit 1; \
    fi; \
    echo "✓ all workflow actions are SHA-pinned"

# ── Workspace shortcuts ──────────────────────────────────────────────────

# Build everything (all workspaces) — slower; for release prep only
build-all:
    cargo build --release --workspace
    cd codex-rs && cargo build --release --workspace
    cd helios-rs && cargo build --release

# Print effective Cargo metadata (workspaces, lock, rustc)
info:
    @echo "rustc: $(rustc --version)"
    @echo "cargo: $(cargo --version)"
    @echo "just:  $(just --version 2>/dev/null || echo 'n/a')"
    @cargo metadata --no-deps --format-version 1 | head -c 200 && echo

# Pre-flight install: rustup show + cargo fetch
install:
    rustup show active-toolchain
    cargo fetch

# Apply clippy auto-fixes
fix *args:
    cargo clippy --fix --workspace --all-targets --allow-dirty "$@"

# ── codex-rs wrappers (forwarded to the vendored subdir's justfile) ─────

alias c := codex
codex *args:
    cd codex-rs && just codex "$@"

# `codex exec`
exec *args:
    cd codex-rs && just exec "$@"

# `codex file-search` (the CLI version of the file-search crate)
file-search *args:
    cd codex-rs && cargo run --bin codex-file-search -- "$@"

# Run the MCP server
mcp-server-run *args:
    cd codex-rs && cargo run -p codex-mcp-server -- "$@"

# Tail logs from the state SQLite database
log *args:
    @if [ "${1:-}" = "--" ]; then shift; fi
    cd codex-rs && cargo run -p codex-state --bin logs_client -- "$@"

# DevOps / delivery helpers
devops-status:
    git status --short --branch
    git remote -v
    git log --oneline -n 5
