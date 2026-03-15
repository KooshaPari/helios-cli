#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SKIP_ALL="${SKIP_LOCAL_PREPUSH:-0}"
SKIP_BAZEL="${PREPUSH_SKIP_BAZEL:-0}"
SKIP_NAMESPACE="${PREPUSH_SKIP_NAMESPACE_AUDIT:-0}"

log() {
  echo "[prepush-ci] $*"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[prepush-ci] missing required command: $1" >&2
    exit 1
  fi
}

run_step() {
  local label="$1"
  shift
  log "running: ${label}"
  "$@"
}

if [[ "$SKIP_ALL" == "1" ]]; then
  log "skipped because SKIP_LOCAL_PREPUSH=1"
  exit 0
fi

require_cmd just
require_cmd cargo
require_cmd python3

cd "$ROOT_DIR"

run_step "rust fmt check" just fmt-check
run_step "rust lint surface" just surface-lint
run_step "rust test surface" just surface-test
run_step "feature matrix" just feature-matrix
run_step "bazel lock check" just bazel-lock-check
run_step "patch superset check" just patch-superset-check

if [[ "$SKIP_NAMESPACE" == "1" ]]; then
  log "skipping namespace audit because PREPUSH_SKIP_NAMESPACE_AUDIT=1"
else
  require_cmd gh
  run_step "namespace audit" bash scripts/namespace-audit.sh --repo KooshaPari/helios-cli --dry-run --only-prs
fi

if [[ "$SKIP_BAZEL" == "1" ]]; then
  log "skipping bazel because PREPUSH_SKIP_BAZEL=1"
else
  require_cmd bazel
  run_step "bazel test" just bazel-test
fi

log "all local billed-runner replacement checks passed"
