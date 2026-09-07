#!/usr/bin/env bash
# ralph-docs1 — Stage 1 (docs) page-mining fan-out driver.
#
# Same isolation contract as ralph1.sh: one BRAND-NEW `opencode` process per docs page, per
# Huntley's Ralph technique — nothing carries between pages except what is on disk (specs/**).
# Mines `docs/src/app/(docs)/react/components/<name>/page.mdx` into
# `specs/docs-content/<name>/page.md`, per `ralph/prompts/stage1-docs-mining.md`.
#
# Scoped to routes under `components/` by default — these are the ones TODO.md's Phase D actually
# gates on (docs-pair: docs-content: components/<name>). Phase D-extra (handbook/*, overview/*,
# and the bare `components`/`utils`/`handbook`/`overview` index pages) is explicitly "non-gating,
# ported last" in TODO.md, so it's opt-in here via --include-extra rather than mined by default.
#
# Usage:
#   bash ralph/scripts/ralph-docs1.sh                    # all components/* docs pages
#   bash ralph/scripts/ralph-docs1.sh accordion menu      # only these units
#   bash ralph/scripts/ralph-docs1.sh --parallel 5        # 5 opencode processes at once
#   bash ralph/scripts/ralph-docs1.sh --include-extra     # also mine Phase D-extra pages
#
# Env overrides:
#   OPENCODE_BIN     opencode executable (default: opencode)
#   OPENCODE_FLAGS   extra flags passed to every invocation (default: --auto)
#   FORCE=1          re-mine units that already have a page.md (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage1-docs-mining.md"
MANIFEST="ralph/generated/docs-content.json"
LOG_DIR="ralph/logs/stage1-docs"
STATUS_DIR="${LOG_DIR}/.exitcodes"
mkdir -p "$LOG_DIR" "$STATUS_DIR"

if [ ! -f "$MANIFEST" ]; then
  echo "ralph-docs1: $MANIFEST missing — run 'node ralph/scripts/enumerate-demos.mjs' first." >&2
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
  echo "ralph-docs1: --parallel must be a positive integer, got \"$PARALLEL\"" >&2
  exit 1
fi

want_unit() {
  local name="$1"
  [ "${#unit_filters[@]}" -eq 0 ] && return 0
  for u in "${unit_filters[@]}"; do [ "$u" = "$name" ] && return 0; done
  return 1
}

mine_page() {
  local name="$1" route="$2" page_path="$3"
  local spec_path="specs/docs-content/${name}/page.md"

  local prompt_file
  prompt_file="$(mktemp)"
  sed \
    -e "s#{{unit}}#${name}#g" \
    -e "s#{{route}}#${route}#g" \
    -e "s#{{pagePath}}#${page_path}#g" \
    "$TEMPLATE" > "$prompt_file"

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph-docs1: mining '$name' ($route) -> $spec_path (log: $log_file)"

  # Same two fixes as ralph1.sh: message must be positional (not -f), and </dev/null so opencode
  # can't drain this loop's row-pipe stdin.
  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph-docs1: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph-docs1: '$name' done."
  fi
  return "$status"
}

active_jobs=0
attempted_units=()

while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  route="$(echo "$row" | jq -r '.[0]')"
  page_path="$(echo "$row" | jq -r '.[1]')"

  # Component page name = last path segment (e.g. "components/accordion" -> "accordion"),
  # matching the existing specs/docs-content/<name>/ and specs/library/<name>/ directory scheme.
  name="${route##*/}"

  if [ "$INCLUDE_EXTRA" -eq 0 ] && [[ "$route" != components/* ]]; then
    continue
  fi
  [[ "$route" == "components" ]] && continue # bare index page, no single owning component

  want_unit "$name" || continue

  spec_path="specs/docs-content/${name}/page.md"
  if [ -f "$spec_path" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "ralph-docs1: skip '$name' — $spec_path already exists (set FORCE=1 to re-mine)"
    continue
  fi

  attempted_units+=("$name")

  if [ "$PARALLEL" -eq 1 ]; then
    mine_page "$name" "$route" "$page_path" || true
  else
    mine_page "$name" "$route" "$page_path" &
    active_jobs=$((active_jobs + 1))
    if [ "$active_jobs" -ge "$PARALLEL" ]; then
      wait -n || true
      active_jobs=$((active_jobs - 1))
    fi
  fi
done < <(jq -r '.[] | [.route, .pagePath] | @base64' "$MANIFEST")

wait || true

failed=()
for name in "${attempted_units[@]}"; do
  code_file="${STATUS_DIR}/${name}"
  if [ -f "$code_file" ] && [ "$(cat "$code_file")" != "0" ]; then
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "ralph-docs1: done, but ${#failed[@]} unit(s) FAILED: ${failed[*]} — see ralph/logs/stage1-docs/<name>.log" >&2
else
  echo "ralph-docs1: done."
fi
