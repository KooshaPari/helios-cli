# helioscope → helios-cli absorption assessment

**Status:** Blocked (do not subtree-merge full repo)  
**Plan reference:** `phenotype-registry/RATIONALIZATION_PLAN.md` — helioscope retires to helios-cli (canonical codex fork)  
**Date:** 2026-05-31

## Verdict

| Criterion | Result |
|-----------|--------|
| Same upstream (openai/codex) | Yes — both are codex-monorepo forks |
| Canonical absorber | **helios-cli** (`main`) — active Phenotype fork with CVE/workspace fixes |
| Retire candidate | **helioscope** (`master`) — legacy fork; README still references `heliosCLI` |
| `git merge-base` | **None** — unrelated histories after divergent rebases |
| Unique commits on helioscope | ~283 (sample: harness version bumps, deny.toml, fleet docs) |
| Unique commits on helios-cli | ~4169 ahead of helioscope tip |
| Full-tree `git subtree add --squash` | **Mechanically succeeds** but adds **~5,134 files** (~790k LOC) — duplicate codex monorepo, not consolidation |

## Why full subtree merge is rejected

1. **No merge-base** — cannot do a normal merge; only squash-subtree of entire tree.
2. **Duplicate payload** — subtree places a second full codex workspace under `vendor/` with no deduplication against existing `codex-rs/`.
3. **Plan intent** — Step 2 in `RATIONALIZATION_PLAN.md` lists helioscope retirement as **`gh repo archive` + husk README**, not code merge.
4. **helios-cli is strictly ahead** — Phenotype-specific security and workspace fixes (#522–#527) live on helios-cli only.

## Recommended path (reversible, PR-only)

1. **Archive** `KooshaPari/helioscope` with README redirect → `KooshaPari/helios-cli` (separate ops PR; not done in code-absorb PR).
2. **Cherry-pick audit** — if any of the ~283 helioscope-only commits contain unique fixes not on helios-cli, open targeted cherry-pick PRs (do not bulk subtree).
3. **Optional:** add `helioscope` to `phenotype-registry` redirect table when archive lands.

## Build verification (absorber)

Run on `helios-cli` `main` (or this branch) with target dir on `E:`:

```powershell
$env:CARGO_TARGET_DIR = 'E:\cargo-target\helios-cli'
cargo check --workspace --manifest-path codex-rs/Cargo.toml
```

CI on this repo (`rust-ci.yml`) is the authoritative green gate before any archive.

**Local check (2026-05-31, Windows):** `cargo check --workspace` in `codex-rs/` failed on pre-existing `codex-windows-sandbox` parse error (`setup_orchestrator.rs:838` unclosed delimiter) — not introduced by this assessment doc. Linux CI remains the merge gate.

## helioscope-only commit sample (for cherry-pick triage)

```
32e097e docs(fleet): branch protection audit report
028627f fix(pheno-cli): add missing README; fix(guardis): correct repo URL
0e0ae3d chore(helioscope): update harness crate versions
74be1c4 fix(helioscope): remove broken symlink CONSTITUTION.yaml
```
