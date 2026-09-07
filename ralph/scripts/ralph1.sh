#!/usr/bin/env bash
# ralph1 — Stage 1 (behavior mining) fan-out driver.
#
# Per Geoffrey Huntley's Ralph technique: every unit is mined by a BRAND-NEW `claude` process, not
# a subagent dispatched from one long-lived session. The only thing that carries from one unit to
# the next is what got written to disk (specs/**, this unit's citation sidecar) — no process here
# remembers any other unit. This mirrors `ralph/prompts/stage1-behavior-mining.md`'s own framing:
# "Filled in and dispatched per-unit as a bounded fan-out task ... NOT a loop" — the bash `for`
# below is the fan-out; each iteration's `claude` call is a fresh, unrelated invocation.
#
# Usage:
#   bash ralph/scripts/ralph1.sh                  # all units from ralph/generated/{components,utils}.json
#   bash ralph/scripts/ralph1.sh dialog menu       # only these units (by `name` field)
#
# Env overrides:
#   CLAUDE_BIN     claude executable (default: claude)
#   CLAUDE_FLAGS   extra flags passed to every invocation (default: --dangerously-skip-permissions)
#   FORCE=1        re-mine units that already have a behavior.md (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CLAUDE_BIN="${CLAUDE_BIN:-claude}"
CLAUDE_FLAGS="${CLAUDE_FLAGS:---dangerously-skip-permissions}"
TEMPLATE="ralph/prompts/stage1-behavior-mining.md"
LOG_DIR="ralph/logs/stage1"
mkdir -p "$LOG_DIR"

# Build the (name, kind, specPath, testFiles-newline-list) rows to process.
rows() {
  jq -r '.[] | select(.hasTests == true) |
    [.name, "library", .testFiles] as $r | $r' \
    ralph/generated/components.json 2>/dev/null | jq -r '@base64' || true

  jq -r '.[] |
    [.name, "utils", .testFiles] as $r | $r' \
    ralph/generated/utils.json 2>/dev/null | jq -r '@base64' || true
}

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

while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  name="$(echo "$row" | jq -r '.[0]')"
  kind="$(echo "$row" | jq -r '.[1]')"

  want_unit "$name" "$@" || continue

  if [ "$kind" = "library" ]; then
    spec_path="specs/library/${name}/behavior.md"
  else
    spec_path="specs/utils/${name}.md"
  fi

  if [ -f "$spec_path" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "ralph1: skip '$name' — $spec_path already exists (set FORCE=1 to re-mine)"
    continue
  fi

  test_files="$(echo "$row" | jq -r '.[2][]' | sed 's/^/- /')"

  prompt_file="$(mktemp)"
  {
    sed \
      -e "s#{{unit}}#${name}#g" \
      -e "s#{{testFiles}}#see list below#g" \
      -e "s#{{sharedHarnessFiles}}#detect from imports per the prompt's own instructions#g" \
      "$TEMPLATE"
    echo
    echo "{{testFiles}} for this unit:"
    echo "$test_files"
  } > "$prompt_file"

  log_file="${LOG_DIR}/${name}.log"
  echo "ralph1: mining '$name' -> $spec_path (log: $log_file)"

  set +e
  "$CLAUDE_BIN" -p "$(cat "$prompt_file")" $CLAUDE_FLAGS 2>&1 | tee "$log_file"
  status="${PIPESTATUS[0]}"
  set -e
  rm -f "$prompt_file"

  if [ "$status" -ne 0 ]; then
    echo "ralph1: '$name' FAILED (exit $status) — see $log_file" >&2
  fi
done < <(rows)

echo "ralph1: done."
