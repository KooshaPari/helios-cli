# Changelog

All notable changes to HeliosCLI are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0](https://github.com/KooshaPari/helios-cli/compare/v0.3.0...v0.4.0) (2026-08-21)


### Features

* **sre:** add chaos CI gate, Terraform IaC, SLO burn rate alerting, and OTel collector config ([edf00f3](https://github.com/KooshaPari/helios-cli/commit/edf00f30eae0718a4b53d7384c73f8366c78a3a2))
* **sre:** add SLO alerting, OTel deployment scripts, terraform validate CI ([a32d976](https://github.com/KooshaPari/helios-cli/commit/a32d976e199833e268f0e562e871a7b21ef099a4))
* **sre:** add SLO monitoring, Terraform CI validation, and performance dashboard ([20a1265](https://github.com/KooshaPari/helios-cli/commit/20a12655f94a5c44373e8e585a6e6ab04a10ce20))

## [0.3.0](https://github.com/KooshaPari/helios-cli/compare/v0.2.0...v0.3.0) (2026-08-20)


### Features

* **absorb:** add CHANGELOG.md + justfile from HeliosCLI final wave ([ae2f311](https://github.com/KooshaPari/helios-cli/commit/ae2f3112095d68a2cd3d85a3aac1b9c8c0126596))
* **absorb:** complete HeliosCLI absorption (build configs + docs + artifacts + 6 crates) ([f743027](https://github.com/KooshaPari/helios-cli/commit/f7430274a3ce8e98664973af0cb7ca804142c690))
* **absorb:** merge HeliosCLI workspace (20 crates + root config + 9 dirs) into helios-cli ([fc6a792](https://github.com/KooshaPari/helios-cli/commit/fc6a792156f03beef90d04394a138f5d42b5d2e3))
* add i18n Rust module and locale files, perf baselines ([fcf0eb1](https://github.com/KooshaPari/helios-cli/commit/fcf0eb18cdc2963482cf5ae27bf33a763c05b50e))
* add integration tests, release-please, SBOM workflow, and i18n locales ([94415dd](https://github.com/KooshaPari/helios-cli/commit/94415dd938ab817de327ca530aed75c15fb89d9f))
* **agileplus:** bootstrap full AgilePlus setup with 31-pillar scorecard, sprint tracking, quality gates ([d24cc21](https://github.com/KooshaPari/helios-cli/commit/d24cc21ef3c75153b6465047a07535293c568d64))
* **dashboard:** migrate UI components from helios-router ([1f4df51](https://github.com/KooshaPari/helios-cli/commit/1f4df514d5b421ed57d466e7fe985796221b803d))
* **devex:** add ADRs, DORA metrics, Docker dev env, and incident response playbook ([b5dabc1](https://github.com/KooshaPari/helios-cli/commit/b5dabc1856df1febf8c29768c2d05432bdd1f2df))
* **fuzz:** add corpus seeds, perf trend tracking, and SLA/SLO docs ([271983b](https://github.com/KooshaPari/helios-cli/commit/271983b387e50a01ea411e44fc06f839ad9361cd))
* **harness_recorder:** absorb KLA (KommandLineAutomation) as harness_recorder crate (L5-200) ([#600](https://github.com/KooshaPari/helios-cli/issues/600)) ([eccd9de](https://github.com/KooshaPari/helios-cli/commit/eccd9de1034e17575dcbe7ee71fa998ba111dd4c))
* **harness_runner:** dual-harness shared-3task fixture adapter ([027e3ca](https://github.com/KooshaPari/helios-cli/commit/027e3cad0b13d3af985dd745a16d6e3caf83d9ac))
* **harness_runner:** dual-harness shared-3task fixture adapter ([d7c718e](https://github.com/KooshaPari/helios-cli/commit/d7c718e7a26b4250aeff12c5965152880e271a7f))
* **helios:** land benchmark-provenance + harness-preservation + toolchain-refresh ([#628](https://github.com/KooshaPari/helios-cli/issues/628)) ([3f44dec](https://github.com/KooshaPari/helios-cli/commit/3f44dec84384f80666e9ffc5239270b2d1a8fa02))
* **i18n:** add internationalization scaffolding with English locale ([e480687](https://github.com/KooshaPari/helios-cli/commit/e4806873b78f6d4dbece25f41d28153fd1f1c9d4))
* **infra:** add OpenTelemetry, chaos testing, perf dashboard, and multi-region docs ([24c45db](https://github.com/KooshaPari/helios-cli/commit/24c45dbd0093ee7498c5e62ff41639418cef658b))
* **metrics:** add observability metrics collector with tests + dashboard workflow ([e60efee](https://github.com/KooshaPari/helios-cli/commit/e60efee5b0c89fe41d95927613160962fa04d080))
* **testing:** add fuzz harnesses, wire i18n into CLI, 3 new locales, and codeowners verification ([81ea651](https://github.com/KooshaPari/helios-cli/commit/81ea651807e7ccbb31f3f0db88070d4effd762ea))


### Bug Fixes

* **ci:** add codex-cli/README.md stub required by ASCII check CI ([#581](https://github.com/KooshaPari/helios-cli/issues/581)) ([43586a4](https://github.com/KooshaPari/helios-cli/commit/43586a4bbd16c2a2a9e9207d9404e7cbec783340))
* **ci:** add missing scripts/asciicheck.py and readme_toc.py from upstream ([#577](https://github.com/KooshaPari/helios-cli/issues/577)) ([ce12d71](https://github.com/KooshaPari/helios-cli/commit/ce12d7103bec2ce3d7d8869a483527541215297f))
* **ci:** add root README.md required by ASCII check CI step ([#579](https://github.com/KooshaPari/helios-cli/issues/579)) ([fa01135](https://github.com/KooshaPari/helios-cli/commit/fa0113523c52dc467b7a1324836606bcd2388410))
* **ci:** apply oxfmt formatting to workflow files and docs ([#582](https://github.com/KooshaPari/helios-cli/issues/582)) ([de60cde](https://github.com/KooshaPari/helios-cli/commit/de60cde63f884ac926dde453a03b3de29702a22c))
* **ci:** apply rustfmt for coverage test modules ([3e89105](https://github.com/KooshaPari/helios-cli/commit/3e89105e50735e92d8fad4d3dc5335da3295cd5c))
* **ci:** close PR gates with AgilePlus trace ([68872cd](https://github.com/KooshaPari/helios-cli/commit/68872cddf53338157872e604031dc3193b7bb39c))
* **ci:** gate kla PTY test_settings on target OS for clippy ([b0df018](https://github.com/KooshaPari/helios-cli/commit/b0df018fd3a252c380cde7af4f47315101dfb69c))
* **ci:** hard-fork harness green - ascii, deny, disable bazel/sdk/ACL ([b9ff1a4](https://github.com/KooshaPari/helios-cli/commit/b9ff1a4b0f0f8f9861b9d5f8d8c0983038859898))
* **ci:** hard-fork harness green - ascii, deny, disable bazel/sdk/ACL ([97cc436](https://github.com/KooshaPari/helios-cli/commit/97cc436d5d1666570520c09d5877a2f7b0029244))
* **ci:** ignore transitive rustsec advisories; skip codespell noise ([3745d19](https://github.com/KooshaPari/helios-cli/commit/3745d19217c5fff5653fa3ca903015c0f666583e))
* **ci:** load .codespellrc in Codespell workflow ([95212a8](https://github.com/KooshaPari/helios-cli/commit/95212a85e1592dea451557b42875e49c6b81a827))
* **ci:** migrate trunk.yaml to schema v0.1 (lint/fmt blocks) ([#626](https://github.com/KooshaPari/helios-cli/issues/626)) ([3a4c0fd](https://github.com/KooshaPari/helios-cli/commit/3a4c0fd7dbee97f40c430d400702c8749a92d793))
* **ci:** pin codeql upload-sarif to valid v3.37.6 SHA in scorecard ([#624](https://github.com/KooshaPari/helios-cli/issues/624)) ([8973cb8](https://github.com/KooshaPari/helios-cli/commit/8973cb866fed9fa3422cb0ee793b334dd9959677))
* **ci:** pin trunk-action to valid v1.3.1 SHA in trunk-check ([#623](https://github.com/KooshaPari/helios-cli/issues/623)) ([a329fff](https://github.com/KooshaPari/helios-cli/commit/a329fffaaf5c5fbecd3f462a1730592c6b21b60d))
* **ci:** replace broken trunk-action with prettier-scoped check ([#631](https://github.com/KooshaPari/helios-cli/issues/631)) ([0449d4e](https://github.com/KooshaPari/helios-cli/commit/0449d4e3a22854805993a9eff31a8f280077eaa0))
* **ci:** replace em-dash with ASCII hyphen in README (asciicheck fix) ([#580](https://github.com/KooshaPari/helios-cli/issues/580)) ([7665846](https://github.com/KooshaPari/helios-cli/commit/7665846bb4145e24edddd16e35326e218db0c668))
* **ci:** resolve merge conflict markers and invalid action refs in ci.yml ([#574](https://github.com/KooshaPari/helios-cli/issues/574)) ([62ae525](https://github.com/KooshaPari/helios-cli/commit/62ae525843d06218c8371e50f295542a885ef0f8))
* **ci:** restore main CI ([#622](https://github.com/KooshaPari/helios-cli/issues/622)) ([fc14336](https://github.com/KooshaPari/helios-cli/commit/fc14336bd3c6813634c0c5b2821bfd6a5b5e70be))
* **ci:** restore working trunk config on main ([2eb20d4](https://github.com/KooshaPari/helios-cli/commit/2eb20d4c50b331959861375c3f570ffd98fefee2))
* **ci:** restore working trunk config on main ([#629](https://github.com/KooshaPari/helios-cli/issues/629)) ([8cbd6a6](https://github.com/KooshaPari/helios-cli/commit/8cbd6a69962bf90c6f00f4c7fc5068b819b7db07))
* **ci:** use python3 explicitly for script invocations (permission fix) ([#578](https://github.com/KooshaPari/helios-cli/issues/578)) ([22aa970](https://github.com/KooshaPari/helios-cli/commit/22aa970093043fa5b36dc3756a1a9f05a2d55434))
* **codex-client:** add url dep for TransportError ([3c2b322](https://github.com/KooshaPari/helios-cli/commit/3c2b322bb76061c3537e36509966f4ba54c775cb))
* **codex-rs:** add model-provider crates to workspace manifest ([1c83322](https://github.com/KooshaPari/helios-cli/commit/1c83322f922a35e43a27bcd79ce0d2d6779bef30))
* **codex-rs:** align sqlx 0.9 + v8 149 for sqlite links ([09cba62](https://github.com/KooshaPari/helios-cli/commit/09cba622441972ee6fdd1aff26f04db783a3abce))
* **codex-rs:** backfill workspace deps for codex-cli manifest graph ([e161c7c](https://github.com/KooshaPari/helios-cli/commit/e161c7c425aeacab5e0286cdb26bd378e91cff5b))
* **codex-rs:** restore 57 missing source files from snapshot ([eed4e1d](https://github.com/KooshaPari/helios-cli/commit/eed4e1d973ae919d327f5aeccce639943da4d4db))
* **codex-rs:** restore 6 missing include and source files ([7846d4c](https://github.com/KooshaPari/helios-cli/commit/7846d4c6eaa5f6b7478dbae4bacb2f1678f1b460))
* **codex-rs:** restore additional missing upstream source modules ([00e9504](https://github.com/KooshaPari/helios-cli/commit/00e9504f98e64b7b529b495baa2433e72b14b27c))
* **codex-rs:** restore agent_job model for helios fork state runtime ([a58572b](https://github.com/KooshaPari/helios-cli/commit/a58572b23758baf5bb910eff74dd9c62454d2fcd))
* **codex-rs:** restore codex-skills crate manifest from upstream ([feab082](https://github.com/KooshaPari/helios-cli/commit/feab082dc1ada63126ac980a75960148b206de1a))
* **codex-rs:** restore explorer builtin role config ([50cdd8f](https://github.com/KooshaPari/helios-cli/commit/50cdd8f8b10ba388231b4f59d4ec27be6b4900dd))
* **codex-rs:** restore file-search and utils/cli sources from upstream ([463a6a6](https://github.com/KooshaPari/helios-cli/commit/463a6a67dbcf5ea00572236bbdf951f44b7757f4))
* **codex-rs:** restore json-to-toml and prune missing workspace members ([d6a0dd9](https://github.com/KooshaPari/helios-cli/commit/d6a0dd9ba98a97f65b35ed80bf210c411313d70d))
* **codex-rs:** restore login pkce module ([f067ce4](https://github.com/KooshaPari/helios-cli/commit/f067ce47feee8b08b296a05549ea50c75d4b1e99))
* **codex-rs:** restore missing openapi-models and protocol sources ([fe6d878](https://github.com/KooshaPari/helios-cli/commit/fe6d87840d0e21c61906afdd1199885103e02e50))
* **codex-rs:** restore missing secrets sanitizer and otel metrics modules ([f96ff48](https://github.com/KooshaPari/helios-cli/commit/f96ff489de630fd576368afdb139469e2b47ea8b))
* **codex-rs:** restore tui animation frame assets from snapshot ([eec6db7](https://github.com/KooshaPari/helios-cli/commit/eec6db7e77e8dbbea21df680c067e97bcbd3c311))
* **codex-rs:** restore web_search_detail from snapshot ([c4e9e32](https://github.com/KooshaPari/helios-cli/commit/c4e9e3207a2dbb16b9e07bfd0ceeeae8b57be1d0))
* **codex-rs:** restore workspace_acl and config overrides modules ([f63dc3f](https://github.com/KooshaPari/helios-cli/commit/f63dc3f8019d9064c528fdfcd774704c186ec915))
* **codex-rs:** starlark 0.14 + restore codex-client error module ([9fdab01](https://github.com/KooshaPari/helios-cli/commit/9fdab01fde101a4d53623e2f1c109c5142b27e61))
* **codex-rs:** sync state model from upstream and restore backfill_state ([d19d576](https://github.com/KooshaPari/helios-cli/commit/d19d576fbf20b5a8f0f2542c88e463f12d558448))
* **codex-rs:** TransportError url as String + rustls aws_lc_rs ([a2aa4cb](https://github.com/KooshaPari/helios-cli/commit/a2aa4cbc96d6350428f71d80cfa8234d430bd6c8))
* **codex-rs:** use ReviewDecision::Denied for mcp-server approvals ([343b7c2](https://github.com/KooshaPari/helios-cli/commit/343b7c2489b079e689de6006bf16b2c4d4106b35))
* **deps:** bump 10 HIGH CVEs in codex-rs (aws-lc-sys, quinn-proto, rustls-webpki) ([#526](https://github.com/KooshaPari/helios-cli/issues/526)) ([e5b1157](https://github.com/KooshaPari/helios-cli/commit/e5b1157911c1dc04e0bd8dab55e2055a60f9fbe9))
* **helios-cli:** add workspace.dependencies to resolve criterion reference ([#563](https://github.com/KooshaPari/helios-cli/issues/563)) ([63cf3a7](https://github.com/KooshaPari/helios-cli/commit/63cf3a70349472e5974d77baba20d606015ffd24))
* **p3:** clippy clean + fmt + test coverage ([#603](https://github.com/KooshaPari/helios-cli/issues/603)) ([82dce95](https://github.com/KooshaPari/helios-cli/commit/82dce95f81cd1d77c2c27ef928edb5ce9b83672c))
* **recorder:** avoid duplicate logger initialization ([ee70a02](https://github.com/KooshaPari/helios-cli/commit/ee70a0295c2d848f4158eb9d1f6e3b1c9c9222f3))
* resolve merge conflicts in codex-rs workspace (143 tests passing) ([#559](https://github.com/KooshaPari/helios-cli/issues/559)) ([eae259b](https://github.com/KooshaPari/helios-cli/commit/eae259bbcfb072fdefad5ed1566d7f764b2f70b6))
* **workspace:** declare 19 missing workspace dependencies ([#527](https://github.com/KooshaPari/helios-cli/issues/527)) ([2b06f43](https://github.com/KooshaPari/helios-cli/commit/2b06f437df13767aa9ecb39937f3db5a211fd353))
* **workspace:** unblock codex-rs cargo resolution ([#525](https://github.com/KooshaPari/helios-cli/issues/525)) ([0b2bc0f](https://github.com/KooshaPari/helios-cli/commit/0b2bc0fb661a7252ba164f6a1ee4cda637e0fa85))
* **workspace:** unblock codex-rs cargo resolution ([#570](https://github.com/KooshaPari/helios-cli/issues/570)) ([775f24f](https://github.com/KooshaPari/helios-cli/commit/775f24fe934dcbe6e9aad02084ead2d0b0f89ed9))

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
