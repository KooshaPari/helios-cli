# AgilePlus in `helios-cli`

> One-page explainer. If you're new to the repo, read this first.

## What is AgilePlus here?

AgilePlus is the delivery + quality framework this repo runs on. It bundles four things into one set of files under `agileplus/`:

| File | What it does |
| --- | --- |
| `agileplus/AGILEPLUS.md` | Master config: cadence, gate summary, pillar snapshot, cross-repo pointers. |
| `agileplus/sprint-current.md` | The sprint we're in right now (goal, owner, backlog, status). |
| `agileplus/sprint-retrospective-template.md` | Copy-paste retro skeleton; archived per sprint. |
| `agileplus/backlog.md` | Top 10 prioritized items, P0–P3, with owner and pillar impact. |
| `agileplus/pillars/31-pillar-scorecard.json` | 31 dimensions we audit weekly; drives the grade. |
| `agileplus/quality-gates.yml` | Lint, test, coverage, security, license, docs gates in one file. |
| `agileplus/velocity.md` | Last 5 sprints' committed vs. completed points + CI red time. |
| `agileplus/CODEOWNERS-pillars` | Pillar-based ownership mapping. |

A weekly GitHub Actions job (`.github/workflows/agileplus-pillar-scorecard.yml`) reads the scorecard and posts a Markdown table to a tracked issue titled `Weekly Pillar Scorecard YYYY-MM-DD`.

## Cadence

- **Sprint length:** 1 week (Tue → Tue).
- **Sprint #34:** 2026-08-19 → 2026-08-26.
- **Planning:** Tuesday 09:00 PT.
- **Retro:** Final day, 14:00 PT.
- **Refinement:** Friday 14:00 PT.

## How a feature flows

```
idea → user story (docs/user-stories/) → spec (agileplus/NNN-name/spec.md)
     → work item in backlog.md → PR(s) → pillar delta on next weekly issue
```

Each spec follows the Helios pattern (see `agileplus/003-helios-portage-completion/spec.md`):

1. Requirements with `HCLI-FR-NNN-MMM` IDs.
2. Work packages (WP01, WP02, …).
3. Requirements-to-test matrix pointing at `tests/test_*.py`.
4. Acceptance criteria.

## Quality gates

`agileplus/quality-gates.yml` is the source of truth for what blocks a PR. Blocking gates: **lint, test, coverage, security, license, docs**. The `i18n` gate is advisory until the i18n pillar reaches 6.

To run them locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features --locked
cargo deny --workspace check advisories bans licenses sources
pnpm --filter docs build
```

## Pillars

We track 31 pillars grouped into tiers:

- **Tier 1** (weight ≥ 1.5, blocks releases): CI/CD, Security, Tests, Branch Mgmt, Vuln Disc, Releases.
- **Tier 2** (weight 1.0, core quality): Docs, Code Quality, Architecture, DX, Deps, Reviews, …
- **Tier 3** (weight 0.5, long tail): i18n, Accessibility, Mobile, Logging, Performance, …

Current snapshot (audit 2026-08-19): average **6.42**, grade **7.5**. Remediation targets are in the scorecard JSON under `remediation`.

## Ownership

Two layers:

1. **Path-based** — `.github/CODEOWNERS` (existing).
2. **Pillar-based** — `agileplus/CODEOWNERS-pillars` (this setup). Pillar owners are accountable for keeping the scorecard honest and filing remediation work when a pillar drops.

## Cross-repo

- We export `agileplus/pillars/31-pillar-scorecard.json` to the Portage hub weekly.
- `phenotype-infra` consumes the public aggregate to draw the cross-repo matrix.
- Vulnerability disclosures go through `.github/SECURITY.md` and feed the Vuln Disc pillar.

## Where to start

1. Read `agileplus/sprint-current.md` for "what's hot".
2. Read `agileplus/backlog.md` and pick an unowned P0/P1 if you have capacity.
3. Open a PR with a spec under `agileplus/NNN-name/spec.md` for any feature that touches Tier 1 pillars.