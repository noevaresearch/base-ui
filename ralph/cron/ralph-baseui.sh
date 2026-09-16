#!/usr/bin/env bash
# Ralph loop for /data/workspace/baseui — Base UI → Leptos migration
# Usage: ralph-baseui.sh [N_LOOPS]
set -euo pipefail
cd /data/workspace/baseui
# Load the dedicated OpenRouter key for this loop
set -a; source /data/.env; set +a
export OPENROUTER_API_KEY="$OPENROUTER_API_KEY_BASEUI"
export PATH="$HOME/.opencode/bin:$PATH"

N="${1:-1}"
for i in $(seq 1 "$N"); do
  echo "=== Ralph loop $i/$(date -u +%FT%TZ) ==="
  opencode run --model "openrouter/${RALPH_MODEL#z-ai/}" "$(cat PROMPT.md)" || echo "loop $i failed, continuing"
  git add -- TODO.md ralph/generated ralph/logs specs  # was: git add -A (swept the loop's in-progress source into a bot commit; it now stages only the ledger, generated data, logs and specs it actually owns) && git commit -m "ralph: loop $i $(date -u +%FT%TZ)" --allow-empty -q || true
done
