# Codex snapshot reconstruction

Status: proper red; do not regenerate or accept dependency lockfiles yet.

## Provenance evidence

- [x] The damaged import is commit `4723d2c4c9cb48d6e10b5386cd7d6a9a0ad24555`
  (`merge: upstream/main into fork main (superset)`), committed at
  `2026-06-26T09:56:30Z`.
- [x] Its sole parent is `eccd9de1034e17575dcbe7ee71fa998ba111dd4c`; this was not a real Git merge,
  so Git cannot reconstruct an upstream parent automatically.
- [x] The import changed 4,193 files (1,312,430 insertions and 1,674 deletions).
- [x] `git blame codex-rs/Cargo.lock` attributes the checked-in conflict markers directly to
  `4723d2c4c`; they are committed corruption, not a local merge in progress.
- [x] The last OpenAI Codex commit returned before the import timestamp is
  `6d2168f06ae275d5e1f73cabf935d2bcc8549998` at `2026-06-26T08:27:41Z`.
- [ ] Prove that commit is the imported source by comparing Git blob IDs for representative root,
  Rust, workflow, and lock files. Network fetch and GitHub tree requests timed out on 2026-07-14,
  so the commit remains the leading candidate rather than asserted provenance.

## Current deterministic failures

- [x] `codex-rs/Cargo.lock` contains unresolved `<<<<<<< HEAD`, `=======`, and
  `>>>>>>> upstream/main` blocks.
- [x] `codex-rs/Cargo.toml` lists workspace members whose directories are absent:
  `debug-client`, `cloud-requirements`, and `test-macros`.
- [x] The interrupted local rewrite of `tools/argument-comment-lint/Cargo.lock` was reverted; it
  was a broad registry refresh unrelated to snapshot reconstruction.
- [ ] Determine whether each missing member belongs to the timestamped upstream snapshot or is a
  later manifest-only fork change.
- [ ] Audit all workspace path dependencies and member manifests after restoring those sources.
- [ ] Regenerate `codex-rs/Cargo.lock` only from the restored, coherent source/manifests.
- [ ] Reconcile `pnpm-lock.yaml` against the timestamped root workspace and prove a frozen install.

## Required acceptance gates

- [ ] No merge markers in `codex-rs/Cargo.lock`, `tools/argument-comment-lint/Cargo.lock`, or
  `pnpm-lock.yaml`.
- [ ] `cargo metadata --locked --manifest-path codex-rs/Cargo.toml --no-deps` succeeds.
- [ ] `cargo metadata --locked --manifest-path tools/argument-comment-lint/Cargo.toml --no-deps`
  succeeds.
- [ ] `pnpm install --frozen-lockfile` succeeds at the repository root.
- [ ] The repository CI-contract tests pass.
- [ ] Rust formatting and the focused jobs selected by `.github/workflows/rust-ci.yml` pass.
- [ ] Any remaining deterministic failure is documented as proper red with its owning path and
  reproduction command; no required gate is weakened or skipped.

## Reconstruction rule

Restore files only from the proven timestamped snapshot plus intentional fork commits after
`4723d2c4c`. Do not copy current upstream wholesale, select one side of every lock conflict, or
generate locks while workspace sources are missing.
