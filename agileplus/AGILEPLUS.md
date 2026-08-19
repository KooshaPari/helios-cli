# AgilePlus Master Configuration

`helios-cli` participates in **AgilePlus** — a multi-repo Agile program management framework that ties delivery cadence, quality gates, and pillar-based scorecards to a single source of truth.

## Repository identity

| Field | Value |
| --- | --- |
| Repo slug | `helios-cli` |
| Program slug | `helios` |
| Workspace kind | Rust workspace (`crates/*`, `sdk/{python,typescript}`, `harness/*`) |
| Tier | T1 (release-bearing) |
| Default branch | `main` |
| Audit date (this snapshot) | 2026-08-19 |
| Current grade | **7.5 / 10** (31-pillar average 6.42) |

## Sprint cadence

- **Cadence:** 1-week sprints (Tue → following Tue).
- **Sprint #34:** 2026-08-19 → 2026-08-26.
- **Sprint planning:** Tuesday 09:00 PT, in `agileplus/sprint-current.md`.
- **Retro / demo:** Final day of sprint, 14:00 PT.
- **Backlog refinement:** Friday 14:00 PT.
- **No-sprint windows:** Last week of quarter is reserved for stabilization + pillar audit refresh.

## Work items

- Feature specs live at `agileplus/<NNN>-<short-name>/spec.md`.
- Each spec references a Portage `HCLI-FR-NNN-MMM` requirement set.
- Stories and tasks live under `docs/user-stories/` and `docs/changes/`.
- Spec ↔ test traceability is enforced by `tests/test_ci_traceability.py` and similar in the spec.

## Quality gates

Defined in `agileplus/quality-gates.yml`. Summary:

| Gate | Scope | Required to merge? |
| --- | --- | --- |
| `lint` | rustfmt, clippy, actionlint, codespell, yamllint | yes |
| `test` | `cargo test --workspace`, sdk pytest, sdk jest | yes |
| `coverage` | ≥ 70% patch coverage (gate fail-closed at < 60%) | yes |
| `security` | cargo-deny (advisories, bans, licenses, sources) + sonar security rating A | yes |
| `license` | cargo-deny allowlist + REUSE compliance | yes |
| `docs` | doc build + broken-link scan | yes |
| `i18n` | extraction parity check (advisory until i18n pillar ≥ 6) | advisory |

## Pillar scorecard

31-pillar scorecard lives at `agileplus/pillars/31-pillar-scorecard.json`. Snapshot:

- Code Quality 8 · Tests 6 · Docs 8 · CI/CD 8 · Security 8 · Architecture 8 · Performance 6 · DX 7 · Releases 8 · Monitoring 6 · Deps 7 · Reviews 7 · Branch Mgmt 4 · Issue Tracking 7 · Agile PM 5 · Accessibility 4 · i18n 2 · Mobile 3 · API 7 · DB 5 · Errors 7 · Logging 6 · Config 7 · Env 6 · Build 7 · Pkg 7 · License 8 · Community 6 · Contributing 7 · CoC 8 · Vuln Disc 8
- **Average:** 6.42 · **Grade:** 7.5

Pillars under remediation: Branch Mgmt (4), Accessibility (4), i18n (2), Mobile (3), DB (5), Agile PM (5), Monitoring (6), Performance (6), Logging (6), Env (6).

## Governance

- Pillar owners: see `agileplus/CODEOWNERS-pillars`.
- Weekly pillar scorecard issue: posted by `.github/workflows/agileplus-pillar-scorecard.yml` every Monday 08:00 PT.
- Velocity: tracked in `agileplus/velocity.md`.
- Retros: `agileplus/sprint-retrospective-template.md` filled per sprint and archived under `agileplus/archive/sprint-NN-retro.md`.

## Cross-repo pointers

- Portage canonical artifacts: `portage/agileplus/003-helios-portage-completion/`.
- Phenotype hub scorecard: shared by `phenotype-infra`; we consume the public JSON export.
- Quota and infrastructure audits: `docs/plans/` and `docs/reports/`.