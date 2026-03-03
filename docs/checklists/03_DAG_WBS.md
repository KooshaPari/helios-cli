# Absorb Stack Prep - Manifest Diff and Patch Checklist

## Scope

Prepare an absorb stack from `heliosCLI` (source) and `helios-cli` (lowercase mirror) using deterministic git-manifest diffs.

## Evidence

- `docs/sessions/20260303-absorb-stack-prep/artifacts/heliosCLI.git-manifest.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/helios-cli.git-manifest.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/only-in-heliosCLI.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/only-in-helios-cli.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/manifest-summary.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/only-in-heliosCLI.topdirs.txt`
- `docs/sessions/20260303-absorb-stack-prep/artifacts/only-in-helios-cli.topdirs.txt`

## Current Delta Snapshot

- heliosCLI files: 3525
- helios-cli files: 5848
- only in heliosCLI: 3452
- only in helios-cli: 5775

Top skew in lowercase mirror (`only-in-helios-cli.topdirs.txt`):
1. `codex-rs` (2595)
2. `rust_core` (2567)
3. `perf-results` (478)
4. `docs` (29)
5. `sdk` (28)

## Concrete Patch Plan

1. Lock canonical source of truth per subtree
1. `codex-rs/*`: keep canonical pathing from `heliosCLI` and map any lowercase mirror variants.
1. `rust_core/*`: classify as mirror-only lane and block direct absorb until ownership decision.
1. `perf-results/*`: treat as generated artifacts; do not absorb blindly.

1. Build absorb candidate manifests
1. Generate `absorb-allowlist.txt` from `only-in-heliosCLI.txt` for canonical code paths.
1. Generate `absorb-blocklist.txt` from mirror-only/generated/high-churn paths.

1. Execute staged patch stack (planned commits)
1. Commit A: tooling/config parity (`justfile`, workspace/package manifests).
1. Commit B: docs and SDK path normalization.
1. Commit C: core runtime subtree migration after path-map validation.
1. Commit D: generated/perf artifact policy (exclude/archive only, no destructive delete).

1. Validate each staged patch
1. `git diff --name-status main...HEAD` stays inside allowlist.
1. targeted smoke checks per commit before advancing stack.
1. reject commit if it introduces new mirror-only roots.

## Checklist

- [x] Generated deterministic manifests for both repos.
- [x] Produced directional diffs and topdir skew reports.
- [ ] Author `absorb-allowlist.txt` from canonical roots.
- [ ] Author `absorb-blocklist.txt` for mirror/generated roots.
- [ ] Prepare Commit A (tooling/config parity).
- [ ] Prepare Commit B (docs/sdk normalization).
- [ ] Prepare Commit C (runtime subtree mapping).
- [ ] Prepare Commit D (artifact quarantine policy).
- [ ] Run targeted validation for each staged commit.
- [ ] Open absorb stack PR chain.

## Blockers

1. Mirror has dual runtime roots (`codex-rs` and `rust_core`) that require explicit canonicalization policy before absorb.
2. Lowercase mirror contains nested `repos/` worktree artifacts; must remain quarantined and excluded from absorb commits.
3. Generated `perf-results` footprint is large and should not be auto-migrated without explicit retention rules.
