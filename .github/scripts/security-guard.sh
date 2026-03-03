#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

PRE_COMMIT_CONFIG=".pre-commit-config.yaml"
RUN_FAST=${SECURITY_GUARD_RUN_FAST:-1}
REQUIRE_FAST_TOOLS=${SECURITY_GUARD_REQUIRE_FAST_TOOLS:-0}

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
    printf '%s\0' "$tool"
    return 0
  fi
  if command -v uvx >/dev/null 2>&1; then
    printf '%s\0' "uvx" "$tool"
    return 0
  fi
  if command -v uv >/dev/null 2>&1; then
    printf '%s\0' "uv" "tool" "run" "$tool"
    return 0
  fi
  return 1
}

run_ggshield() {
  local runner
  local -a runner_args
  if ! runner="$(resolve_runner "ggshield")"; then
    fail "ggshield is required. Install via pipx install ggshield or uv tool install ggshield"
  fi
  read -r -d '' -a runner_args <<< "$runner"
  log "running mandatory secret scan"
  "${runner_args[@]}" secret scan pre-commit
}

run_fast_optional_checks() {
  if [ "$RUN_FAST" -ne 1 ]; then
    log "FAST checks disabled (SECURITY_GUARD_RUN_FAST=0)"
    return 0
  fi

  local files cmd runner
  local -a cmd_args
  files="$(git diff --cached --name-only --diff-filter=ACM || true)"
  if [ -z "$files" ]; then
    files="$(git diff --name-only HEAD~1..HEAD 2>/dev/null || true)"
  fi
  if [ -z "$files" ]; then
    log "no changed files for FAST checks"
    return 0
  fi

  if ! runner="$(resolve_runner "codespell")"; then
    if [ "$REQUIRE_FAST_TOOLS" -eq 1 ]; then
      fail "optional fast check tool missing: codespell"
    fi
    log "codespell not found; skipping optional fast check"
    return 0
  fi
  read -r -d '' -a cmd_args <<< "$runner"

  log "running optional FAST checks (codespell)"
  echo "$files" \
    | grep -E '\.(md|txt|py|ts|tsx|js|go|rs|kt|java|yaml|yml)$' \
    | xargs -r "${cmd_args[@]}" -q 2 -L "hte,teh"
}

append_precommit_block() {
  if [ ! -f "$PRE_COMMIT_CONFIG" ]; then
    fail "missing pre-commit config: $PRE_COMMIT_CONFIG"
  fi
  if grep -Eq 'security-guard-pre-commit-pre-push|\.github/scripts/security-guard\.sh' "$PRE_COMMIT_CONFIG"; then
    log "pre-commit hook block already present"
    return 0
  fi

  cat <<'EOF' >> "$PRE_COMMIT_CONFIG"

  - repo: local
    hooks:
      - id: security-guard-pre-commit-pre-push
        name: security-guard pre-commit/pre-push
        entry: .github/scripts/security-guard.sh
        language: script
        pass_filenames: false
        stages: [pre-commit, pre-push]
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
