# Sprint velocity — helios-cli

> Source of truth for cycle time and throughput. Updated at each retro.
> Each row is one finished sprint.

| Sprint | Window | Committed pts | Completed pts | Carryover | Avg PR review (h) | CI red on main (min) | Pillar Δ |
| ------ | ------ | ------------- | ------------- | --------- | ----------------- | -------------------- | -------- |
| 30 | 2026-08-05 → 2026-08-12 | 18 | 14 | 4 | 22.4 | 41 | +0.1 (CoC, License) |
| 31 | 2026-08-12 → 2026-08-19 | 20 | 19 | 1 | 18.7 | 27 | +0.3 (CI/CD, Security) |
| 32 | 2026-08-19 → 2026-08-26 | 22 | 21 | 1 | 17.1 | 19 | +0.2 (Docs, Releases) |
| 33 | 2026-08-26 → 2026-09-02 | 21 | 17 | 2 | 19.8 | 33 | +0.1 (Deps, Reviews) |
| 34 | 2026-08-19 → 2026-08-26 | 24 | — | — | — | — | target +0.4 |

> Note: Sprint 32 / 33 overlap shown intentionally during the migration to Tuesday-start cadence; effective from Sprint 35 onward the cadence stabilizes.

## Trailing averages (last 5 sprints)

- **Velocity (completed pts):** 17.25 → trend up.
- **PR review latency:** ~20h median, target < 24h.
- **CI red time on main:** trending down (41 → 19 min); branch protection (HCLI-WP-34-03) expected to drop this further.
- **Pillar Δ / sprint:** +0.18 average; 0.4 needed to hit grade 8.0 by end of Sprint 38.

## Forecasting

- Sprint 35 forecast (capacity 22 pts, backlog P1+P2 mix): 18 pts likely, 4 pts at risk (HCLI-WP-35-02 accessibility, HCLI-WP-35-01 monitoring).
- Branch Mgmt pillar projected to reach 7 within 2 sprints after HCLI-WP-34-03 lands.

## Outliers

- Sprint 33 carryover (2 items): both blockers on admin token rotation. Token rotation completed 2026-08-18; carryover absorbed into Sprint 34.