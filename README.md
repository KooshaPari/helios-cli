<!-- PHENOTYPE FORK CONTEXT — see "About this fork" below -->

## About this fork

`KooshaPari/helios-cli` is a Phenotype-org fork of [OpenAI Codex CLI](https://github.com/openai/codex). The upstream README is preserved verbatim below; this preamble documents fork-specific divergence so downstream consumers can tell what is upstream behavior versus a Phenotype patch.

**Why we fork:** Codex CLI is one of the agent backends wired into the Phenotype `thegent` dispatcher. We carry security and workspace fixes ahead of upstream merge cadence and ship them via this fork's tagged releases.

**Recent Phenotype-specific changes:**

- **#527 — `fix(workspace): declare 19 missing workspace dependencies`** — restores `cargo metadata`/`cargo build` resolution in `codex-rs/` after upstream workspace drift.
- **#526 — `fix(deps): bump 10 HIGH CVEs in codex-rs`** — `aws-lc-sys`, `quinn-proto`, `rustls-webpki` upgrades; closes RUSTSEC advisories that upstream had not yet absorbed.
- **#525 — `fix(workspace): unblock codex-rs cargo resolution`** — preparatory unblock for the CVE sweep.
- **#524 / #523 / #522** — OpenSSF Scorecard workflow, `CODE_OF_CONDUCT.md`, `.github/FUNDING.yml` — Phenotype-org hygiene baseline.
- **#519–#521** — pinned floating external GitHub Actions to commit SHAs across `stage-gates.yml`, `issue-labeler.yml`, `issue-deduplicator.yml`.
- **#518** — bootstrapped a VitePress docs deploy workflow.

**Where to find Phenotype-specific docs:**

- Roadmap and fork strategy: [`PLAN.md`](./PLAN.md)
- Phenotype contribution conventions: [`docs/contributing.md`](./docs/contributing.md) and the upstream `AGENTS.md` chain in `docs/agents_md.md`
- Phenotype org context: [`KooshaPari/phenotype-infrakit`](https://github.com/KooshaPari/phenotype-infrakit) and the `thegent` dispatcher

**Upstream tracking:** we periodically rebase or merge from `openai/codex@main`. Open a PR against this fork for Phenotype-specific changes; upstreamable fixes should also be PR'd to `openai/codex` directly.

---

<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
