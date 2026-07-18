# AgilePlus Feature 003: CI Green and Portage Evaluation Trace

This repository participates in Portage feature `003-helios-portage-completion`.

## Requirements

- **HCLI-FR-003-001:** CI policy changes MUST reference this feature and work package WP02.
- **HCLI-FR-003-002:** Third-party GitHub Actions MUST be pinned to full commit SHAs.
- **HCLI-FR-003-003:** PRs MUST not contain diagnostic logs, API dumps, or user-specific push scripts.
- **HCLI-FR-003-004:** Required Rust build/test/security checks, codespell, and Sonar security rating MUST pass before merge.
- **HCLI-FR-003-005:** Automation scripts MUST resolve the repository root dynamically and accurately describe whether they mutate git/GitHub state.

## Work package WP02

1. Remove diagnostic/ad-hoc artifacts from PR #604.
2. Pin cargo-deny action to a full SHA.
3. Add trace comments to changed CI policy files.
4. Fix or remove `scripts/push_ci_green_pr.bat`.
5. Rerun and archive CI evidence.

## Acceptance

PR #604 is mergeable with required checks green, Sonar issue S7637 closed, codespell green, and review findings resolved or explicitly dispositioned.

Canonical cross-repo artifacts: `portage/agileplus/003-helios-portage-completion/`.
