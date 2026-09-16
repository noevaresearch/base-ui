#!/usr/bin/env bash
# Prepare a VM sandbox for building the ported demos.
#
# What runs here is `cargo`, `cargo-leptos` and `wasm-bindgen` — the three things this sandbox
# cannot do without, and the three that must NOT be installed from source on every cold boot
# (`cargo install cargo-leptos` is a multi-minute build by itself, and a CodeSandbox synced template
# discards its memory snapshot on every commit to the backing branch). The image below ships the
# Rust toolchain; this script adds only what the image cannot know about.
#
# Versions are pinned to what the port builds with (wasm-bindgen must match the `wasm-bindgen`
# version the app's Cargo.lock resolves — a mismatch fails with a version complaint at the
# wasm-bindgen step, not at compile time):
#     cargo-leptos 0.3.7   wasm-bindgen-cli 0.2.128
set -euo pipefail

CARGO_LEPTOS_VERSION="${CARGO_LEPTOS_VERSION:-0.3.7}"
WASM_BINDGEN_VERSION="${WASM_BINDGEN_VERSION:-0.2.128}"

rustup target add wasm32-unknown-unknown

# Prebuilt tarballs from each project's own GitHub releases: the same approach (and the same
# fallback) as the docs deploy workflow, for the same reason — an install step must not depend on a
# moving path in a third-party repo.
fetch() { # fetch <url> <tool>
  local tmp
  tmp="$(mktemp -d)"
  if curl -fsSL "$1" | tar -xz -C "$tmp"; then
    local found
    found="$(find "$tmp" -type f -name "$2" | head -1)"
    if [ -n "$found" ]; then
      install -m 755 "$found" "$HOME/.cargo/bin/$2"
      rm -rf "$tmp"
      return 0
    fi
  fi
  rm -rf "$tmp"
  return 1
}

fetch "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-x86_64-unknown-linux-musl.tar.gz" cargo-leptos \
  || cargo install cargo-leptos --version "$CARGO_LEPTOS_VERSION" --locked

fetch "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" wasm-bindgen \
  || cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --locked

cargo leptos --version
wasm-bindgen --version

# Warm the dependency graph so the first edit → rebuild is the fast, incremental case rather than a
# cold `cargo fetch` of the whole Leptos tree inside the sandbox.
cargo fetch
echo "sandbox setup complete: the dev server starts from .codesandbox/tasks.json"
