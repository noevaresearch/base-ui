#!/usr/bin/env bash
# Turn a `cargo leptos build --release` output into the directory Cloudflare serves for
# https://baseui.noevaresearch.com.
#
# Why this exists
# ---------------
# `cargo leptos build` emits only the compiled bundle (`<site-root>/pkg/*`, plus the
# `assets-dir` copy `<site-root>/fonts/*`) — `crates/docs-app` is CSR-only, so there is no
# generated HTML entry document. The document that boots it is the *source* file
# `crates/docs-app/index.html`, which is what the local harness
# (`ralph/scripts/serve-docs-app.py`) hands back for every non-file route. Publishing needs
# the same two things the harness provides: that file at the site root, and an edge fallback
# (Cloudflare's `not_found_handling = "single-page-application"`, set in `wrangler.toml`) so a
# deep route such as `/react/components/checkbox` still loads it.
#
# The wasm bundle is also held to Cloudflare's per-static-asset ceiling (25 MiB, 26214400
# bytes). A debug build is ~36 MB and simply cannot be uploaded, so this script optionally
# shrinks the bundle with `wasm-opt` (when binaryen is installed) and then FAILS rather than
# producing a `dist/` that the deploy would reject — a size regression should be visible at
# the build step, not as an opaque 413 from the upload.
#
# Usage
# -----
#     bash crates/docs-app/deploy/assemble-site.sh [site-root] [index.html]
#
# Site root resolution order: $1, then $DOCS_APP_SITE_ROOT, then whichever of
# `<cargo target dir>/site` and `<repo>/target/site` actually holds `pkg/docs-app.wasm`
# (cargo-leptos resolves its `site-root` metadata differently depending on whether
# CARGO_TARGET_DIR is set, and both directories have been seen on this repo).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
INDEX="${2:-$REPO_ROOT/crates/docs-app/index.html}"
DEST="$REPO_ROOT/crates/docs-app/deploy/dist"
ASSET_CEILING=26214400   # Cloudflare: 25 MiB per static asset

# cargo may be redirecting its target dir via env (CI sets it; the dev box's committed
# `.cargo/config.toml` points it at /data/cargo-target instead).
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
  case "$CARGO_TARGET_DIR" in
    /*) TARGET_DIR="$CARGO_TARGET_DIR" ;;
    *)  TARGET_DIR="$REPO_ROOT/$CARGO_TARGET_DIR" ;;
  esac
else
  TARGET_DIR="$REPO_ROOT/target"
fi

find_site_root() {
  local candidate
  for candidate in "$@"; do
    if [ -n "$candidate" ] && [ -f "$candidate/pkg/docs-app.wasm" ]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

SITE_ROOT="$(find_site_root "${1:-}" "${DOCS_APP_SITE_ROOT:-}" "$TARGET_DIR/site" "$REPO_ROOT/target/site" "/data/cargo-target/site" || true)"
if [ -z "$SITE_ROOT" ]; then
  echo "assemble-site: no wasm bundle found. Looked for pkg/docs-app.wasm under:" >&2
  echo "  ${1:-<no arg>} ${DOCS_APP_SITE_ROOT:-<unset DOCS_APP_SITE_ROOT>} $TARGET_DIR/site $REPO_ROOT/target/site /data/cargo-target/site" >&2
  echo "Run \`cargo leptos build --release\` first (see .github/workflows/deploy-docs-app.yml)." >&2
  exit 2
fi
if [ ! -f "$INDEX" ]; then
  echo "assemble-site: no entry document at $INDEX" >&2
  exit 2
fi

echo "assemble-site: site root = $SITE_ROOT"
echo "assemble-site: entry document = $INDEX"

# --- optional size reduction -----------------------------------------------------------------
WASM="$SITE_ROOT/pkg/docs-app.wasm"
before="$(stat -c%s "$WASM")"
echo "assemble-site: docs-app.wasm is $before bytes ($(awk -v s="$before" 'BEGIN{printf "%.1f", s/1048576}') MiB)"
if [ "$before" -gt 12582912 ]; then  # > 12 MiB: worth shrinking while there is still headroom
  if command -v wasm-opt >/dev/null 2>&1; then
    if wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int "$WASM" -o "$WASM.opt" 2>/dev/null; then
      mv "$WASM.opt" "$WASM"
      echo "assemble-site: wasm-opt -Oz applied: $before -> $(stat -c%s "$WASM") bytes"
    else
      rm -f "$WASM.opt"
      echo "assemble-site: wasm-opt could not process this bundle; keeping it as built"
    fi
  else
    echo "assemble-site: wasm-opt not installed; skipping (apt-get install binaryen / brew install binaryen)"
  fi
fi

# --- assemble --------------------------------------------------------------------------------
rm -rf "$DEST"
mkdir -p "$DEST/pkg" "$DEST/fonts"
cp "$INDEX" "$DEST/index.html"

# The bundle files, minus the wasm-bindgen type declarations (`*.d.ts`), which are for
# TypeScript consumers of the crate, not for the browser.
shopt -s nullglob
for f in "$SITE_ROOT"/pkg/*; do
  case "$f" in
    *.d.ts) continue ;;
  esac
  cp "$f" "$DEST/pkg/"
done
if [ -d "$SITE_ROOT/fonts" ]; then
  cp "$SITE_ROOT"/fonts/* "$DEST/fonts/"
fi

# Identify what is live: a deploy that cannot say which commit it published cannot be checked.
{
  printf '{\n  "commit": "%s",\n  "branch": "%s",\n  "built_at": "%s"\n}\n' \
    "${GITHUB_SHA:-$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)}" \
    "${GITHUB_REF_NAME:-$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)}" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$DEST/version.json"

# --- guard -----------------------------------------------------------------------------------
final="$(stat -c%s "$DEST/pkg/docs-app.wasm")"
if [ "$final" -gt "$ASSET_CEILING" ]; then
  echo "assemble-site: docs-app.wasm is $final bytes ($(awk -v s="$final" 'BEGIN{printf "%.1f", s/1048576}') MiB), over Cloudflare's 25 MiB per-static-asset ceiling — refusing to assemble an undeployable site." >&2
  exit 1
fi

echo "assemble-site: $DEST assembled"
echo "  index.html, version.json, $(find "$DEST" -type f | wc -l) files, $(du -sh "$DEST" | cut -f1) total"
echo "  pkg/docs-app.wasm $(awk -v s="$final" 'BEGIN{printf "%.1f", s/1048576}') MiB of the 25 MiB ceiling"
