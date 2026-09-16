#!/usr/bin/env bash
# Prepare a VM sandbox for building the ported demos.
#
# What has to be true when this finishes:
#   1. `cargo`, the wasm32 target, and `trunk` are on PATH for a NON-INTERACTIVE shell (sandbox
#      tasks do not get the devcontainer's interactive profile).
#   2. The app has already been built once, in the SAME profile the dev-server task serves
#      (`trunk serve --release`). CodeSandbox waits only 60 SECONDS for a declared preview port to
#      open when a template is built; a cold Leptos wasm build takes minutes. The first attempt at
#      this template died on exactly that ("Timeout of 60000ms exceeded waiting for port 3000 to
#      open") and no template was created. A warm build makes the dev server's first compile a
#      ~1 second incremental one.
#
# Only `trunk` is installed here. The wasm-bindgen CLI and wasm-opt versions this app needs are
# pinned in `Trunk.toml` under `[tools]`, and Trunk downloads them itself — one less place for a
# version to drift out of step with `Cargo.lock`.
set -euo pipefail

TRUNK_VERSION="${TRUNK_VERSION:-0.21.14}"

rustup target add wasm32-unknown-unknown

# The Rust devcontainer image puts cargo in /usr/local/cargo (CARGO_HOME), NOT ~/.cargo — installing
# into `$HOME/.cargo/bin` there would create a directory nothing has on PATH, and the tool would look
# installed while `trunk` stayed "command not found" inside the sandbox.
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$CARGO_BIN"
export PATH="$CARGO_BIN:$PATH"

# Prebuilt tarball from Trunk's own GitHub release: compiling it from source is a multi-minute build
# on every cold boot, and the release binary is the thing their install script would fetch anyway.
fetch() { # fetch <url> <binary name>
  local tmp
  tmp="$(mktemp -d)"
  if curl -fsSL "$1" | tar -xz -C "$tmp"; then
    local found
    found="$(find "$tmp" -type f -name "$2" | head -1)"
    if [ -n "$found" ]; then
      install -m 755 "$found" "$CARGO_BIN/$2"
      rm -rf "$tmp"
      return 0
    fi
  fi
  rm -rf "$tmp"
  return 1
}

fetch "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" trunk \
  || cargo install trunk --version "$TRUNK_VERSION" --locked

command -v trunk >/dev/null || { echo "trunk is not on PATH ($CARGO_BIN)"; exit 1; }
trunk --version

# Warm the dependency graph AND the release build the dev server will re-run. See (2) above: this is
# load-bearing, not an optimisation.
cargo fetch
trunk build --release
echo "sandbox setup complete: the dev server starts from .codesandbox/tasks.json"
