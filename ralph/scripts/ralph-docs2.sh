#!/usr/bin/env bash
# ralph-docs2 — Stage 2 (docs) demo-mining fan-out driver.
#
# Same isolation contract as ralph-docs1.sh/ralph2.sh: one BRAND-NEW `opencode` process per unit.
# Mines each demo's tailwind/css-modules source into `specs/docs-content/<name>/demos.json`, per
# `ralph/prompts/stage2-docs-mining.md`. Requires Stage 1 (docs)'s page.md to exist first, same
# ordering rule as ralph2.sh needing behavior.md.
#
# Usage:
#   bash ralph/scripts/ralph-docs2.sh                    # all components/* units with a page.md
#   bash ralph/scripts/ralph-docs2.sh accordion menu      # only these units
#   bash ralph/scripts/ralph-docs2.sh --parallel 5        # 5 opencode processes at once
#   bash ralph/scripts/ralph-docs2.sh --include-extra     # also mine Phase D-extra pages
#
# Env overrides:
#   OPENCODE_BIN     opencode executable (default: opencode)
#   OPENCODE_FLAGS   extra flags passed to every invocation (default: --auto)
#   FORCE=1          re-mine units that already have a demos.json (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage2-docs-mining.md"
MANIFEST="ralph/generated/docs-content.json"
LOG_DIR="ralph/logs/stage2-docs"
STATUS_DIR="${LOG_DIR}/.exitcodes"
mkdir -p "$LOG_DIR" "$STATUS_DIR"

if [ ! -f "$MANIFEST" ]; then
  echo "ralph-docs2: $MANIFEST missing — run 'node ralph/scripts/enumerate-demos.mjs' first." >&2
  exit 1
fi

PARALLEL=1
INCLUDE_EXTRA=0
unit_filters=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --parallel) PARALLEL="$2"; shift 2 ;;
    --parallel=*) PARALLEL="${1#--parallel=}"; shift ;;
    --include-extra) INCLUDE_EXTRA=1; shift ;;
    *) unit_filters+=("$1"); shift ;;
  esac
done
if ! [[ "$PARALLEL" =~ ^[0-9]+$ ]] || [ "$PARALLEL" -lt 1 ]; then
  echo "ralph-docs2: --parallel must be a positive integer, got \"$PARALLEL\"" >&2
  exit 1
fi

want_unit() {
  local name="$1"
  [ "${#unit_filters[@]}" -eq 0 ] && return 0
  for u in "${unit_filters[@]}"; do [ "$u" = "$name" ] && return 0; done
  return 1
}

mine_unit() {
  local name="$1" demos_json="$2"
  local spec_path="specs/docs-content/${name}/demos.json"

  local prompt_file
  prompt_file="$(mktemp)"
  {
    sed -e "s#{{unit}}#${name}#g" -e "s#{{demos}}#see JSON below#g" "$TEMPLATE"
    echo
    echo "{{demos}} for this unit (from ralph/generated/docs-content.json):"
    echo "$demos_json"
  } > "$prompt_file"

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph-docs2: mining '$name' -> $spec_path (log: $log_file)"

  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph-docs2: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph-docs2: '$name' done."
  fi
  return "$status"
}

active_jobs=0
attempted_units=()

while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  route="$(echo "$row" | jq -r '.route')"
  name="${route##*/}"

  # See ralph-docs1.sh's matching comment: utils/* also carries real docs-pair items, not just
  # components/*.
  if [ "$INCLUDE_EXTRA" -eq 0 ] && [[ "$route" != components/* ]] && [[ "$route" != utils/* ]]; then
    continue
  fi
  [[ "$route" == "components" || "$route" == "utils" ]] && continue

  want_unit "$name" || continue

  page_spec="specs/docs-content/${name}/page.md"
  demos_spec="specs/docs-content/${name}/demos.json"

  if [ ! -f "$page_spec" ]; then
    echo "ralph-docs2: skip '$name' — no $page_spec yet (run ralph-docs1.sh for this unit first)"
    continue
  fi
  if [ -f "$demos_spec" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "ralph-docs2: skip '$name' — $demos_spec already exists (set FORCE=1 to re-mine)"
    continue
  fi

  demos_json="$(echo "$row" | jq -c '.demos')"
  attempted_units+=("$name")

  if [ "$PARALLEL" -eq 1 ]; then
    mine_unit "$name" "$demos_json" || true
  else
    mine_unit "$name" "$demos_json" &
    active_jobs=$((active_jobs + 1))
    if [ "$active_jobs" -ge "$PARALLEL" ]; then
      wait -n || true
      active_jobs=$((active_jobs - 1))
    fi
  fi
done < <(jq -r '.[] | @base64' "$MANIFEST")

wait || true

failed=()
for name in "${attempted_units[@]}"; do
  code_file="${STATUS_DIR}/${name}"
  if [ -f "$code_file" ] && [ "$(cat "$code_file")" != "0" ]; then
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "ralph-docs2: done, but ${#failed[@]} unit(s) FAILED: ${failed[*]} — see ralph/logs/stage2-docs/<name>.log" >&2
else
  echo "ralph-docs2: done."
fi
