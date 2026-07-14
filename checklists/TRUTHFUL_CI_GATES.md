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
