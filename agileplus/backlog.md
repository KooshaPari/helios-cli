# AgilePlus Backlog — Top 10

> Source of truth for the next two sprints. Items are scored **P0 (now)** through **P3 (next quarter)**.
> Re-rank weekly during Friday refinement.

## Priority definitions

- **P0** — must ship in current sprint; blocks release.
- **P1** — must ship this sprint or carry over with rationale.
- **P2** — schedule into next sprint.
- **P3** — backlog / roadmap; revisit each quarter.

## Top 10

| # | Priority | ID | Title | Pillar impact | Est. pts | Owner | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | **P0** | HCLI-WP-34-03 | Branch protection rollout on `main` (linear, 1 review, CI required) | Branch Mgmt 4 → 7 | 3 | @kooshapari | todo |
| 2 | **P0** | HCLI-WP-34-01 | Pin all third-party GH Actions to SHA in `trunk.yaml` | Security 8 → 9, CI/CD 8 → 9 | 2 | @kooshapari | todo |
| 3 | **P0** | HCLI-WP-34-02 | Codespell run on `docs/`, `agileplus/`, `sdk/` | Docs 8 → 9, CI/CD 8 → 9 | 2 | @kooshapari | todo |
| 4 | **P1** | HCLI-WP-34-04 | `harness/benchmarks/run.rs` unit-test coverage gate | Tests 6 → 7 | 5 | @kooshapari | todo |
| 5 | **P1** | HCLI-WP-34-05 | Linguijin extraction on `docs/` markdown + parity check | i18n 2 → 4 | 5 | @kooshapari | todo |
| 6 | **P1** | HCLI-WP-34-06 | Remove diagnostic artifacts from PR #604 follow-ups | CI/CD 8 → 9, DX 7 → 8 | 1 | @kooshapari | todo |
| 7 | **P2** | HCLI-WP-35-01 | Sentry/OTel pipeline for `harness` runtime + dashboard | Monitoring 6 → 8 | 8 | @kooshapari | todo |
| 8 | **P2** | HCLI-WP-35-02 | Accessibility audit + remediation on `docs/.vitepress` theme | Accessibility 4 → 6 | 5 | @kooshapari | todo |
| 9 | **P3** | HCLI-WP-36-01 | Mobile parity (Termux/iOS) smoke harness | Mobile 3 → 5 | 13 | @kooshapari | todo |
| 10 | **P3** | HCLI-WP-36-02 | DB layer abstraction review + migration plan off raw rusqlite | DB 5 → 7 | 8 | @kooshapari | todo |

## Refinement notes

- Items 1–3 are sprint-34 commitments.
- Items 4–6 are sprint-34 stretch; will roll into sprint 35 if not landed.
- Item 7 unblocks Observability pillar and is a hard dep for the next quarterly audit.
- Items 9–10 require cross-repo design; track in `docs/plans/` once kickoff scheduled.

## Carryover log

- HCLI-WP-33-09 — Sonar S7637 closure (carried into Sprint 34 as HCLI-WP-34-06).
- HCLI-WP-33-12 — Branch protection rollout (carried into Sprint 34 as HCLI-WP-34-03).