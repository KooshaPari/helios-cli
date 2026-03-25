# Helios Chain Merge Ledger (2026-03-03)

## Scope
- Repos checked: `KooshaPari/helios-cli`, `kooshapari/heliosCLI`
- Target chain focus: PR #347 and linked replay/superseded lanes

## Task Outcomes

### 1) PR #347 rust-ci targeted fix
- Root rust-ci failure identified: duplicate `large_stack_test` import in `codex-rs/core/tests/suite/compact_resume_fork.rs` (`E0252`).
- Fix commit: `cc9fd747a5` on branch `codex/pr347-rustci-import-fix-20260303`.
- Delivery path (policy-compliant): opened PR #358 into `codex/absorb-prep-gh013-20260303` and merged.
- PR #358: https://github.com/KooshaPari/helios-cli/pull/358 (MERGED)

### 2) Merge #347 if policy allows
- Current status: NOT MERGED (OPEN)
- PR #347: https://github.com/KooshaPari/helios-cli/pull/347
- Hard blocker: `policy-gate` fails on merge commits in diff range from `main` to PR head.
  - failing run: https://github.com/KooshaPari/helios-cli/actions/runs/22630868191/job/65580261614
  - reported merge SHAs: `b3fd14f74d829370b29d5ef0c89736478cbfd78c`, `17506601cc5ddce7678316ced0258ba5421d63eb`
- Note: rust-ci `Format / etc` is now passing on the rerun; policy gate remains a deterministic gate.

### 3) PR chain bookkeeping verification
- Verified superseded closures include explicit replacement-path comments:
  - #348 (CLOSED) -> references #350 and #356
  - #349 (CLOSED) -> references #350 and #356
  - #351 (CLOSED) -> superseded by #356
  - #353 (CLOSED) -> references #350 and #356
- Recent chain PR states:
  - #350 MERGED
  - #352 MERGED
  - #354 MERGED
  - #355 MERGED
  - #356 MERGED
  - #357 MERGED
  - #358 MERGED
  - #347 OPEN (blocked)

### 4) heliosCLI repo check
- No PR #347 exists in `kooshapari/heliosCLI`.
- Chain execution for this request was on `KooshaPari/helios-cli`.

## Recommended Next Routing (for #347)
- Rebuild PR #347 as a clean, non-merge-commit branch (cherry-pick/replay onto `main`) and open replacement PR.
- Close #347 as superseded once replacement branch is green and policy-compliant.
