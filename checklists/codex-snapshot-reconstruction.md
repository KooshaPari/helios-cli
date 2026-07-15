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
- [x] Prove the timestamped source boundary. Commit
  `6d2168f06ae275d5e1f73cabf935d2bcc8549998` supplies the exact imported
  `pnpm-lock.yaml` blob (`2d4c6f34972b61e8336433ff7e6a6d2b35974df9`) and the Rust source families
  added by the pseudo-merge. The pseudo-merge retained the older fork
  `codex-rs/Cargo.toml`, committed conflict-marked Rust locks, and omitted 1,050 files that exist
  in that source tree; those omissions explain the incoherent workspace without requiring a
  newer upstream snapshot.

## Current deterministic failures

- [x] `codex-rs/Cargo.lock` contains unresolved `<<<<<<< HEAD`, `=======`, and
  `>>>>>>> upstream/main` blocks.
- [x] `codex-rs/Cargo.toml` lists workspace members whose directories are absent:
  `debug-client`, `cloud-requirements`, and `test-macros`.
- [x] The interrupted local rewrite of `tools/argument-comment-lint/Cargo.lock` was reverted; it
  was a broad registry refresh unrelated to snapshot reconstruction.
- [x] Determine whether each missing member belongs to the timestamped upstream snapshot or is a
  later manifest-only fork change. `debug-client`, `cloud-requirements`, and `test-macros` are stale
  entries in the retained fork manifest and do not exist in the timestamped source tree.
- [x] Audit workspace members and path dependencies after restoring the omitted timestamped files;
  locked Cargo metadata succeeds for the reconstructed workspace.
- [x] Restore `codex-rs/Cargo.lock` from the exact timestamped source tree. This avoids inventing a
  resolution from the incoherent retained manifest and preserves the upstream-tested lock graph.
- [ ] Prove a frozen pnpm install. The lock blob already matches the timestamped source exactly, but
  the 2026-07-14 local attempt timed out while `npm exec` tried to obtain pnpm; the installed Corepack
  shim is also broken (`Cannot find module ...corepack/dist/pnpm.js`). CI remains the authoritative
  frozen-install gate.

## Required acceptance gates

- [x] No merge markers in `codex-rs/Cargo.lock`, `tools/argument-comment-lint/Cargo.lock`, or
  `pnpm-lock.yaml`.
- [x] `cargo metadata --locked --manifest-path codex-rs/Cargo.toml --no-deps` succeeds.
- [x] `cargo metadata --locked --manifest-path tools/argument-comment-lint/Cargo.toml --no-deps`
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
