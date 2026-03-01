#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Compute changed files from env vars, fallback to all tracked files
if [[ -n "${AIRLOCK_BASE_SHA:-}" && -n "${AIRLOCK_HEAD_SHA:-}" ]]; then
  CHANGED_FILES=$(git diff --name-only --diff-filter=ACMR "$AIRLOCK_BASE_SHA" "$AIRLOCK_HEAD_SHA" 2>/dev/null || true)
elif [[ -n "${AIRLOCK_BASE_SHA:-}" ]]; then
  CHANGED_FILES=$(git diff --name-only --diff-filter=ACMR "$AIRLOCK_BASE_SHA" HEAD 2>/dev/null || true)
else
  CHANGED_FILES=$(git diff --name-only --diff-filter=ACMR HEAD~1 HEAD 2>/dev/null || true)
fi

if [[ -z "$CHANGED_FILES" ]]; then
  echo "No changed files detected."
  exit 0
fi

# Filter helpers
filter_ext() {
  local exts="$1"
  echo "$CHANGED_FILES" | grep -E "\\.($exts)$" || true
}

ERRORS=0

# ── Prettier (JS/JSON/MD/YAML) ──
PRETTIER_FILES=$(filter_ext "js|mjs|ts|tsx|json|md|yml|yaml")
if [[ -n "$PRETTIER_FILES" ]]; then
  # Filter out prettierignored files by letting prettier handle it
  echo "==> Prettier: formatting..."
  echo "$PRETTIER_FILES" | xargs npx prettier --write --ignore-unknown 2>/dev/null || true
  echo "==> Prettier: checking..."
  if ! echo "$PRETTIER_FILES" | xargs npx prettier --check --ignore-unknown 2>/dev/null; then
    echo "FAIL: Prettier check failed"
    ERRORS=$((ERRORS + 1))
  fi
fi

# ── Codespell ──
SPELL_FILES=$(filter_ext "md|py|sh|mjs|js|ts|yml|yaml|bzl|patch|txt")
if [[ -n "$SPELL_FILES" ]]; then
  CODESPELL="${CODESPELL_BIN:-codespell}"
  if ! command -v "$CODESPELL" &>/dev/null; then
    CODESPELL="$HOME/Library/Python/3.9/bin/codespell"
  fi
  if command -v "$CODESPELL" &>/dev/null; then
    echo "==> Codespell: checking..."
    # Use project config; only check changed files that exist
    EXISTING_SPELL_FILES=""
    while IFS= read -r f; do
      [[ -f "$f" ]] && EXISTING_SPELL_FILES="$EXISTING_SPELL_FILES $f"
    done <<< "$SPELL_FILES"
    if [[ -n "$EXISTING_SPELL_FILES" ]]; then
      if ! $CODESPELL $EXISTING_SPELL_FILES 2>/dev/null; then
        echo "WARN: Codespell found issues (non-blocking)"
      fi
    fi
  else
    echo "SKIP: codespell not found"
  fi
fi

# ── ShellCheck ──
SHELL_FILES=$(filter_ext "sh")
if [[ -n "$SHELL_FILES" ]]; then
  if command -v shellcheck &>/dev/null; then
    echo "==> ShellCheck: checking..."
    EXISTING_SHELL=""
    while IFS= read -r f; do
      [[ -f "$f" ]] && EXISTING_SHELL="$EXISTING_SHELL $f"
    done <<< "$SHELL_FILES"
    if [[ -n "$EXISTING_SHELL" ]]; then
      if ! shellcheck --severity=warning $EXISTING_SHELL 2>/dev/null; then
        echo "WARN: ShellCheck found issues (non-blocking)"
      fi
    fi
  else
    echo "SKIP: shellcheck not found"
  fi
fi

if [[ $ERRORS -gt 0 ]]; then
  echo "Lint completed with $ERRORS error(s)."
  exit 1
fi

echo "All checks passed."
exit 0
