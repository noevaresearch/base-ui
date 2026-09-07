#!/usr/bin/env bash
# classralph — the canonical Ralph loop (Stage 3: forward loop / port execution).
#
# This is "classic Ralph" in Geoffrey Huntley's original sense: a `while` loop where every single
# iteration is a brand-new, disposable `opencode run` process with NO memory of any previous
# iteration. The only channel between iterations is what got committed to git: TODO.md's status
# fields, the specs/** the item cites, and the crate code itself. That is the entire point of the
# technique — unlike ralph1.sh/ralph2.sh (bounded fan-outs over a fixed unit list), this loop is
# open-ended: it keeps going, one bounded objective per iteration (Principle 2), until there is
# nothing left that is unblocked, or something tells it to stop.
#
# Each iteration does, in order:
#   1. `pick-next-todo.mjs` selects exactly one TODO.md item (deterministic, dependency-aware).
#   2. `stage3-forward-loop.md` is rendered with that item's id.
#   3. A brand-new `opencode run` process gets that prompt and does the work end-to-end, INCLUDING
#      running `run-regression.sh` itself before checking the item off (see that prompt, step 7).
#   4. This driver independently re-runs run-regression.sh too — an iteration cannot fake a
#      "done" past this loop by only self-reporting success (see run-regression.sh's own header).
#   5. Progress (or a `blocked` status) is committed by the iteration itself; if it did not commit
#      anything, that is treated as a stall and the loop halts rather than spinning forever.
#
# Fallback selection: if pick-next-todo.mjs finds nothing pickable, that can mean the port is
# complete, everything left is genuinely blocked, OR the blocked-by graph is overly conservative
# (generate-todo.mjs's coarse default blocks every Phase B item on "Phase A complete" even though
# most only need a handful of specific deps). Rather than halt outright, one fallback iteration
# gets ralph/prompts/stage3-fallback-selection.md instead — the agent studies TODO.md/specs itself
# and may narrow one item's blocked-by (with justification) and complete it, mirroring Huntley's
# original "study specs, pick the most important thing" pattern for this one edge case. The
# mechanical picker remains the default path every other iteration.
#
# Usage:
#   bash ralph/scripts/classralph.sh
#
# Env overrides:
#   OPENCODE_BIN        opencode executable (default: opencode)
#   OPENCODE_FLAGS      extra flags passed to every invocation (default: --auto)
#   MAX_ITERATIONS      safety cap on iterations (default: 500)
#   TODO_PATH           path to TODO.md (default: TODO.md)
#   STOP_FILE           if this file appears, halt after the current iteration (default: .ralph-stop)
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage3-forward-loop.md"
TODO_PATH="${TODO_PATH:-TODO.md}"
MAX_ITERATIONS="${MAX_ITERATIONS:-500}"
STOP_FILE="${STOP_FILE:-.ralph-stop}"
LOG_DIR="ralph/logs/stage3"
mkdir -p "$LOG_DIR"

iteration=0
while [ "$iteration" -lt "$MAX_ITERATIONS" ]; do
  if [ -f "$STOP_FILE" ]; then
    echo "classralph: stop file '$STOP_FILE' present — halting after $iteration iteration(s)."
    break
  fi

  todo_id="$(node ralph/scripts/pick-next-todo.mjs "$TODO_PATH")"
  pick_status=$?
  fallback_mode=false
  if [ "$pick_status" -ne 0 ] || [ -z "$todo_id" ]; then
    # The mechanical picker's one failure mode: nothing not-started has all blocked-by deps done.
    # This can mean the port is complete, everything left is genuinely blocked, OR the blocked-by
    # graph is overly conservative (generate-todo.mjs's coarse default: every Phase B item lists
    # "Phase A complete" even though most only need a handful of specific deps). Rather than halt
    # outright, give one fallback iteration a chance to tell these apart — see
    # ralph/prompts/stage3-fallback-selection.md for exactly what it's allowed to do.
    fallback_mode=true
    todo_id="FALLBACK"
  fi

  iteration=$((iteration + 1))
  before_sha="$(git rev-parse HEAD)"
  ts="$(date +%Y%m%d-%H%M%S)"
  # todo_id may contain slashes/spaces; make a filesystem-safe log name.
  safe_id="$(echo "$todo_id" | tr -c 'A-Za-z0-9._-' '-')"
  log_file="${LOG_DIR}/${iteration}-${safe_id}-${ts}.log"

  echo "classralph: iteration $iteration/$MAX_ITERATIONS — item: \"$todo_id\" (log: $log_file)"

  prompt_file="$(mktemp)"
  if [ "$fallback_mode" = true ]; then
    cp ralph/prompts/stage3-fallback-selection.md "$prompt_file"
  else
    sed "s#{{todo-id}}#${todo_id}#g" "$TEMPLATE" > "$prompt_file"
  fi

  # Fresh process. This session has no memory of any prior iteration beyond TODO.md/specs/git.
  "$OPENCODE_BIN" run $OPENCODE_FLAGS -f "$prompt_file" > "$log_file" 2>&1
  agent_status=$?
  rm -f "$prompt_file"

  if [ "$agent_status" -ne 0 ]; then
    echo "classralph: iteration for \"$todo_id\" FAILED (opencode exit $agent_status) — see $log_file" >&2
    echo "classralph: halting rather than retrying blindly. Inspect the log, then resume." >&2
    break
  fi

  after_sha="$(git rev-parse HEAD)"
  if [ "$after_sha" = "$before_sha" ]; then
    if [ "$fallback_mode" = true ]; then
      echo "classralph: fallback iteration made no commit — it determined the port is complete," \
           "everything remaining is genuinely blocked, or it only reported findings. Halting; see $log_file." >&2
    else
      echo "classralph: iteration for \"$todo_id\" made no commit (item was likely marked 'blocked' \
without committing, per the prompt's step 7/8) — halting so a human can look at $log_file." >&2
    fi
    break
  fi

  if [ "$fallback_mode" = true ]; then
    # We don't know in advance which item(s) the fallback picked — detect it from the diff.
    before_todo="$(mktemp)"
    git show "$before_sha:$TODO_PATH" > "$before_todo"
    changed_ids="$(node ralph/scripts/diff-todo-done.mjs "$before_todo" "$TODO_PATH")"
    rm -f "$before_todo"

    if [ -z "$changed_ids" ]; then
      echo "classralph: fallback committed something but marked no item 'done' — treating as a \
stall (it should either finish an item or report findings without committing). Halting." >&2
      break
    fi

    all_ok=true
    while IFS= read -r changed_id; do
      [ -z "$changed_id" ] && continue
      echo "classralph: fallback marked \"$changed_id\" done — re-running regression independently..."
      if ! bash ralph/scripts/run-regression.sh "$changed_id" >> "$log_file" 2>&1; then
        echo "classralph: independent regression check FAILED for fallback-selected \"$changed_id\" \
after commit $after_sha — see $log_file. Halting; this item's 'done' status should not be trusted." >&2
        all_ok=false
      else
        echo "classralph: \"$changed_id\" (via fallback) verified OK at $after_sha."
      fi
    done <<< "$changed_ids"
    [ "$all_ok" = false ] && break
    continue
  fi

  # Independent re-verification: the loop does not trust the iteration's own self-report.
  echo "classralph: re-running regression independently for \"$todo_id\"..."
  if ! bash ralph/scripts/run-regression.sh "$todo_id" >> "$log_file" 2>&1; then
    echo "classralph: independent regression check FAILED for \"$todo_id\" after commit $after_sha \
— see $log_file. Halting; this item's 'done' status should not be trusted." >&2
    break
  fi

  echo "classralph: \"$todo_id\" verified OK at $after_sha."
done

echo "classralph: stopped after $iteration iteration(s)."
