#!/usr/bin/env bash
# ralph2 — Stage 2 (implementation mining) fan-out driver.
#
# Same isolation contract as ralph1: one BRAND-NEW `claude` process per unit, per Huntley's Ralph
# technique — nothing carries between units except what is on disk (specs/**, the unit's citation
# sidecar). `ralph/prompts/stage2-implementation-mining.md` is explicit that it runs "after Stage 1
# has produced specs/library/{{unit}}/behavior.md ... dispatched per-unit as a bounded fan-out
# task ... NOT a loop" — this script is that fan-out, in bash, with a fresh process per iteration.
#
# A unit is only processed once its Stage 1 output (specs/library/{{unit}}/behavior.md) exists;
# units without one are skipped with a note, since implementation.md would have no ground truth to
# build on.
#
# Usage:
#   bash ralph/scripts/ralph2.sh                  # all library units with a behavior.md
#   bash ralph/scripts/ralph2.sh dialog menu       # only these units (by `name` field)
#
# Env overrides:
#   CLAUDE_BIN     claude executable (default: claude)
#   CLAUDE_FLAGS   extra flags passed to every invocation (default: --dangerously-skip-permissions)
#   FORCE=1        re-mine units that already have an implementation.md (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CLAUDE_BIN="${CLAUDE_BIN:-claude}"
CLAUDE_FLAGS="${CLAUDE_FLAGS:---dangerously-skip-permissions}"
TEMPLATE="ralph/prompts/stage2-implementation-mining.md"
LOG_DIR="ralph/logs/stage2"
mkdir -p "$LOG_DIR"

want_unit() {
  local name="$1"
  if [ "$#" -le 1 ]; then
    return 0
  fi
  shift
  for u in "$@"; do
    if [ "$u" = "$name" ]; then
      return 0
    fi
  done
  return 1
}

# Stage 2 is source-only (packages/react/src units); implementation.md's own template only ever
# names specs/library/{{unit}}, so this driver is scoped to library components, not packages/utils.
while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  name="$(echo "$row" | jq -r '.[0]')"
  src_dir="$(echo "$row" | jq -r '.[1]')"

  want_unit "$name" "$@" || continue

  behavior_path="specs/library/${name}/behavior.md"
  impl_path="specs/library/${name}/implementation.md"

  if [ ! -f "$behavior_path" ]; then
    echo "ralph2: skip '$name' — no $behavior_path yet (run ralph1.sh for this unit first)"
    continue
  fi
  if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "ralph2: skip '$name' — $impl_path already exists (set FORCE=1 to re-mine)"
    continue
  fi

  src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"

  prompt_file="$(mktemp)"
  {
    sed \
      -e "s#{{unit}}#${name}#g" \
      -e "s#{{srcFiles}}#see list below#g" \
      "$TEMPLATE"
    echo
    echo "{{srcFiles}} for this unit (non-test files under ${src_dir}):"
    echo "$src_files"
  } > "$prompt_file"

  log_file="${LOG_DIR}/${name}.log"
  echo "ralph2: mining '$name' -> $impl_path (log: $log_file)"

  set +e
  "$CLAUDE_BIN" -p "$(cat "$prompt_file")" $CLAUDE_FLAGS 2>&1 | tee "$log_file"
  status="${PIPESTATUS[0]}"
  set -e
  rm -f "$prompt_file"

  if [ "$status" -ne 0 ]; then
    echo "ralph2: '$name' FAILED (exit $status) — see $log_file" >&2
  fi
done < <(jq -r '.[] | select(.hasTests == true) | [.name, .srcDir] | @base64' ralph/generated/components.json)

echo "ralph2: done."
