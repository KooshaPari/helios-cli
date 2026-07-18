#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const manifestPath = join(repoRoot, "scripts", "oxfmt-targets.json");
const { targets } = JSON.parse(readFileSync(manifestPath, "utf8"));
const mode = process.argv[2];

if (!new Set(["--check", "--write"]).has(mode) || process.argv.length !== 3) {
  console.error("usage: node scripts/format-targets.mjs <--check|--write>");
  process.exit(2);
}

const oxfmtBin = join(repoRoot, "node_modules", "oxfmt", "bin", "oxfmt");
const result = spawnSync(process.execPath, [oxfmtBin, mode, ...targets], {
  cwd: repoRoot,
  stdio: "inherit"
});

if (result.error) {
  throw result.error;
}
process.exit(result.status ?? 1);
