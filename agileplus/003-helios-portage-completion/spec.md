# AgilePlus Feature 003: CI Green and Portage Evaluation Trace

This repository participates in Portage feature `003-helios-portage-completion`.

## Requirements

- **HCLI-FR-003-001:** CI policy changes MUST reference this feature and work package WP02.
- **HCLI-FR-003-002:** Third-party GitHub Actions in WP02-traced workflows MUST be pinned to full commit SHAs.
- **HCLI-FR-003-003:** PRs MUST not contain diagnostic logs, API dumps, or user-specific push scripts.
- **HCLI-FR-003-004:** Required Rust build/test/security checks, codespell, and Sonar security rating MUST pass before merge.
- **HCLI-FR-003-005:** Automation scripts MUST resolve the repository root dynamically and accurately describe whether they mutate git/GitHub state.

## Work package WP02

1. Remove diagnostic/ad-hoc artifacts from PR #604.
2. Pin cargo-deny action to a full SHA.
3. Add trace comments to changed CI policy files.
4. Fix or remove `scripts/push_ci_green_pr.bat`.
5. Rerun and archive CI evidence.

## Requirements-to-test matrix

All 5 requirements are covered by executable checks in
`tests/test_ci_traceability.py` (5/5 = 100% traceability coverage).

| Requirement | Automated test | Runtime evidence |
| --- | --- | --- |
| HCLI-FR-003-001 | `test_hcli_fr_003_001_ci_policy_has_feature_trace` | `python tests/test_ci_traceability.py` |
| HCLI-FR-003-002 | `test_hcli_fr_003_002_external_actions_are_sha_pinned` | `python tests/test_ci_traceability.py` |
| HCLI-FR-003-003 | `test_hcli_fr_003_003_diagnostic_artifacts_are_absent` | `python tests/test_ci_traceability.py` |
| HCLI-FR-003-004 | `test_hcli_fr_003_004_required_gate_configuration_is_strict` | PR checks: Rust CI, cargo-deny, codespell, and SonarCloud |
| HCLI-FR-003-005 | `test_hcli_fr_003_005_user_specific_push_scripts_are_absent` | `python tests/test_ci_traceability.py` |

## Acceptance

PR #604 is mergeable with required checks green, Sonar issue S7637 closed, codespell green, and review findings resolved or explicitly dispositioned.

Canonical cross-repo artifacts: `portage/agileplus/003-helios-portage-completion/`.
