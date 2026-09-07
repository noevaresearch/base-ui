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
#   1. `pick-next-todo.mjs` suggests one TODO.md item (deterministic, dependency-aware) — or the
#      sentinel "NONE" if it finds nothing pickable.
#   2. `stage3-forward-loop.md` is rendered with that suggestion. The prompt is explicit that this
#      is a suggestion, not a mandate (see its "Step 0" section): the agent reads TODO.md/specs
#      itself and may confirm it, or pick a different item instead — including narrowing an
#      overly conservative `blocked-by` (generate-todo.mjs's coarse default blocks every Phase B
#      item on "Phase A complete" even though most only need a handful of specific deps), or
#      determining nothing is actually pickable. This mirrors Huntley's original pattern of
#      letting the agent itself judge the most important next task, rather than always deferring
#      to a mechanical rule — the picker exists to save most iterations the trouble, not to
#      constrain the ones with a legitimate reason to differ.
#   3. A brand-new `opencode run` process gets that prompt and does the work end-to-end, INCLUDING
#      running `run-regression.sh` itself before checking the item off (see that prompt, step 7).
#   4. Because the agent may have worked on a different item than suggested, this driver never
#      assumes which item(s) it completed — it diffs TODO.md before/after to detect that, then
#      independently re-runs run-regression.sh for each (see run-regression.sh's own header): an
#      iteration cannot fake a "done" past this loop by only self-reporting success.
#   5. Progress (or a `blocked` status) is committed by the iteration itself; if it did not commit
#      anything, that is treated as a stall and the loop halts rather than spinning forever.
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

  suggested_id="$(node ralph/scripts/pick-next-todo.mjs "$TODO_PATH")"
  pick_status=$?
  if [ "$pick_status" -ne 0 ] || [ -z "$suggested_id" ]; then
    # The mechanical picker's one failure mode: nothing not-started has all blocked-by deps done.
    # Don't halt outright — hand the agent "NONE" and let it judge for itself (per the prompt's
    # "Step 0") whether the port is complete, everything is genuinely blocked, or the blocked-by
    # graph is just overly conservative and something is actually workable.
    suggested_id="NONE"
  fi

  iteration=$((iteration + 1))
  before_sha="$(git rev-parse HEAD)"
  ts="$(date +%Y%m%d-%H%M%S)"
  # suggested_id may contain slashes/spaces; make a filesystem-safe log name.
  safe_id="$(echo "$suggested_id" | tr -c 'A-Za-z0-9._-' '-')"
  log_file="${LOG_DIR}/${iteration}-${safe_id}-${ts}.log"

  echo "classralph: iteration $iteration/$MAX_ITERATIONS — suggested: \"$suggested_id\" (log: $log_file)"

  prompt_file="$(mktemp)"
  sed "s#{{todo-id}}#${suggested_id}#g" "$TEMPLATE" > "$prompt_file"

  # Fresh process. This session has no memory of any prior iteration beyond TODO.md/specs/git.
  "$OPENCODE_BIN" run $OPENCODE_FLAGS -f "$prompt_file" > "$log_file" 2>&1
  agent_status=$?
  rm -f "$prompt_file"

  if [ "$agent_status" -ne 0 ]; then
    echo "classralph: iteration (suggested \"$suggested_id\") FAILED (opencode exit $agent_status) \
— see $log_file" >&2
    echo "classralph: halting rather than retrying blindly. Inspect the log, then resume." >&2
    break
  fi

  after_sha="$(git rev-parse HEAD)"
  if [ "$after_sha" = "$before_sha" ]; then
    echo "classralph: iteration (suggested \"$suggested_id\") made no commit — it likely determined \
the port is complete, everything remaining is genuinely blocked, marked an item 'blocked' without \
committing, or only reported findings. Halting; see $log_file." >&2
    break
  fi

  # The agent may have worked on a different item than suggested (Step 0 lets it override), so
  # never assume — detect which item(s) actually became `done` from the TODO.md diff itself.
  before_todo="$(mktemp)"
  git show "$before_sha:$TODO_PATH" > "$before_todo"
  changed_ids="$(node ralph/scripts/diff-todo-done.mjs "$before_todo" "$TODO_PATH")"
  rm -f "$before_todo"

  if [ -z "$changed_ids" ]; then
    echo "classralph: iteration (suggested \"$suggested_id\") committed something but marked no \
item 'done' — treating as a stall (it should either finish an item or report findings without \
committing). Halting." >&2
    break
  fi

  all_ok=true
  while IFS= read -r changed_id; do
    [ -z "$changed_id" ] && continue
    if [ "$changed_id" != "$suggested_id" ]; then
      echo "classralph: note — agent worked on \"$changed_id\", overriding the suggestion \"$suggested_id\"."
    fi
    echo "classralph: \"$changed_id\" marked done — re-running regression independently..."
    if ! bash ralph/scripts/run-regression.sh "$changed_id" >> "$log_file" 2>&1; then
      echo "classralph: independent regression check FAILED for \"$changed_id\" after commit \
$after_sha — see $log_file. Halting; this item's 'done' status should not be trusted." >&2
      all_ok=false
    else
      echo "classralph: \"$changed_id\" verified OK at $after_sha."
    fi
  done <<< "$changed_ids"
  [ "$all_ok" = false ] && break
done

echo "classralph: stopped after $iteration iteration(s)."
