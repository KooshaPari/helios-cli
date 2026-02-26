set working-directory := "helios-rs"
set positional-arguments

# Display help
help:
    just -l

# Build and install a dev copy in ~/.local/bin with a stable entrypoint pair:
# ~/.local/bin/helios and ~/.local/bin/helios-dev.
build-dev *args:
    cargo build --release -p helios-cli --bin helios "$@"
    mkdir -p ~/.local/bin
    ln -sf "$(pwd)/target/release/helios" ~/.local/bin/helios
    ln -sf ~/.local/bin/helios ~/.local/bin/helios-dev

install-dev *args:
    just build-dev "$@"

# Install to system path (non-dev, no local dev symlink).
install *args:
    cargo install --locked --force --path ./cli --package helios-cli --bin helios --root ~/.local "$@"
    ln -sf ~/.local/bin/helios ~/.local/bin/helios-dev

debug *args:
    cargo run --quiet --bin helios -- "$@"

# Backward-compatible extension/IDE entrypoint in just-based command form.
debug-helios *args:
    cargo run --quiet --bin helios -- "$@"

# `helios`
alias h := helios
alias c := helios
helios *args:
    cargo run --bin helios -- "$@"

# `helios exec`
exec *args:
    cargo run --bin helios -- exec "$@"

# Run the CLI version of the file-search crate.
file-search *args:
    cargo run -p helios-file-search -- "$@"

# Run the Rust CLI directly in release mode (useful as DX startup entry)
start *args:
    cargo run --release --bin helios -- "$@"

build *args:
    cargo build -p helios-cli "$@"

build-release *args:
    cargo build --release -p helios-cli "$@"

# Build the CLI and run the app-server test client
app-server-test-client *args:
    cargo build -p helios-app-server-test-client
    cargo run -p helios-app-server-test-client -- --helios-bin ./target/debug/helios "$@"

# format code
fmt:
    cargo fmt -- --config imports_granularity=Item 2>/dev/null

fix *args:
    cargo clippy --fix --tests --allow-dirty "$@"

clippy:
    cargo clippy --tests "$@"

deps:
    rustup show active-toolchain
    cargo fetch

# Run `cargo nextest` since it's faster than `cargo test`, though including
# --no-fail-fast is important to ensure all tests are run.
#
# Run `cargo install cargo-nextest` if you don't have it installed.
# Prefer this for routine local runs; use explicit `cargo test --all-features`
# only when you specifically need full feature coverage.
test:
    cargo nextest run --no-fail-fast

[no-cd]
bazel-lock-update:
    bazel mod deps --lockfile_mode=update

[no-cd]
bazel-lock-check:
    if ! bazel mod deps --lockfile_mode=error; then
      echo "MODULE.bazel.lock is out of date."
      echo "Run 'just bazel-lock-update' and commit the updated lockfile."
      exit 1
    fi

bazel-test:
    bazel test //... --keep_going

bazel-remote-test:
    bazel test //... --config=remote --platforms=//:rbe --keep_going

build-for-release:
    bazel build //helios-rs/cli:release_binaries --config=remote

# Run the MCP server
mcp-server-run *args:
    cargo run -p helios-mcp-server -- "$@"

# Regenerate the json schema for config.toml from the current config types.
write-config-schema:
    cargo run -p helios-core --bin helios-write-config-schema

# Regenerate vendored app-server protocol schema artifacts.
write-app-server-schema *args:
    cargo run -p helios-app-server-protocol --bin write_schema_fixtures -- "$@"

# Tail logs from the state SQLite database
log *args:
    if [ "${1:-}" = "--" ]; then shift; fi; cargo run -p helios-state --bin logs_client -- "$@"
