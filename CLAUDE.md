# Claude AI Agent Guide - heliosCLI

heliosCLI is a multi-runtime AI coding CLI (Codex, Claude, Gemini, Cursor, Copilot) built with a Bazel monorepo, Rust core (`codex-rs`), and TypeScript CLI (`codex-cli`). It integrates with `thegent` for agent orchestration.

**Authority and Scope**

- This file is the canonical contract for all agent behavior in this repository.
- Act autonomously; only pause when blocked by missing secrets, external access, or truly destructive actions.

---

## Quick Start

```bash
# Build everything
bazel build //...

# Run Rust tests
cargo test --workspace

# Run TypeScript CLI tests
pnpm --filter codex-cli test

# Run a specific Bazel target
bazel run //codex-cli:codex -- --help
```

---

## 1. Core Expectations for Agents

### Autonomous Operation

**Proceed without asking:**

- Implementation details and technical approach
- Adding new CLI flags, commands, or agent integrations
- Refactoring and optimization within existing patterns
- Bug fixes and test improvements
- Documentation updates

**Only ask when blocked by:**

- Missing API keys or secrets
- External service access permissions
- Genuine product ambiguity
- Destructive operations (production data, forced pushes)

### Optionality and Failure Behavior

- **Fail clearly, not silently.** Use explicit failures—not silent degradation or logging-only warnings.
- **Force requirements where they belong.** If a service or config is required for correctness, fail when it is missing.
- **Graceful only via:** retries with visible feedback; error messages listing each failing item; actionable, non-obscure stack traces.

---

## 2. Repository Structure

```
heliosCLI/
├── codex-rs/           # Rust core (exec engine, protocol, sandbox)
│   ├── core/           # Core types, models, config
│   ├── exec/           # Execution engine
│   └── ...
├── codex-cli/          # TypeScript CLI (user-facing commands)
├── helios-rs/          # Helios-specific Rust extensions
├── sdk/                # SDKs for agent integration
├── scripts/            # Dev and CI scripts
├── docs/               # Documentation
├── BUILD.bazel         # Root Bazel build
├── MODULE.bazel        # Bazel module deps
├── Cargo.toml          # Rust workspace
└── pnpm-workspace.yaml # Node workspace
```

---

## 3. Build System (Bazel)

heliosCLI uses Bazel as the primary build system with Cargo and pnpm as secondary.

```bash
# Build all targets
bazel build //...

# Test all targets
bazel test //...

# Build specific target
bazel build //codex-rs/core:core

# Run specific binary
bazel run //codex-cli:codex

# Query targets
bazel query //...
```

### Bazel Rules

- Rust targets use `rules_rust`
- Node targets use `rules_nodejs` / `aspect_rules_js`
- Do not add raw `build.rs` files that bypass Bazel; use `build_script` rules
- Keep `BUILD.bazel` files in sync when adding new source files

---

## 4. Rust (codex-rs)

### Key Patterns

```rust
// Error handling: use anyhow for application code
use anyhow::{Context, Result};

fn example() -> Result<()> {
    let val = operation().context("failed to run operation")?;
    Ok(())
}

// Async: tokio runtime
#[tokio::main]
async fn main() -> anyhow::Result<()> { ... }
```

### Running Rust Checks

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

---

## 5. TypeScript CLI (codex-cli)

### Key Patterns

```typescript
// Commands use a command registry pattern
// Add new commands in codex-cli/src/commands/

// Error handling: throw with descriptive messages, never swallow
throw new Error(`Failed to connect to agent: ${err.message}`);
```

### Running Node Checks

```bash
pnpm --filter codex-cli build
pnpm --filter codex-cli test
pnpm --filter codex-cli lint
```

---

## 6. CI / Workflows

Key workflows in `.github/workflows/`:

| Workflow          | Purpose                                  |
| ----------------- | ---------------------------------------- |
| `policy-gate.yml` | PR policy enforcement (composite action) |
| `rust-ci.yml`     | Rust lint, test, build                   |
| `bazel.yml`       | Bazel build and test                     |
| `stage-gates.yml` | Stage-based release gates                |
| `ci.yml`          | Main CI pipeline                         |

**Do not inline policy logic in workflows.** Use `KooshaPari/phenotypeActions/actions/policy-gate@main`.

---

## 7. Documentation Organization

Follow `AGENTS.md` for file placement:

| Pattern                                     | Location                   |
| ------------------------------------------- | -------------------------- |
| `*QUICK_START*.md`                          | `docs/guides/quick-start/` |
| `*GUIDE*.md`                                | `docs/guides/`             |
| `*SUMMARY*.md`, `*REPORT*.md`, `PHASE_*.md` | `docs/reports/`            |
| `*INDEX*.md`, `*RESEARCH*.md`               | `docs/research/`           |
| `*CHECKLIST*.md`                            | `docs/checklists/`         |
| `*QUICK_REFERENCE*.md`                      | `docs/reference/`          |

Root-level markdown: only `README.md`, `CHANGELOG.md`, `AGENTS.md`, `CLAUDE.md`.

---

## 8. Worktree Discipline

- Feature work goes in `repos/worktrees/heliosCLI/<topic>/`
- Legacy `heliosCLI-wtrees/` and `PROJECT-wtrees/` roots are migration-only and must not receive new work
- Canonical `heliosCLI/` stays on `main`
- Never commit feature branches directly to canonical `main`

---

## Quick Reference

| Command                        | Purpose                 |
| ------------------------------ | ----------------------- |
| `bazel build //...`            | Build all Bazel targets |
| `bazel test //...`             | Test all Bazel targets  |
| `cargo test --workspace`       | Run all Rust tests      |
| `cargo clippy --workspace`     | Rust linting            |
| `pnpm --filter codex-cli test` | TypeScript CLI tests    |

## CI Completeness Policy

- Always evaluate and fix ALL CI check failures on a PR, including pre-existing failures inherited from main.
- Never dismiss a CI failure as "pre-existing" or "unrelated to our changes" — if it fails on the PR, fix it in the PR.
- This includes: build, lint, test, docs build, security scanning (CodeQL), code review gates (CodeRabbit), workflow guard checks, and any other CI jobs.
- When a failure is caused by infrastructure outside the branch (e.g., rate limits, external service outages), implement or improve automated retry/bypass mechanisms in CI workflows.
- After fixing CI failures, verify locally where possible (build, vet, tests) before pushing.

## Phenotype Git and Delivery Workflow Protocol <!-- PHENOTYPE_GIT_DELIVERY_PROTOCOL -->

- Use branch-based delivery with pull requests; do not rely on direct default-branch writes where rulesets apply.
- Prefer stacked PRs for multi-part changes so each PR is small, reviewable, and independently mergeable.
- Keep PRs linear and scoped: one concern per PR, explicit dependency order for stacks, and clear migration steps.
- Enforce CI and required checks strictly: do not merge until all required checks and policy gates are green.
- Resolve all review threads and substantive PR comments before merge; do not leave unresolved reviewer feedback.
- Follow repository coding standards and best practices (typing, tests, lint, docs, security) before requesting merge.
- Rebase or restack to keep branches current with target branch and to avoid stale/conflicting stacks.
- When a ruleset or merge policy blocks progress, surface the blocker explicitly and adapt the plan (for example: open PR path, restack, or split changes).

## Phenotype Org Cross-Project Reuse Protocol <!-- PHENOTYPE_SHARED_REUSE_PROTOCOL -->

- Treat this repository as part of the broader Phenotype organization project collection, not an isolated codebase.
- During research and implementation, actively identify code that is sharable, modularizable, splittable, or decomposable for reuse across repositories.
- When reusable logic is found, prefer extraction into existing shared modules/projects first; if none fit, propose creating a new shared module/project.
- Include a `Cross-Project Reuse Opportunities` section in plans with candidate code, target shared location, impacted repos, and migration order.
- For cross-repo moves or ownership-impacting extractions, ask the user for confirmation on destination and rollout, then bake that into the execution plan.
- Execute forward-only migrations: extract shared code, update all callers, and remove duplicated local implementations.

## Phenotype Long-Term Stability and Non-Destructive Change Protocol <!-- PHENOTYPE_LONGTERM_STABILITY_PROTOCOL -->

- Optimize for long-term platform value over short-term convenience; choose durable solutions even when implementation complexity is higher.
- Classify proposed changes as `quick_fix` or `stable_solution`; prefer `stable_solution` unless an incident response explicitly requires a temporary fix.
- Do not use deletions/reversions as the default strategy; prefer targeted edits, forward fixes, and incremental hardening.
- Prefer moving obsolete or superseded material into `.archive/` over destructive removal when retention is operationally useful.
- Prefer clean manual merges, explicit conflict resolution, and auditable history over forceful rewrites, force merges, or history-destructive workflows.
- Prefer completing unused stubs into production-quality implementations when they represent intended product direction; avoid leaving stubs ignored indefinitely.
- Do not merge any PR while any check is failing, including non-required checks, unless the user gives explicit exception approval.
- When proposing a quick fix, include a scheduled follow-up path to a stable solution in the same plan.

## Worktree Discipline

- Feature work goes in `.worktrees/<topic>/`
- Legacy `PROJECT-wtrees/` and `repo-wtrees/` roots are for migration only and must not receive new work.
- Canonical repository remains on `main` for final integration and verification.
