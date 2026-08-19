# Sprint 34 — 2026-08-19 → 2026-08-26

## Theme

**Restore pillar averages ≥ 7.0 and ship Helios Portage Completion.**

## Sprint goal

Land all in-scope `agileplus/003-helios-portage-completion` work packages, raise the Tests / Branch Mgmt / i18n pillars, and keep the trunk green throughout.

## Owner

- **Scrum lead:** @kooshapari
- **Pillar owner rotation:** see `agileplus/CODEOWNERS-pillars`
- **Scrum master (acting):** @kooshapari

## Status

| Metric | Value |
| --- | --- |
| Day | 1 / 7 |
| Burndown | not started |
| Carryover from Sprint 33 | 2 items |
| Risks open | 1 (R-2026-08-19-i18n-extraction) |
| Pillar delta target | +0.4 weighted (Branch Mgmt 4 → 5, i18n 2 → 3, Tests 6 → 6.5) |

## Sprint backlog

| ID | Title | Pillar impact | Owner | Status |
| --- | --- | --- | --- | --- |
| HCLI-WP-34-01 | WP03 — Pin all third-party GH Actions to SHA in trunk workflows | Security, CI/CD | @kooshapari | todo |
| HCLI-WP-34-02 | WP04 — Codespell dictionary extend + run on all docs/ | Docs | @kooshapari | todo |
| HCLI-WP-34-03 | Branch protection: require linear history + 1 review + CI green on main | Branch Mgmt | @kooshapari | todo |
| HCLI-WP-34-04 | `harness/` test coverage gate: add missing unit tests for `benchmarks/run.rs` | Tests | @kooshapari | todo |
| HCLI-WP-34-05 | i18n: enable `Linguijin` extraction on `docs/` markdown parity check | i18n | @kooshapari | todo |
| HCLI-WP-34-06 | Remove stale diagnostic artifacts from PR #604 followups | CI/CD, DX | @kooshapari | todo |
| HCLI-WP-34-07 | Cargo feature audit: remove unused `harness-bench/llama` feature flags | Build, Pkg | @kooshapari | todo |

## Definition of done

- [ ] All P0 backlog items shipped and demoed.
- [ ] Quality gates green on `main` at sprint close (lint, test, coverage, security, license, docs).
- [ ] 31-pillar scorecard refreshed and diff posted to `Weekly Pillar Scorecard` issue.
- [ ] Retro written and filed under `agileplus/archive/sprint-34-retro.md`.
- [ ] Velocity entry appended to `agileplus/velocity.md`.

## Carryover from Sprint 33

- HCLI-WP-33-09 — Sonar S7637 closure verification (test-rail only). → re-opened as HCLI-WP-34-06.
- HCLI-WP-33-12 — Branch protection rollout (blocked on admin token rotation). → re-opened as HCLI-WP-34-03.

## Risks

- **R-2026-08-19-i18n-extraction:** Linguijin needs unicode CLDR data refresh; mitigation: ship extraction behind `docs-i18n-extra` CI job, leave core gates alone.