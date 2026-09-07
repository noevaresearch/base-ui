#!/usr/bin/env bash
# ralph1 — Stage 1 (behavior mining) fan-out driver.
#
# Per Geoffrey Huntley's Ralph technique: every unit is mined by a BRAND-NEW `opencode` process,
# not a subagent dispatched from one long-lived session. The only thing that carries from one unit
# to the next is what got written to disk (specs/**, this unit's citation sidecar) — no process here
# remembers any other unit. This mirrors `ralph/prompts/stage1-behavior-mining.md`'s own framing:
# "Filled in and dispatched per-unit as a bounded fan-out task ... NOT a loop" — this script is
# that fan-out; each unit's `opencode run` call is a fresh, unrelated invocation regardless of
# --parallel below, which only controls how many of those independent invocations run at once —
# it does NOT introduce shared state between units (each still only touches its own spec files),
# so it doesn't compromise the "no cross-unit memory" property, unlike using a coordinating
# long-lived session (e.g. dispatching subagents from Claude Code) would for the STAGE 3 forward
# loop — see the conversation this was built from for why that distinction matters there but not
# here: Stage 1/2 units are independent, order-agnostic archaeology, not a sequence where drift
# can compound.
#
# Usage:
#   bash ralph/scripts/ralph1.sh                        # all units, one at a time (default)
#   bash ralph/scripts/ralph1.sh dialog menu             # only these units
#   bash ralph/scripts/ralph1.sh --parallel 5            # all units, 5 opencode processes at once
#   bash ralph/scripts/ralph1.sh --parallel 5 dialog menu button avatar checkbox
#
# With --parallel > 1, each unit's own `opencode` transcript still streams to stdout (via `tee`),
# so concurrent units' output WILL interleave in the terminal — that's an inherent, accepted
# tradeoff of real parallelism. Each unit's own log file under ralph/logs/stage1/ stays clean
# regardless, since each unit only ever writes to its own log file.
#
# Env overrides:
#   OPENCODE_BIN     opencode executable (default: opencode)
#   OPENCODE_FLAGS   extra flags passed to every invocation (default: --auto)
#   FORCE=1          re-mine units that already have a behavior.md (default: skip them)
#
# `select(.hasTests == true)` below deliberately EXCLUDES units with no test files (measured
# 2026-09-07: types, unstable-use-media-query, both library units) — Stage 1's whole premise is
# mining observable behavior FROM TESTS, and there is nothing to mine when none exist. Those
# units are not silently lost: ralph2.sh handles them itself via
# `ralph/prompts/stage-source-only-mining.md`, which produces both behavior.md AND
# implementation.md from source directly, in one pass, skipping this stage entirely.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage1-behavior-mining.md"
LOG_DIR="ralph/logs/stage1"
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
  echo "ralph1: --parallel must be a positive integer, got \"$PARALLEL\"" >&2
  exit 1
fi

# Build the (name, kind, testFiles) rows to process.
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

# Mines exactly one unit. Runs either inline (PARALLEL=1) or backgrounded (PARALLEL>1) — the body
# is identical either way; only how it's invoked differs. Writes its own exit code to
# STATUS_DIR/<name> so the final summary can report failures even for backgrounded jobs, since a
# background job can't hand its exit status back to the parent any other way that's simple/portable.
mine_unit() {
  local name="$1" kind="$2" test_files="$3"
  local spec_path
  if [ "$kind" = "library" ]; then
    spec_path="specs/library/${name}/behavior.md"
  else
    spec_path="specs/utils/${name}.md"
  fi

  local prompt_file
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

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph1: mining '$name' -> $spec_path (log: $log_file)"

  # Two things matter here, both found the hard way:
  # 1. `-f`/`--file` on `opencode run` ATTACHES a file to the message — it is not how you pass
  #    the message itself (confirmed via `opencode run --help`). The actual prompt must be the
  #    positional message argument, the same way classralph.sh passes claude's prompt via
  #    `-p "$(cat "$prompt_file")"`.
  # 2. </dev/null is required regardless of --parallel: without it, this invocation would inherit
  #    whatever stdin its caller has, and in the PARALLEL=1 (non-backgrounded) case that caller is
  #    the top-level loop reading from `<(rows)` — opencode could silently drain the remaining
  #    unit rows from that shared pipe (observed: the loop appeared to finish early with no error
  #    after only 2 real invocations out of ~90, with the leaked pipe bytes incidentally being
  #    what satisfied opencode's "you must provide a message" check when `-f` alone left the
  #    message empty).
  # set +e/-e is scoped to this call only. When mine_unit runs backgrounded (`&`), bash forks a
  # subshell for it, so this toggle never leaks into the parent script's own -e setting.
  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph1: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph1: '$name' done."
  fi
  return "$status"
}

active_jobs=0
attempted_units=()

while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  name="$(echo "$row" | jq -r '.[0]')"
  kind="$(echo "$row" | jq -r '.[1]')"

  want_unit "$name" || continue

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
  attempted_units+=("$name")

  # || true on every path below: a single unit failing must not abort the whole batch under
  # set -e (mine_unit's own `return "$status"` would otherwise propagate here, and `wait -n`/
  # `wait` return the exit status of the job(s) they collect) — failures are tracked via
  # STATUS_DIR and reported in the summary at the end instead, matching the original script's
  # "report all failures, don't stop the batch" behavior.
  if [ "$PARALLEL" -eq 1 ]; then
    mine_unit "$name" "$kind" "$test_files" || true
  else
    mine_unit "$name" "$kind" "$test_files" &
    active_jobs=$((active_jobs + 1))
    if [ "$active_jobs" -ge "$PARALLEL" ]; then
      wait -n || true
      active_jobs=$((active_jobs - 1))
    fi
  fi
done < <(rows)

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
  echo "ralph1: done, but ${#failed[@]} unit(s) FAILED: ${failed[*]} — see ralph/logs/stage1/<name>.log" >&2
else
  echo "ralph1: done."
fi
