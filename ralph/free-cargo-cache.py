"""Free regenerable cargo build caches so the docs-app gate can link.

Why this exists: /data is a 25G mount and /data/cargo-target is the shared
cargo target dir, so a full workspace gate can hit "No space left on device"
mid-link. The directories removed here are pure build caches — incremental
compilation state and cargo-leptos' `front` output — that cargo rebuilds on
demand; the compiled host rlibs the workspace gate depends on are left alone.

Usage: python3 ralph/free-cargo-cache.py [target-filter ...]
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

TARGET_DIR = Path("/data/cargo-target")
CACHES = [
    TARGET_DIR / "debug" / "incremental",
    TARGET_DIR / "wasm32-unknown-unknown" / "debug" / "incremental",
    TARGET_DIR / "front",
]


def main() -> None:
    filters = sys.argv[1:]
    for cache in CACHES:
        if filters and not any(f in str(cache) for f in filters):
            continue
        if cache.exists():
            size = sum(f.stat().st_size for f in cache.rglob("*") if f.is_file())
            shutil.rmtree(cache)
            print(f"removed {cache} ({size / 1e9:.2f} GB)")


if __name__ == "__main__":
    main()
