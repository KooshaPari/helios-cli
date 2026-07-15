# Truthful CI gates

Checklist semantics are evidence-based:

- `[x]` means the named test/evidence command passed for the code in this change.
- `[ ]` means unverified or failing; the reason must be recorded beside the item.
- Every item maps one requirement to its enforcing code, regression test, and reproducible evidence.

- [x] **FR-HELIOS-CI-001 — Required CI results propagate mandatory failures.**
      Code: `.github/workflows/rust-ci.yml` (`results`) and
      `.github/scripts/verify_required_ci.py`.
      Test: `.github/scripts/test_verify_required_ci.py` covers missing dependencies and
      missing success assertions.
      Evidence: `python -m unittest discover -s .github/scripts -p 'test_verify_required_ci.py'` and
      `python .github/scripts/verify_required_ci.py`.

- [x] **FR-HELIOS-CI-002 — npm staging failures and lockfile drift fail CI.**
      Code: `.github/workflows/ci.yml` and `.github/scripts/verify_required_ci.py`.
      Test: `.github/scripts/test_verify_required_ci.py` covers swallowed staging failures and
      mutable dependency installs.
      Evidence: `python -m unittest discover -s .github/scripts -p 'test_verify_required_ci.py'`
      and `python .github/scripts/verify_required_ci.py`.

- [ ] **FR-HELIOS-CI-003 — The vendored Codex workspace lockfile is parseable.**
      Code: `codex-rs/Cargo.lock`.
      Test: `.github/scripts/test_rusty_v8_bazel.py` parses the lockfile.
      Evidence: `python -m unittest discover -s .github/scripts -p 'test_*.py'` is red because
      the tracked lockfile contains unresolved merge markers; resolve/regenerate it in a dedicated
      dependency-reconciliation change.

- [x] **FR-HELIOS-CI-004 — TypeScript SDK formatting is ratcheted without hiding repository debt.**
      Code: `sdk/typescript/` plus its direct `@eslint/js` and `@jest/globals` tooling dependencies.
      Evidence: tracked SDK formatting issues fell from 4 to 0 under Oxfmt, SDK Prettier/build/lint pass,
      and the authoritative repository-wide Oxfmt count fell from 1,092 to 1,089 of 1,332 files.
      The full format gate deliberately remains red on those 1,089 files.

- [x] **FR-HELIOS-CI-005 — TypeScript SDK module URLs are normalized cross-platform.**
      Code: `sdk/typescript/src/exec.ts` canonicalizes malformed Windows `file://C:\...` locations
      before calling `createRequire` while preserving valid POSIX file URLs.
      Evidence: `tests/exec.test.ts` covers both URL forms and passes with SDK
      Prettier/build/lint green.

- [x] **FR-HELIOS-CI-006 — TypeScript SDK stream cleanup owns spawned-process termination.**
      Code: `sdk/typescript/src/exec.ts` waits for the exact spawned process to exit after sending
      termination, while failed spawns skip the exit wait because no process exists.
      Test: `tests/exec.test.ts` proves early stream closure remains pending until child exit and
      proves spawn failure cannot hang waiting for an exit event.
      Evidence: focused `tests/exec.test.ts` passes. With the pinned Rust 1.95 debug binary, all four
      real-binary abort cases pass; the non-aborted control remains a proper red at the unchanged
      five-second Jest limit when debug startup exceeds that budget under host contention.
