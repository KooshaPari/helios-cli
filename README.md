<p align="center"><code>npm i -g @phenotype/helios</code><br />or <code>brew install --cask helios</code></p>
<p align="center"><strong>Helios CLI</strong> is a coding agent from Phenotype that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Helios CLI splash" width="80%" />
</p>
</br>
If you want Helios in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>helios app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Helios App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from Phenotype, <strong>Helios Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Helios CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @phenotype/helios
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then run `helios` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `helios-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `helios-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `helios-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `helios-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `helios-x86_64-unknown-linux-musl`), so you likely want to rename it to `helios` after extracting it.

</details>

### Using Helios with your ChatGPT plan

Run `helios` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Helios as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Helios with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Helios Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
