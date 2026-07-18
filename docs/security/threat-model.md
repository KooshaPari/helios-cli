# Threat Model (STRIDE-per-component)

> **Source audit:** [`FLEET-AUDIT-REPORT.md`](../../../docs/audits/FLEET-AUDIT-REPORT.md) — S7 (Threat model) is the #1 P0 gap (priority 42, 10 of 11 audited repos at score 0).
> **Template:** [`THREAT-MODEL-TEMPLATE.md`](../../../docs/audits/THREAT-MODEL-TEMPLATE.md) v1.0
> **Lifts audit score:** HeliosCLI S7 (Security / Threat model) from 0 to 2 (wired).
> **Date filed:** 2026-06-16. **Owner:** security. **Next review:** 2026-09-16 (quarterly cadence).

This model covers the four components of HeliosCLI that handle the bulk of its
trust boundary: the user-facing CLI binary, the on-disk auth and secret store,
the Rust supply chain (Cargo registry and direct deps), and the CI/CD + release
pipelines. The remaining components (TUI, exec, apply-patch, MCP server, harness
crates, plugin runtime) are out of scope for S7-2 and will be re-assessed when
S7-3 adds the 90-day CI gate.

## STRIDE cheat sheet

| Letter | Threat                 | Property violated | Question to ask                              |
| ------ | ---------------------- | ----------------- | -------------------------------------------- |
| **S**  | Spoofing               | Authentication    | Can an attacker impersonate a user/system?   |
| **T**  | Tampering              | Integrity         | Can an attacker modify data or code?         |
| **R**  | Repudiation            | Non-repudiation   | Can a user deny an action they took?         |
| **I**  | Information disclosure | Confidentiality   | Can an attacker read data they shouldn't?    |
| **D**  | Denial of service      | Availability      | Can an attacker make the system unavailable? |
| **E**  | Elevation of privilege | Authorization     | Can an attacker gain higher privileges?      |

For each cell, the rating is one of: **N/A** (not applicable), **low** (mitigation
optional), **med** (mitigation required), **high** (mitigation + test required).

---

## Component inventory

HeliosCLI is a Rust CLI fork of OpenAI Codex. The audited components are:

1. **HeliosCLI binary** — `codex-rs/cli/` (clap entry, subcommand dispatch).
2. **Auth and credential store** — `codex-rs/login/`, `codex-rs/secrets/`,
   on-disk `auth.json` + OS keyring under `KEYRING_SERVICE = "codex"`.
3. **Cargo supply chain** — `Cargo.lock` + `deny.toml` (license allowlist,
   `unknown-git = "deny"`, crates.io-only registry) and the
   `rustsec/audit-check@v2` weekly cargo-audit job in
   `.github/workflows/cargo-audit.yml`.
4. **CI/CD + release pipeline** — 26 workflows under `.github/workflows/`,
   including tag-driven release in `.github/workflows/rust-release.yml` and
   pull-request npm staging in `.github/workflows/ci.yml`.

Out of scope for S7-2 (deferred to S7-3 or future waves): TUI
(`codex-rs/tui/`), non-interactive exec (`codex-rs/exec/`), apply-patch
(`codex-rs/apply-patch/`), MCP server (`codex-rs/mcp-server/`), 18 harness
crates under `crates/harness_*`, plugin runtime (`crates/pheno-plugin/`,
`crates/plugin-arch/`). These are stable, sandboxed, and re-reviewed when
S7-3 adds the 90-day CI gate.

---

## Per-component threat grid

### Component: HeliosCLI binary (`codex-rs/cli/`)

| Threat                  | Rating | Specific attack vector                                                                          | Mitigation                                                                                                                                                         | Owner    | Last reviewed |
| ----------------------- | ------ | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------- | ------------- |
| **S — Spoofing**        | low    | Attacker plants a `helios` shim earlier in `$PATH`; user runs the wrong binary                  | clap's `command!` and explicit arg validation; release tarball SHA256 published in release notes                                                                   | cli-ops  | 2026-06-16    |
| **T — Tampering**       | med    | Malicious PR alters the clap arg parser or shell-exec wrapper to inject commands                | CODEOWNERS (`CODEOWNERS`); required PR review; `rust-ci.yml` runs `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` on every PR                    | cli-ops  | 2026-06-16    |
| **R — Repudiation**     | low    | User denies issuing a `helios exec` patch                                                       | `codex-rs/state/` persists a session log keyed to a `session_id`; PR descriptions carry commit SHAs                                                                | cli-ops  | 2026-06-16    |
| **I — Info disclosure** | med    | Error paths print full env or absolute paths to stderr                                          | `secrets::redact_secrets` in `codex-rs/secrets/src/lib.rs` scrubs `OPENAI_API_KEY` etc. from logs; release builds use `strip = true` (root `Cargo.toml`)           | security | 2026-06-16    |
| **D — DoS**             | low    | Attacker triggers expensive build of a huge workspace on user machine                           | `helios` is local; no remote service to flood. Largest risk is a malicious `.codex/config.toml` triggering heavy `apply-patch`                                     | n/a      | 2026-06-16    |
| **E — Elevation**       | med    | `helios exec` runs as the user but with sandbox enabled; sandbox escape yields user-level shell | Landlock+seccomp on Linux (`codex-rs/core/src/linux-sandbox*`), Seatbelt on macOS (`codex-rs/core/src/seatbelt.rs`); `fuzzing.yml` exercises the execpolicy parser | security | 2026-06-16    |

### Component: Auth and credential store (`codex-rs/login/`, `codex-rs/secrets/`)

| Threat                  | Rating | Specific attack vector                                                                                                                                            | Mitigation                                                                                                                                                          | Owner    | Last reviewed |
| ----------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------- |
| **S — Spoofing**        | med    | Phishing OAuth flow: `device_code_auth` (`codex-rs/login/src/device_code_auth.rs`) shows a one-time code; attacker convinces user to paste into a look-alike site | Display the full issuer URL in the TUI; warn when the redirect URL is non-`localhost`; rate-limit `request_device_code`                                             | security | 2026-06-16    |
| **T — Tampering**       | high   | Local attacker edits `~/.codex/auth.json` to mint their own tokens                                                                                                | `auth::save_auth` writes to a temp file and renames atomically; keyring is the source of truth for the refresh token, and `auth.json` is re-validated on every load | security | 2026-06-16    |
| **R — Repudiation**     | low    | User denies having run `helios login`                                                                                                                             | `codex-rs/state/` records the session start/end timestamps and the auth mode (`AuthMode` enum)                                                                      | security | 2026-06-16    |
| **I — Info disclosure** | high   | `auth.json` readable by other local users (`umask` default) or accidentally committed                                                                             | `CodexAuth` writes with mode `0o600`; `secrets::redact_secrets` scrubs logs; trufflehog (`trufflehog.yml`) gates every commit                                       | security | 2026-06-16    |
| **D — DoS**             | low    | Attacker exhausts the OS keyring by repeatedly calling `save_auth`                                                                                                | Keyring handles ~thousands of entries before perf degrades; not exploitable remotely                                                                                | n/a      | 2026-06-16    |
| **E — Elevation**       | med    | A non-privileged local user reads another user's `auth.json` and replays the API key                                                                              | File mode `0o600`; CI gate (`leak-detection.yml`) blocks PRs that introduce `auth.json` fixtures                                                                    | security | 2026-06-16    |

### Component: Cargo supply chain (`Cargo.lock`, `deny.toml`, `cargo-audit.yml`)

| Threat                  | Rating | Specific attack vector                                                                                               | Mitigation                                                                                                                                                                                                      | Owner        | Last reviewed |
| ----------------------- | ------ | -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | ------------- |
| **S — Spoofing**        | med    | Attacker publishes a typosquatted crate (e.g. `serde-json` vs `serde_json`) and HeliosCLI depends on it transitively | `deny.toml` pins `allow-registry = ["https://github.com/rust-lang/crates.io-index"]` and `unknown-git = "deny"`; `cargo-deny` runs in CI on every PR                                                            | supply-chain | 2026-06-16    |
| **T — Tampering**       | high   | Malicious update to a transitive dep modifies the build output                                                       | `Cargo.lock` is committed (62K lines, 200+ direct deps); weekly `rustsec/audit-check@v2` runs in `cargo-audit.yml`; `cargo-machete.yml` flags dead deps; `cargo-semver-checks.yml` blocks semver-breaking bumps | supply-chain | 2026-06-16    |
| **R — Repudiation**     | low    | Build provenance cannot be reproduced from a clean checkout                                                          | `rust-release.yml` and `helios-cli-release.yml` build in clean `ubuntu-24.04` GitHub-hosted runners with no `actions/cache` keys that survive runs                                                              | ci-ops       | 2026-06-16    |
| **I — Info disclosure** | low    | Build artifact contains a hard-coded secret from a malicious crate                                                   | trufflehog gate + `redact_secrets` runtime scrub; no build-time secrets are baked into release binaries (`strip = true`)                                                                                        | security     | 2026-06-16    |
| **D — DoS**             | med    | Compromised dep runs a long-running `build.rs` that pins a CI runner                                                 | `cargo-deny` rejects unknown registries; `network-optimization.yml` and `policy-gate.yml` cap build minutes; standard `ubuntu-24.04` runners only (per global CI billing policy)                                | ci-ops       | 2026-06-16    |
| **E — Elevation**       | high   | Malicious `build.rs` exec's arbitrary code at build time with the CI runner's privileges                             | `rust-ci.yml` does not set `CARGO_NET_OFFLINE`; relies on crates.io allowlist + weekly audit. **Open gap:** no `cargo-crev` review or signed-lockfile verification — tracked under HEL-061 (SC3 attestation)    | supply-chain | 2026-06-16    |

### Component: CI/CD + release pipeline (`.github/workflows/`)

| Threat                  | Rating | Specific attack vector                                                                   | Mitigation                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Owner    | Last reviewed |
| ----------------------- | ------ | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------- |
| **S — Spoofing**        | med    | Compromised third-party GitHub Action runs attacker code in CI                           | Critical actions in `.github/workflows/ci.yml` are commit-pinned; `trufflehog.yml` pins its reusable workflow caller to verified commit `c43cc4af2cbcc2bb2df37f3e4ab78cc5d8c1b3ad`; repository-wide pin coverage remains a review item rather than an assumed invariant                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | ci-ops   | 2026-07-18    |
| **T — Tampering**       | high   | A pull request changes staging or build code and abuses the workflow token               | Pull-request staging executes the checked-out PR merge ref. Never expose an upstream PAT or GitHub App key directly to this mutable job; move authenticated staging behind a trusted workflow boundary first. `rust-ci.yml` limits mutable pull-request jobs to `contents: read`; `rust-ci-full.yml` limits its repository token to `contents: read`; `rust-ci-full-nextest-platform.yml` limits its repository token to `contents: read`; `sdk.yml` limits its repository token to `contents: read`; `blob-size-policy.yml` limits its repository token to `contents: read`; `python-runtime-build.yml` limits its repository token to `contents: read`; `v8-canary.yml` limits its repository token to `contents: read`; `trufflehog.yml` limits its repository token to `contents: read` | ci-ops   | 2026-07-18    |
| **R — Repudiation**     | low    | Workflow authorship or the artifact source is ambiguous                                  | Git commit and Actions run logs identify the executed revision; the pinned release fallback verifies GitHub's published SHA-256 digest and embedded package identity for every npm asset                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | ci-ops   | 2026-07-18    |
| **I — Info disclosure** | high   | Workflow logs or PR-controlled code leak a release credential                            | `.github/workflows/ci.yml` grants its repository-scoped `github.token` only `contents: read`. Its release fallback downloads exact public URLs with no token or secret                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | security | 2026-07-18    |
| **D — DoS**             | med    | PRs trigger expensive multi-platform jobs or downloads of stale artifacts                | The npm workflow has a concurrency group and ten-minute timeout. Current CI also uses macOS and Windows runners, so runner cost is not Linux-only                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | infra    | 2026-07-18    |
| **E — Elevation**       | high   | A workflow inherits broad repository permissions or a compromised cross-repository token | Repository workflow permissions currently default to `write`; 14 of 26 workflow files now declare a top-level `permissions:` block, including `ci.yml`, `rust-ci.yml`, `rust-ci-full.yml`, `rust-ci-full-nextest-platform.yml`, `sdk.yml`, `blob-size-policy.yml`, `python-runtime-build.yml`, `v8-canary.yml`, and `trufflehog.yml` with only `contents: read`. The built-in token remains limited to this repository and cannot authorize upstream artifact downloads                                                                                                                                                                                                                                                                                                                     | ci-ops   | 2026-07-18    |

### npm staging boundary (PR #605)

Evidence reviewed on 2026-07-18:

- [x] Staging failure propagation — `.github/workflows/ci.yml` does not swallow
      the staging exit status, and the required-CI contract rejects
      `continue-on-error: true`.
- [x] Successful npm staging — when the pinned `0.115.0` workflow artifacts
      report expired, staging falls back only to the exact seven public npm release
      assets on `rust-v0.115.0`. Every asset requires a published SHA-256 digest,
      matching size, exact filename and URL, and embedded package identity before use.

Missing, duplicate, or unexpected package assets, unavailable or mismatched
digests, release identity drift, and package name/version drift remain explicit
failures. No cross-repository credential is exposed to mutable pull-request code.

---

## Worked example: phenodocs (kept from template for reference)

The `phenodocs` worked example from the template remains the canonical
STRIDE-per-component walkthrough. See
[`THREAT-MODEL-TEMPLATE.md` § Worked example: phenodocs](../../../docs/audits/THREAT-MODEL-TEMPLATE.md#worked-example-phenodocs).
HeliosCLI replaces it with the four components above.

---

## How this lifts the S7 score

- **0 → 1 (ad-hoc):** `docs/security/threat-model.md` exists with at least one
  component's STRIDE table. ✅ Four components covered.
- **0 → 2 (wired):** Referenced from `README.md` and `SECURITY.md`. Covers the
  four highest-trust components. Each row has an owner and a `Last reviewed`
  date. ✅ See README/SECURITY links added in the same commit.
- **2 → 3 (measured):** Add a CI gate that fails if
  `docs/security/threat-model.md` is older than 90 days OR if a previously-
  scored component row is deleted. **Not done in this commit** — tracked
  under HEL-057 (S7-2) → S7-3 follow-up.

## Review cadence

Review this threat model:

- **On every major release** (semver minor: `rust-v0.x.0` / `v0.x.0`)
- **On any new external dependency** added to `Cargo.lock`
- **On any new public-facing subcommand** (currently: `exec`, `review`,
  `resume`, `fork`, `login`, `logout`, `sandbox`, `mcp`, `mcp-server`,
  `features`, `completion`, `app-server`, `apply`)
- **Quarterly minimum** (a 90-day-old model is a CI failure once S7-3 lands)

## How to validate

```bash
# Verify all 6 STRIDE letters appear in this file
for c in S T R I D E; do
  grep -q "^\*\*$c " docs/security/threat-model.md || echo "missing $c"
done
```

A clean run (no "missing" output) means the file is structurally valid.

## Cross-references

- `SECURITY.md` — reporting channel (Bugcrowd), disclosure policy.
- `deny.toml` — license allowlist and registry allowlist (supply-chain mitigations).
- `.github/workflows/cargo-audit.yml` — weekly `rustsec/audit-check@v2` job.
- `.github/workflows/cargo-deny.yml` — PR-gate for the `deny.toml` config.
- `.github/workflows/codeql-rust.yml` — CodeQL on the `codex-rs` workspace.
- `FLEET-AUDIT-REPORT.md` (Phenotype org) — S7 P0 backlog, priority 42.
- `docs/audits/HeliosCLI/ACTION-PLAN.md` — HEL-057 (S7) work item.

## Provenance

- **Template version:** 1.0 (Phenotype Org, 2026-06-16)
- **Customised by:** security workstream, S7 fan-out wave `audit-wave-2026-06-16`
- **Source data:** `Cargo.lock`, `deny.toml`, `codex-rs/{cli,login,secrets,core}/`,
  `.github/workflows/*` (52 files), `SECURITY.md`
- **License:** Same as parent repo (Apache-2.0)
