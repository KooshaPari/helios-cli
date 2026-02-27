#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
RELEASE_BIN="${ROOT_DIR}/target/release/helios"
LOCAL_BIN_DIR="${HOME}/.local/bin"

if [ ! -x "${RELEASE_BIN}" ]; then
  echo "Release binary not found: ${RELEASE_BIN}" >&2
  exit 1
fi

ln -sfn "${RELEASE_BIN}" "${LOCAL_BIN_DIR}/helios"
ln -sfn "${RELEASE_BIN}" "${LOCAL_BIN_DIR}/helios-dev"

echo "Registered release binary:"
ls -l "${LOCAL_BIN_DIR}/helios" "${LOCAL_BIN_DIR}/helios-dev"
echo "Version check:"
"${LOCAL_BIN_DIR}/helios" --version
