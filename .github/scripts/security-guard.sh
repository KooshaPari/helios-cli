#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

PRE_COMMIT_CONFIG=".pre-commit-config.yaml"
RUN_FAST=${SECURITY_GUARD_RUN_FAST:-1}
REQUIRE_FAST_TOOLS=${SECURITY_GUARD_REQUIRE_FAST_TOOLS:-0}
SCAN_MODE=${SECURITY_GUARD_SCAN_MODE:-pre-commit}
RESOLVED_RUNNER=()

log() {
  echo "[security-guard] $*"
}

fail() {
  echo "[security-guard] ERROR: $*" >&2
  exit 1
}

resolve_runner() {
  local tool="$1"
  if command -v "$tool" >/dev/null 2>&1; then
    RESOLVED_RUNNER=("$tool")
    return 0
  fi
  if command -v uvx >/dev/null 2>&1; then
    RESOLVED_RUNNER=("uvx" "$tool")
    return 0
  fi
  if command -v uv >/dev/null 2>&1; then
    RESOLVED_RUNNER=("uv" "tool" "run" "$tool")
    return 0
  fi
  return 1
}

run_ggshield() {
  local -a runner_args
  if ! resolve_runner "ggshield"; then
    fail "ggshield is required. Install via pipx install ggshield or uv tool install ggshield"
  fi
  runner_args=("${RESOLVED_RUNNER[@]}")
  log "running mandatory secret scan (${SCAN_MODE})"
  "${runner_args[@]}" secret scan "${SCAN_MODE}"
}

run_fast_optional_checks() {
  if [ "$RUN_FAST" -ne 1 ]; then
    log "FAST checks disabled (SECURITY_GUARD_RUN_FAST=0)"
    return 0
  fi

  local files
  local filtered
  local -a cmd_args
  files="$(git diff --cached --name-only --diff-filter=ACM || true)"
  if [ -z "$files" ]; then
    files="$(git diff --name-only HEAD~1..HEAD 2>/dev/null || true)"
  fi
  if [ -z "$files" ]; then
    log "no changed files for FAST checks"
    return 0
  fi

  if ! resolve_runner "codespell"; then
    if [ "$REQUIRE_FAST_TOOLS" -eq 1 ]; then
      fail "optional fast check tool missing: codespell"
    fi
    log "codespell not found; skipping optional fast check"
    return 0
  fi
  cmd_args=("${RESOLVED_RUNNER[@]}")

  log "running optional FAST checks (codespell)"
  filtered="$(printf '%s\n' "$files" | grep -E '\.(md|txt|py|ts|tsx|js|go|rs|kt|java|yaml|yml)$' || true)"
  if [ -z "$filtered" ]; then
    log "no FAST-check eligible files found"
    return 0
  fi
  printf '%s\n' "$filtered" | xargs -r "${cmd_args[@]}" -q 2 -L "hte,teh"
}

append_precommit_block() {
  if [ ! -f "$PRE_COMMIT_CONFIG" ]; then
    fail "missing pre-commit config: $PRE_COMMIT_CONFIG"
  fi
  if grep -Eq 'security-guard-pre-commit|security-guard-pre-push|\.github/scripts/security-guard\.sh' "$PRE_COMMIT_CONFIG"; then
    log "pre-commit hook block already present"
    return 0
  fi

  cat <<'EOF' >> "$PRE_COMMIT_CONFIG"

  - repo: local
    hooks:
      - id: security-guard-pre-commit
        name: security-guard pre-commit
        entry: .github/scripts/security-guard.sh
        language: script
        pass_filenames: false
        stages: [pre-commit]
      - id: security-guard-pre-push
        name: security-guard pre-push
        entry: .github/scripts/security-guard.sh
        language: script
        pass_filenames: false
        stages: [pre-push]
        env:
          SECURITY_GUARD_SCAN_MODE: pre-push
EOF
  log "appended pre-commit hook block"
}

main() {
  if [ "${1-}" = "--ensure-pre-commit-block" ]; then
    append_precommit_block
    return 0
  fi

  run_ggshield
  run_fast_optional_checks
}

main "$@"
