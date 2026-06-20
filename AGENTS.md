# AGENTS.md — helios-cli

**Repo:** [KooshaPari/helios-cli](https://github.com/KooshaPari/helios-cli)
**Branch this AGENTS.md is maintained on:** `main`
**Date:** 2026-06-19 (L5-104 ratatui fork verification shard)

---

## Project Overview

`helios-cli` is the **Phenotype-org multi-runtime CLI scaffold** — a fork of [openai/codex](https://github.com/openai/codex).

**Work state:** **SCAFFOLD** (per `README.md` work-state header, 10% progress, 2026-06-02). The `codex-rs/` workspace declares its member crates in `Cargo.toml` (and the dependency manifest is committed), but the source code for those workspace members is **NOT committed** in this repo. `helios-cli` is governance + CI skeleton only — it does not build.

When source is vendored from upstream `openai/codex`, the pins in `codex-rs/Cargo.toml` (including the ratatui fork below) become authoritative.

See `README.md` for the work-state header and `docs/` for project docs.

---

## SOTA exceptions

This repo has **one SOTA exception** carried in `codex-rs/Cargo.toml` `[patch.crates-io]` (lines 387-393):

### `ratatui` — `nornagon-v0.29.0-patch` git fork

**Verdict (verified 2026-06-19, L5-104):** **KEEP** the fork pin. The patch has **not** landed upstream as a `pub fn`. Drop the pin when the upstream ratatui release (next minor, likely 0.31.x) exposes `set_viewport_area` as `pub` or an equivalent public API on `Terminal` is added; revisit on every upstream ratatui release.

#### What the fork actually contains

The fork `nornagon/ratatui` branch `nornagon-v0.29.0-patch` (HEAD `9b2ad1298408c45918ee9f8241a6f95498cdbed2`) adds **2 commits** on top of upstream `ratatui v0.29.0` (`28732176e1`):

| SHA | Date | Subject |
|---|---|---|
| `bca287ddc5` | 2025-07-26 | **expose set_viewport_area** — the patch (changes `fn` → `pub fn` + 1-line docstring in `src/terminal/terminal.rs`) |
| `9b2ad12984` | 2025-08-03 | Merge PR #1: bump `unicode-width` 0.2.0 → 0.2.1 (transitive dep bump) |

> **Note:** the SOTA research doc `findings/sota-research-2026-06-19.md` § 7 / § 10 frames this as a "renderer bug fix". That framing is inaccurate. The patch is an **API visibility change** (private → public), not a rendering bug. The Codex TUI needs to call `Terminal::set_viewport_area()` directly from outside the ratatui crate, which the upstream API does not allow.

#### Upstream state (ratatui 0.30.x, verified 2026-06-19)

- **Latest stable:** `ratatui-v0.30.2` (released 2026-06-19, today). Also: `ratatui-v0.30.1` (2026-06-05), `ratatui-v0.30.0`, `ratatui-v0.29.0`.
- **The literal patch (`fn` → `pub fn`) has NOT been applied upstream.** In `ratatui-v0.30.2/ratatui-core/src/terminal/resize.rs:78`, the same method exists as `pub(crate) fn set_viewport_area(&mut self, area: Rect)` — accessible only within the `ratatui-core` crate, not from external crates like Codex.
- **Upstream's chosen solution:** expose viewport-area control through higher-level public API: `Terminal::with_options(Viewport::Inline(area))` (constructor) and `Terminal::resize(area)` (runtime). The internal `set_viewport_area` helper stays crate-internal.
- **Implication for bumping:** to drop the fork pin and move to `ratatui = "0.30.2"` will require a **Codex TUI refactor** — replace direct `terminal.set_viewport_area(area)` calls with the appropriate `Terminal::with_options(Viewport::Inline(area))` (construction-time) or `terminal.resize(area)` (runtime) calls. This is a code change, not a Cargo.toml bump.

#### Cargo.lock evidence (this branch)

```
$ grep -A 3 'name = "ratatui"' codex-rs/Cargo.lock
name = "ratatui"
version = "0.29.0"
source = "git+https://github.com/nornagon/ratatui?branch=nornagon-v0.29.0-patch#9b2ad1298408c45918ee9f8241a6f95498cdbed2"
```

The fork pin is the **active** resolved source — not a no-op override.

#### Sibling SOTA exceptions in the same `[patch.crates-io]` section (out of scope for this shard)

For audit completeness only — not addressed by the L5-104 shard. Track separately:

| Package | Pin | Status |
|---|---|---|
| `crossterm` | `nornagon/crossterm` branch `nornagon/color-query` | unverified |
| `tokio-tungstenite` | `openai-oss-forks/tokio-tungstenite` rev `132f5b39...` | unverified |
| `tungstenite` | `openai-oss-forks/tungstenite-rs` rev `9200079d...` (double-pinned via SSH + HTTPS) | unverified |

---

## Stack

- **Languages:** Rust (primary — codex-rs workspace)
- **Build system:** Cargo workspace (`codex-rs/Cargo.toml`)
- **Upstream lineage:** `openai/codex` (the Codex CLI from OpenAI) — this repo is a Phenotype-org fork

## Conventions

- **Branch naming:** `chore/<req-id>-<slug>-<date>` for chore work; `feat/<req-id>-<slug>-<date>` for features.
- **Commit messages:** Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `build:`, `ci:`) with optional scope.
- **PR labels:** `governance` for cleanup, `L<n>-#<n>` for tracking against DAG level.
- **Meta-bundle for a release-ready crate:** `AGENTS.md` + `llms.txt` + `WORKLOG.md` + `CHANGELOG.md` + `LICENSE-MIT`.
- **SOTA artifacts:** `findings/`, `plans/`, `worklogs/`, `docs/adr/<date>/`.

## Related

- `README.md` — work-state header + project overview
- `CHANGELOG.md` — release notes
- `codex-rs/Cargo.toml` — workspace manifest with the SOTA-exception `[patch.crates-io]` section (lines 387-393)
- `codex-rs/Cargo.lock` — resolved dependency graph (confirms fork is active)
- `findings/sota-research-2026-06-19.md` § 7 (ratatui) and § 10 (SOTA exception log) — source SOTA research; see also the revisit trigger note above
- `findings/2026-06-19-L5-104-ratatui-fork-verification.md` — verification log for this shard (TODO; see "Shard release" below)
