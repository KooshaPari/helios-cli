# Changelog

All notable changes to HeliosCLI are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `.github/CODEOWNERS` — `@kooshapari` owns all paths by default, with explicit rules
  for tier-0 governance files, Rust workspace, vendored upstreams, Python harness,
  Bazel config, and documentation.
- `.github/PULL_REQUEST_TEMPLATE.md` — standard PR template (Summary / Changes /
  Testing / Related).
- `.github/ISSUE_TEMPLATE/` — issue templates for bugs, feature requests, docs
  issues, and Codex app / extension / CLI categories.
- `justfile` — task runner recipes for governance verification, build, test, lint,
  format, and release workflows.
- `CHANGELOG.md` (this file) — central record of notable changes.

### Notes

- Repo is in a post-`absorb` archival state pending `helios-cli` consolidation.
  See commit history (HEAD `767377a`) for the absorption transition.
- Tier-0 governance files (CODEOWNERS, PR template, issue templates, justfile,
  CHANGELOG) are explicitly owned by `@kooshapari` and require review before
  changes are merged.
