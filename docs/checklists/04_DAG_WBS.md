# Absorb Stack Prep - Session 20260303 Continuation

## Scope

Continue the staged absorb stack work from the prior WBS and carry out the remaining commits A-D with explicit validation checkpoints before proceeding to PR chain creation.

## Remaining Staged Commits (A–D)

- [ ] **A: tooling/config parity**
  - Canonicalize repository tooling and manifest files from `heliosCLI` as source of truth.
  - Align root manifests (`package.json`, `Cargo.toml`, lockfile policy, helper scripts) with repo conventions.

- [ ] **B: docs and SDK path normalization**
  - Normalize docs and SDK roots where lowercase mirror variants were identified.
  - Ensure paths and package entrypoints are routed through canonical naming.

- [ ] **C: core runtime subtree migration**
  - Absorb/runtime-map validated Rust/runtime paths from the canonical subtree.
  - Preserve `helios-rs` as blocked until explicit ownership decision is approved.

- [ ] **D: generated/perf artifact policy**
  - Implement quarantine/exclusion policy for `perf-results` and other generated artifacts.
  - No destructive deletes; move only into explicit retention zones.

## Validation Plan per Commit

1. **Commit A Validation**
   - `git diff --name-status main...HEAD` only touches allowlisted paths required by Commit A.
   - Confirm no files under blocklisted roots are modified.
   - Run lightweight manifest check for tooling files touched by Commit A.

2. **Commit B Validation**
   - Validate docs/SDK path normalization by sampling both `git status` and name-status for expected relocated paths.
   - Confirm canonical import/build paths resolve for SDK entry points.
   - Re-run `git diff --name-status main...HEAD` against `allowlist` before moving to Commit C.

3. **Commit C Validation**
   - Confirm runtime migration stays within canonical `codex-rs` paths.
   - Verify `helios-rs` remains on the blocklist and untouched.
   - Run targeted smoke commands for touched runtime modules.

4. **Commit D Validation**
   - Confirm blocklisted artifact roots are excluded (`rust_core`, `helios-rs`, `perf-results`, `transport`, `python`, `pnpm-*`, `justfile`, `rbe.bzl`, `defs.bzl`).
   - Confirm no new mirror-only roots are introduced by commit.
   - Validate PR artifact policy docs are present and explain retention/non-delete handling.

## Gate Conditions Before Advancing to Next Commit

- [ ] Validation checklist for current commit is complete and recorded.
- [ ] `git diff --name-status main...HEAD` does not include new paths outside `absorb-allowlist.txt` unless explicitly authorized.
- [ ] No blocklisted paths changed in the commit.
- [ ] No merge or rebase performed; commits remain on the intended absorb stack branch.
- [ ] Move to next commit only after all blockers in the plan are cleared.