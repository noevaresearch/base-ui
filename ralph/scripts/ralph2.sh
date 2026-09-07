#!/usr/bin/env bash
# ralph2 — Stage 2 (implementation mining) fan-out driver.
#
# Same isolation contract as ralph1: one BRAND-NEW `opencode` process per unit, per Huntley's Ralph
# technique — nothing carries between units except what is on disk (specs/**, the unit's citation
# sidecar). `ralph/prompts/stage2-implementation-mining.md` is explicit that it runs "after Stage 1
# has produced specs/library/{{unit}}/behavior.md ... dispatched per-unit as a bounded fan-out
# task ... NOT a loop" — this script is that fan-out, in bash, with a fresh process per iteration.
# --parallel below only controls how many of those independent invocations run at once; it does
# NOT introduce cross-unit state, since each unit still only reads/writes its own spec files.
#
# A unit is only processed once its Stage 1 output (specs/library/{{unit}}/behavior.md) exists;
# units without one are skipped with a note, since implementation.md would have no ground truth to
# build on.
#
# Usage:
#   bash ralph/scripts/ralph2.sh                        # all library units with a behavior.md
#   bash ralph/scripts/ralph2.sh dialog menu             # only these units
#   bash ralph/scripts/ralph2.sh --parallel 5            # all units, 5 opencode processes at once
#
# With --parallel > 1, each unit's own `opencode` transcript still streams to stdout (via `tee`),
# so concurrent units' output WILL interleave in the terminal — an accepted tradeoff of real
# parallelism. Each unit's own log file under ralph/logs/stage2/ stays clean regardless.
#
# Units with `hasTests: false` in components.json (measured 2026-09-07: types,
# unstable-use-media-query) never get a behavior.md from ralph1.sh — Stage 1 mines from tests,
# and there are none to mine. Rather than skip them here too (they'd be silently dropped
# entirely, with neither script ever attempting them — the bug this comment is here to prevent
# regressing to), this driver routes them through `stage-source-only-mining.md` instead, which
# produces both behavior.md AND implementation.md from source in one pass.
#
# Env overrides:
#   OPENCODE_BIN     opencode executable (default: opencode)
#   OPENCODE_FLAGS   extra flags passed to every invocation (default: --auto)
#   FORCE=1          re-mine units that already have an implementation.md (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage2-implementation-mining.md"
SOURCE_ONLY_TEMPLATE="ralph/prompts/stage-source-only-mining.md"
LOG_DIR="ralph/logs/stage2"
STATUS_DIR="${LOG_DIR}/.exitcodes"
mkdir -p "$LOG_DIR" "$STATUS_DIR"

# ---- argument parsing: --parallel N, plus optional unit-name filters ----
PARALLEL=1
unit_filters=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --parallel)
      PARALLEL="$2"
      shift 2
      ;;
    --parallel=*)
      PARALLEL="${1#--parallel=}"
      shift
      ;;
    *)
      unit_filters+=("$1")
      shift
      ;;
  esac
done
if ! [[ "$PARALLEL" =~ ^[0-9]+$ ]] || [ "$PARALLEL" -lt 1 ]; then
  echo "ralph2: --parallel must be a positive integer, got \"$PARALLEL\"" >&2
  exit 1
fi

want_unit() {
  local name="$1"
  if [ "${#unit_filters[@]}" -eq 0 ]; then
    return 0
  fi
  for u in "${unit_filters[@]}"; do
    if [ "$u" = "$name" ]; then
      return 0
    fi
  done
  return 1
}

# Mines exactly one unit's implementation.md. Runs inline (PARALLEL=1) or backgrounded
# (PARALLEL>1) — identical body either way. Writes its own exit code to STATUS_DIR/<name> so the
# final summary can report failures even for backgrounded jobs.
mine_unit() {
  local name="$1" src_dir="$2"
  local impl_path="specs/library/${name}/implementation.md"

  local src_files
  src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"

  local prompt_file
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

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph2: mining '$name' -> $impl_path (log: $log_file)"

  # set +e/-e is scoped to this call only. When mine_unit runs backgrounded (`&`), bash forks a
  # subshell for it, so this toggle never leaks into the parent script's own -e setting.
  set +e
  # Same two fixes as ralph1.sh: `-f` ATTACHES a file rather than sending it as the message
  # (confirmed via `opencode run --help`), so the prompt must be the positional message argument;
  # and </dev/null is required so opencode can't inherit and drain this loop's row-pipe stdin
  # (`done < <(jq ...)`), which is what made the loop appear to finish early with no error.
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph2: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph2: '$name' done."
  fi
  return "$status"
}

# Mines exactly one hasTests:false unit's behavior.md AND implementation.md in one pass, via
# stage-source-only-mining.md. Same isolation/logging contract as mine_unit above.
mine_source_only_unit() {
  local name="$1" src_dir="$2"
  local impl_path="specs/library/${name}/implementation.md"

  local src_files
  src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"

  local prompt_file
  prompt_file="$(mktemp)"
  {
    sed \
      -e "s#{{unit}}#${name}#g" \
      -e "s#{{srcFiles}}#see list below#g" \
      "$SOURCE_ONLY_TEMPLATE"
    echo
    echo "{{srcFiles}} for this unit (non-test files under ${src_dir}):"
    echo "$src_files"
  } > "$prompt_file"

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph2: mining '$name' (source-only, no tests) -> $impl_path (log: $log_file)"

  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph2: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph2: '$name' done."
  fi
  return "$status"
}

active_jobs=0
attempted_units=()

# Stage 2 is source-only (packages/react/src units); implementation.md's own template only ever
# names specs/library/{{unit}}, so this driver is scoped to library components, not packages/utils.
while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  name="$(echo "$row" | jq -r '.[0]')"
  src_dir="$(echo "$row" | jq -r '.[1]')"

  want_unit "$name" || continue

  has_tests="$(echo "$row" | jq -r '.[2]')"
  behavior_path="specs/library/${name}/behavior.md"
  impl_path="specs/library/${name}/implementation.md"

  if [ "$has_tests" = "false" ]; then
    # No test files exist for this unit — Stage 1 never ran (see ralph1.sh's own comment on the
    # same filter). Route through the source-only combined template instead of requiring
    # behavior.md to pre-exist.
    if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
      echo "ralph2: skip '$name' — $impl_path already exists (set FORCE=1 to re-mine)"
      continue
    fi

    attempted_units+=("$name")

    if [ "$PARALLEL" -eq 1 ]; then
      mine_source_only_unit "$name" "$src_dir" || true
    else
      mine_source_only_unit "$name" "$src_dir" &
      active_jobs=$((active_jobs + 1))
      if [ "$active_jobs" -ge "$PARALLEL" ]; then
        wait -n || true
        active_jobs=$((active_jobs - 1))
      fi
    fi
    continue
  fi

  if [ ! -f "$behavior_path" ]; then
    echo "ralph2: skip '$name' — no $behavior_path yet (run ralph1.sh for this unit first)"
    continue
  fi
  if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "ralph2: skip '$name' — $impl_path already exists (set FORCE=1 to re-mine)"
    continue
  fi

  attempted_units+=("$name")

  # || true on every path below: a single unit failing must not abort the whole batch under
  # set -e — failures are tracked via STATUS_DIR and reported in the summary at the end instead.
  if [ "$PARALLEL" -eq 1 ]; then
    mine_unit "$name" "$src_dir" || true
  else
    mine_unit "$name" "$src_dir" &
    active_jobs=$((active_jobs + 1))
    if [ "$active_jobs" -ge "$PARALLEL" ]; then
      wait -n || true
      active_jobs=$((active_jobs - 1))
    fi
  fi
done < <(jq -r '.[] | [.name, .srcDir, .hasTests] | @base64' ralph/generated/components.json)

# Drain any still-running background jobs before summarizing.
wait || true

failed=()
for name in "${attempted_units[@]}"; do
  code_file="${STATUS_DIR}/${name}"
  if [ -f "$code_file" ] && [ "$(cat "$code_file")" != "0" ]; then
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "ralph2: done, but ${#failed[@]} unit(s) FAILED: ${failed[*]} — see ralph/logs/stage2/<name>.log" >&2
else
  echo "ralph2: done."
fi
