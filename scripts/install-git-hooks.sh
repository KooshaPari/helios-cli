#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

chmod +x \
  "$ROOT_DIR/.github/hooks/pre-commit" \
  "$ROOT_DIR/.github/hooks/pre-push" \
  "$ROOT_DIR/.github/hooks/security-guard.sh" \
  "$ROOT_DIR/.github/scripts/security-guard.sh" \
  "$ROOT_DIR/scripts/prepush-ci.sh"

git -C "$ROOT_DIR" config core.hooksPath .github/hooks
"$ROOT_DIR/.github/scripts/security-guard.sh" --ensure-pre-commit-block

echo "Installed repo-local git hooks via core.hooksPath=.github/hooks"
echo "Hooks:"
echo "  pre-commit -> pre-commit run --hook-stage pre-commit"
echo "  pre-push   -> pre-commit run --hook-stage pre-push"
