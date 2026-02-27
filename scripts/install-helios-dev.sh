#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
install_root="${HELIOS_INSTALL_ROOT:-${HOME}/.local}"
local_bin="${install_root}/bin"
cli_package_dir="${HELIOS_CLI_PACKAGE_DIR:-${repo_root}/helios-rs/cli}"
binary_name="helios"
binary_dest="${local_bin}/${binary_name}"
binary_dest_dev="${local_bin}/helios-dev"
binary_dest_compat="${local_bin}/codex"
keep_codex_compat="${HELIOS_KEEP_CODEX_COMPAT:-1}"
profile="release"

for arg in "$@"; do
  if [[ "${arg}" == "--dev" ]]; then
    profile="debug"
  elif [[ "${arg}" == "--no-codex" ]]; then
    keep_codex_compat="0"
  elif [[ "${arg}" == "--help" || "${arg}" == "-h" ]]; then
    cat <<'USAGE'
Usage:
  install-helios-dev.sh [--dev] [--no-codex]

Options:
  --dev        install debug binary instead of release
  --no-codex   skip creating the optional codex compatibility binary
USAGE
    exit 0
  else
    echo "Unknown argument: ${arg}" >&2
    exit 1
  fi
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found in PATH. Install Rust/Cargo before running this script." >&2
  exit 1
fi

mkdir -p "${local_bin}"

cd "${repo_root}"

install_args=(
  --path "${cli_package_dir}"
  --bin "${binary_name}"
  --root "${install_root}"
  --locked
  --force
)
if [[ "${profile}" == "debug" ]]; then
  install_args+=(--debug)
fi

cargo install "${install_args[@]}"

if [[ ! -x "${binary_dest}" ]]; then
  echo "Helios binary was not installed to ${binary_dest}" >&2
  exit 1
fi

cp -f "${binary_dest}" "${binary_dest_dev}"

if [[ "${keep_codex_compat}" == "1" ]]; then
  cp -f "${binary_dest}" "${binary_dest_compat}"
  echo "Kept codex compatibility alias at ${binary_dest_compat}"
fi

echo "Installed Helios binary to ${binary_dest} (${profile})"
echo "Installed dev entrypoint to ${binary_dest_dev}"
