#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const expectedDerivedOutputs = [
  "assets/chunks/framework.Dn7Y7LSn.js",
  "assets/chunks/theme.BNMHX2Hd.js",
  "docs/merged.md"
];
const expectedTargets = [
  ".github/workflows/bazel.yml",
  ".github/workflows/blob-size-policy.yml",
  ".github/workflows/cla.yml",
  ".github/workflows/close-stale-contributor-prs.yml",
  ".github/workflows/codespell.yml",
  ".github/workflows/issue-deduplicator.yml",
  ".github/workflows/issue-labeler.yml",
  ".github/workflows/issue-translator.yml",
  ".github/workflows/python-runtime-build.yml",
  ".github/workflows/python-runtime-release.yml",
  ".github/workflows/python-sdk-release.yml",
  ".github/workflows/rust-ci-full-nextest-platform.yml",
  ".github/workflows/rust-ci-full.yml",
  ".github/workflows/rust-ci.yml",
  ".github/workflows/rust-release-argument-comment-lint.yml",
  ".github/workflows/rust-release-prepare.yml",
  ".github/workflows/rust-release-windows.yml",
  ".github/workflows/rust-release-zsh.yml",
  ".github/workflows/rust-release.yml",
  ".github/workflows/rusty-v8-release.yml",
  ".github/workflows/sdk.yml",
  ".github/workflows/v8-canary.yml",
  "ADR.md",
  "AGENTS.md",
  "CHANGELOG.md",
  "CLAUDE.md",
  "README.md",
  "SECURITY.md",
  "codex-cli/bin/codex.js",
  "codex-rs/responses-api-proxy/npm/bin/codex-responses-api-proxy.js",
  "codex-rs/skills/src/assets/samples/openai-docs/scripts/resolve-latest-model-info.js",
  "docs/CHANGELOG.md",
  "docs/SSOT.md",
  "docs/WORKLOG.md",
  "docs/contributing.md",
  "docs/js_repl.md",
  "docs/perf-local-benchmark.md",
  "docs/slsa.md",
  "dprint.json",
  "pyrightconfig.json"
];
const expectedAuthoredJavaScript = [
  "codex-cli/bin/codex.js",
  "codex-rs/responses-api-proxy/npm/bin/codex-responses-api-proxy.js",
  "codex-rs/skills/src/assets/samples/openai-docs/scripts/resolve-latest-model-info.js"
];

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function samePaths(actual, expected) {
  return actual.length === expected.length && actual.every((path, index) => path === expected[index]);
}

const manifest = JSON.parse(readFileSync(join(repoRoot, "scripts", "oxfmt-targets.json"), "utf8"));
const config = JSON.parse(readFileSync(join(repoRoot, ".oxfmtrc.json"), "utf8"));

assert(samePaths(manifest.targets, expectedTargets), "formatter targets must remain the hosted 40-path inventory");
assert(samePaths(manifest.derivedOutputs, expectedDerivedOutputs), "formatter exclusions must remain the three proved derived outputs");
assert(new Set(manifest.targets).size === manifest.targets.length, "formatter targets must not contain duplicates");
assert(new Set(manifest.derivedOutputs).size === manifest.derivedOutputs.length, "formatter exclusions must not contain duplicates");
assert(manifest.targets.length === 40, "formatter must retain all 40 maintainable paths");
assert(samePaths(config.ignorePatterns, ["assets/chunks/**", "docs/merged.md"]), "Oxfmt ignorePatterns must remain exact");

for (const path of [...manifest.targets, ...manifest.derivedOutputs]) {
  assert(existsSync(join(repoRoot, path)), `formatter inventory path is missing: ${path}`);
}
for (const path of expectedAuthoredJavaScript) {
  assert(manifest.targets.includes(path), `authored JavaScript must remain formatter-covered: ${path}`);
}

console.log("Oxfmt target contract is truthful: 40 maintained paths, 3 derived exclusions, authored JavaScript covered");
