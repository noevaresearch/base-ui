#!/usr/bin/env python3
"""Serve the built `crates/docs-app` wasm bundle so the Playwright differential can run.

Why this exists
---------------
`ralph/scripts/playwright-diff.mjs` drives a real browser against
`http://127.0.0.1:3177/<route>` (its default port) and asserts the Leptos page
mounts. Nothing in the repo served that URL: `docs-app` is a CSR/wasm crate, so
`cargo leptos build` emits only `target/site/pkg/*` (no HTML entry point), and
the workspace's `site-addr` was 127.0.0.1:3000 — which on this box is already
taken by the Hermes WhatsApp bridge, so a `cargo leptos serve` there answers
with the bridge's 404 and the differential can never pass.

This is a plain static file server over the built artifacts, plus the one thing
a client-side-routed CSR app needs: a catch-all fallback to the single HTML
document, so a deep route such as `/react/components/fieldset` loads
`index.html` and lets `leptos_router` resolve the pathname in the browser.

Usage
-----
    python3 ralph/scripts/serve-docs-app.py            # foreground, port 3177
    python3 ralph/scripts/serve-docs-app.py --port 3177 &

Then: node ralph/scripts/playwright-diff.mjs --todo-id "docs-content: components/fieldset"

Requires `cargo leptos build` to have been run in `crates/docs-app` first.
"""

from __future__ import annotations

import argparse
import http.server
import os
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


class SpaHandler(http.server.SimpleHTTPRequestHandler):
    """Static files from the site root; every other path returns the index document."""

    site_root: Path
    index_document: bytes = b""

    def translate_path(self, path: str) -> str:
        # Strip the query/fragment, then serve the site root by default.
        path = path.split("?", 1)[0].split("#", 1)[0]
        return str(self.site_root / path.lstrip("/"))

    def do_GET(self) -> None:  # noqa: N802 (http.server's own naming)
        target = Path(self.translate_path(self.path))
        # A real file under the site root (the wasm bundle, its JS loader, the
        # stylesheet) is served as-is; anything else is a client-side route and
        # must fall back to the document that boots the wasm app.
        if target.is_file():
            return super().do_GET()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(self.index_document)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(self.index_document)

    def log_message(self, format: str, *args) -> None:  # noqa: A002 (base-class signature)
        # Keep the loop's logs readable; SimpleHTTPRequestHandler is very chatty.
        sys.stderr.write("serve-docs-app: " + (format % args) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=3177, help="port playwright-diff.mjs expects")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument(
        "--site-root",
        default=os.environ.get("DOCS_APP_SITE_ROOT", str(REPO_ROOT / "target" / "site")),
        help="cargo-leptos site root (holds pkg/)",
    )
    parser.add_argument(
        "--index",
        default=os.environ.get("DOCS_APP_INDEX", str(REPO_ROOT / "crates" / "docs-app" / "index.html")),
        help="the CSR entry document used for every non-file route",
    )
    args = parser.parse_args()

    site_root = Path(args.site_root)
    index = Path(args.index)
    if not (site_root / "pkg").is_dir():
        print(
            f"serve-docs-app: no wasm bundle at {site_root / 'pkg'} — run "
            f"`cd crates/docs-app && cargo leptos build` first",
            file=sys.stderr,
        )
        return 2
    if not index.is_file():
        print(f"serve-docs-app: no entry document at {index}", file=sys.stderr)
        return 2

    SpaHandler.site_root = site_root
    SpaHandler.index_document = index.read_bytes()

    with http.server.ThreadingHTTPServer((args.host, args.port), SpaHandler) as httpd:
        print(
            f"serve-docs-app: serving {site_root} on http://{args.host}:{args.port} "
            f"(SPA fallback: {index})",
            flush=True,
        )
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
